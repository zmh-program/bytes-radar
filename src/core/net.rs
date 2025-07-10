use crate::core::{
    analysis::{FileMetrics, ProjectAnalysis},
    error::{AnalysisError, Result},
    filter::{FilterStats, IntelligentFilter},
    registry::LanguageRegistry,
};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::io::{Cursor, Read};
use tar::Archive;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task;

static USER_AGENT: &str = "bytes-radar/1.0.0";

#[derive(Deserialize)]
struct GitHubRepoInfo {
    default_branch: String,
}

#[cfg(feature = "cli")]
use indicatif::ProgressBar;

pub struct RemoteAnalyzer {
    client: Client,
    github_token: Option<String>,
    timeout: u64,
    allow_insecure: bool,
    filter: IntelligentFilter,
    #[cfg(feature = "cli")]
    progress_bar: Option<ProgressBar>,
}

impl RemoteAnalyzer {
    pub fn new() -> Self {
        let mut builder = Client::builder().user_agent(USER_AGENT);

        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder.timeout(std::time::Duration::from_secs(300));
        }

        let client = builder.build().expect("Failed to create HTTP client");

        Self {
            client,
            github_token: None,
            timeout: 300,
            allow_insecure: false,
            filter: IntelligentFilter::default(),
            #[cfg(feature = "cli")]
            progress_bar: None,
        }
    }

    #[cfg(feature = "cli")]
    pub fn set_progress_bar(&mut self, progress_bar: Option<ProgressBar>) {
        self.progress_bar = progress_bar;
    }

    pub fn set_github_token(&mut self, token: &str) {
        self.github_token = Some(token.to_string());
        self.rebuild_client();
    }

    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = timeout;
        self.rebuild_client();
    }

    pub fn set_allow_insecure(&mut self, allow_insecure: bool) {
        self.allow_insecure = allow_insecure;
        self.rebuild_client();
    }

    pub fn set_filter(&mut self, filter: IntelligentFilter) {
        self.filter = filter;
    }

    pub fn set_aggressive_filtering(&mut self, enabled: bool) {
        if enabled {
            self.filter = IntelligentFilter::aggressive();
        } else {
            self.filter = IntelligentFilter::default();
        }
    }

    fn rebuild_client(&mut self) {
        let mut builder = Client::builder().user_agent(USER_AGENT);

        #[cfg(not(target_arch = "wasm32"))]
        {
            builder =
                builder.timeout(std::time::Duration::from_secs(self.timeout));
        }

        #[cfg(not(target_arch = "wasm32"))]
        if self.allow_insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(token) = &self.github_token {
            let mut headers = reqwest::header::HeaderMap::new();
            let auth_value = format!("token {token}");
            headers.insert(
                reqwest::header::AUTHORIZATION,
                auth_value.parse().expect("Invalid token format"),
            );
            builder = builder.default_headers(headers);
        }

        self.client = builder.build().expect("Failed to create HTTP client");
    }

    pub async fn analyze_url(&self, url: &str) -> Result<ProjectAnalysis> {
        let download_urls = self.resolve_git_url(url).await?;

        let mut url_errors: Vec<crate::core::error::DownloadUrlError> =
            Vec::new();
        #[allow(unused_variables)]
        let mut total_attempts = 0u32; //used for += 1
        for download_url in download_urls {
            total_attempts += 1;
            match self.analyze_tarball_with_name(&download_url, url).await {
                Ok(analysis) => return Ok(analysis),
                Err(e) => {
                    #[cfg(feature = "cli")]
                    log::debug!("Failed to download from {download_url}: {e}");

                    let error_info = crate::core::error::DownloadUrlError {
                        url: download_url.clone(),
                        error_message: format!("{e}"),
                        error_type: match e {
                            AnalysisError::NetworkError { .. } => {
                                "NetworkError".to_string()
                            }
                            AnalysisError::ArchiveError { .. } => {
                                "ArchiveError".to_string()
                            }
                            _ => "UnknownError".to_string(),
                        },
                        http_status_code: self.extract_http_status_code(&e),
                        retry_count: 1,
                    };

                    url_errors.push(error_info);
                    continue;
                }
            }
        }

        Err(AnalysisError::network("All download URLs failed"))
    }

    async fn resolve_git_url(&self, url: &str) -> Result<Vec<String>> {
        if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            return Ok(vec![url.to_string()]);
        }

        let expanded_url = self.expand_url(url);

        if (expanded_url.starts_with("http://")
            || expanded_url.starts_with("https://"))
            && !expanded_url.contains("github.com")
            && !expanded_url.contains("gitlab.com")
            && !expanded_url.contains("gitlab.")
            && !expanded_url.contains("bitbucket.org")
            && !expanded_url.contains("codeberg.org")
        {
            if expanded_url.ends_with(".tar.gz")
                || expanded_url.ends_with(".tgz")
            {
                return Ok(vec![expanded_url.to_string()]);
            }
            return Ok(vec![expanded_url.to_string()]);
        }

        let mut download_urls = Vec::new();

        if let Some(github_url) =
            self.parse_github_url_with_branch(&expanded_url)
        {
            download_urls.push(github_url);
        }

        if let Some(gitlab_url) =
            self.parse_gitlab_url_with_branch(&expanded_url)
        {
            download_urls.push(gitlab_url);
        }

        if let Some(bitbucket_url) =
            self.parse_bitbucket_url_with_branch(&expanded_url)
        {
            download_urls.push(bitbucket_url);
        }

        if let Some(codeberg_url) =
            self.parse_codeberg_url_with_branch(&expanded_url)
        {
            download_urls.push(codeberg_url);
        }

        if download_urls.is_empty() {
            let mut branches = vec![
                "main".to_string(),
                "master".to_string(),
                "develop".to_string(),
                "dev".to_string(),
            ];

            #[cfg(not(target_arch = "wasm32"))]
            if expanded_url.contains("github.com") {
                if let Some(default_branch) =
                    self.get_github_default_branch(&expanded_url).await
                {
                    branches.insert(0, default_branch);
                    branches.dedup();
                }
            }

            #[cfg(target_arch = "wasm32")]
            if expanded_url.contains("github.com") {
                branches = vec![
                    "main".to_string(),
                    "master".to_string(),
                    "develop".to_string(),
                    "dev".to_string(),
                ];
            }

            for branch in &branches {
                if let Some(github_url) =
                    self.parse_github_url(&expanded_url, branch)
                {
                    download_urls.push(github_url);
                }

                if let Some(gitlab_url) =
                    self.parse_gitlab_url(&expanded_url, branch)
                {
                    download_urls.push(gitlab_url);
                }

                if let Some(bitbucket_url) =
                    self.parse_bitbucket_url(&expanded_url, branch)
                {
                    download_urls.push(bitbucket_url);
                }

                if let Some(codeberg_url) =
                    self.parse_codeberg_url(&expanded_url, branch)
                {
                    download_urls.push(codeberg_url);
                }
            }
        }

        if download_urls.is_empty() {
            return Err(AnalysisError::url_parsing(format!(
                "Unsupported URL format or no accessible branch found: {expanded_url}. Please provide a direct tar.gz URL or a supported repository URL."
            )));
        }

        Ok(download_urls)
    }

    fn parse_github_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("github.com") {
            if url.contains("/tree/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(tree_pos) = parts.iter().position(|&x| x == "tree")
                {
                    if tree_pos + 1 < parts.len() && tree_pos >= 2 {
                        let owner = parts[tree_pos - 2];
                        let repo = parts[tree_pos - 1];
                        let branch = parts[tree_pos + 1];
                        return Some(format!(
                            "https://github.com/{owner}/{repo}/archive/refs/heads/{branch}.tar.gz"
                        ));
                    }
                }
            }

            if url.contains("/commit/") {
                return self.extract_github_commit_url(url);
            }
        }
        None
    }

    fn parse_gitlab_url_with_branch(&self, url: &str) -> Option<String> {
        if (url.contains("gitlab.com") || url.contains("gitlab."))
            && url.contains("/-/tree/")
        {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
                if tree_pos + 1 < parts.len() && tree_pos >= 3 {
                    let gitlab_pos = parts
                        .iter()
                        .position(|&x| x.contains("gitlab"))
                        .unwrap();
                    let host = parts[gitlab_pos];
                    let owner = parts[gitlab_pos + 1];
                    let repo = parts[gitlab_pos + 2];
                    let branch = parts[tree_pos + 1];
                    return Some(format!(
                        "https://{}/{}{}/-/archive/{}/{}-{}.tar.gz",
                        host,
                        owner,
                        if parts.len() > gitlab_pos + 3
                            && parts[gitlab_pos + 3] != "-"
                        {
                            format!(
                                "/{}",
                                parts[gitlab_pos + 3..tree_pos - 1].join("/")
                            )
                        } else {
                            String::new()
                        },
                        branch,
                        repo,
                        branch
                    ));
                }
            }
        }
        None
    }

    fn parse_bitbucket_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("bitbucket.org") {
            if url.contains("/commits/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(commits_pos) =
                    parts.iter().position(|&x| x == "commits")
                {
                    if commits_pos + 1 < parts.len() && commits_pos >= 2 {
                        let owner = parts[commits_pos - 2];
                        let repo = parts[commits_pos - 1];
                        let commit = parts[commits_pos + 1];
                        return Some(format!(
                            "https://bitbucket.org/{owner}/{repo}/get/{commit}.tar.gz"
                        ));
                    }
                }
            }

            if url.contains("/branch/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(branch_pos) =
                    parts.iter().position(|&x| x == "branch")
                {
                    if branch_pos + 1 < parts.len() && branch_pos >= 2 {
                        let owner = parts[branch_pos - 2];
                        let repo = parts[branch_pos - 1];
                        let branch = parts[branch_pos + 1];
                        return Some(format!(
                            "https://bitbucket.org/{owner}/{repo}/get/{branch}.tar.gz"
                        ));
                    }
                }
            }
        }
        None
    }

    fn parse_codeberg_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("codeberg.org") {
            if url.contains("/commit/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(commit_pos) =
                    parts.iter().position(|&x| x == "commit")
                {
                    if commit_pos + 1 < parts.len() && commit_pos >= 2 {
                        let owner = parts[commit_pos - 2];
                        let repo = parts[commit_pos - 1];
                        let commit = parts[commit_pos + 1];
                        return Some(format!(
                            "https://codeberg.org/{owner}/{repo}/archive/{commit}.tar.gz"
                        ));
                    }
                }
            }

            if url.contains("/src/branch/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(branch_pos) =
                    parts.iter().position(|&x| x == "branch")
                {
                    if branch_pos + 1 < parts.len() && branch_pos >= 3 {
                        let owner = parts[branch_pos - 3];
                        let repo = parts[branch_pos - 2];
                        let branch = parts[branch_pos + 1];
                        return Some(format!(
                            "https://codeberg.org/{owner}/{repo}/archive/{branch}.tar.gz"
                        ));
                    }
                }
            }
        }
        None
    }

    fn parse_bitbucket_url(&self, url: &str, branch: &str) -> Option<String> {
        if url.contains("bitbucket.org") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(bitbucket_pos) =
                parts.iter().position(|&x| x == "bitbucket.org")
            {
                if bitbucket_pos + 2 < parts.len() {
                    let owner = parts[bitbucket_pos + 1];
                    let repo = parts[bitbucket_pos + 2];
                    return Some(format!(
                        "https://bitbucket.org/{owner}/{repo}/get/{branch}.tar.gz"
                    ));
                }
            }
        }
        None
    }

    fn parse_codeberg_url(&self, url: &str, branch: &str) -> Option<String> {
        if url.contains("codeberg.org") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(codeberg_pos) =
                parts.iter().position(|&x| x == "codeberg.org")
            {
                if codeberg_pos + 2 < parts.len() {
                    let owner = parts[codeberg_pos + 1];
                    let repo = parts[codeberg_pos + 2];
                    return Some(format!(
                        "https://codeberg.org/{owner}/{repo}/archive/{branch}.tar.gz"
                    ));
                }
            }
        }
        None
    }

    fn extract_http_status_code(&self, error: &AnalysisError) -> Option<u16> {
        match error {
            AnalysisError::NetworkError { message } => {
                if message.contains("HTTP request failed with status: ") {
                    if let Some(start) =
                        message.find("HTTP request failed with status: ")
                    {
                        let status_start =
                            start + "HTTP request failed with status: ".len();
                        let status_str = &message[status_start..];
                        if let Some(end) = status_str.find(' ') {
                            status_str[..end].parse().ok()
                        } else {
                            status_str.parse().ok()
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn expand_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }

        if url.contains('/')
            && !url.starts_with("http://")
            && !url.starts_with("https://")
        {
            let parts: Vec<&str> = url.split('@').collect();
            let repo_part = parts[0];
            let branch_or_commit = parts.get(1);

            let path_parts: Vec<&str> = repo_part.split('/').collect();
            if path_parts.len() == 2 {
                if let Some(branch) = branch_or_commit {
                    if branch.len() >= 7
                        && branch.chars().all(|c| c.is_ascii_hexdigit())
                    {
                        return format!(
                            "https://github.com/{repo_part}/commit/{branch}"
                        );
                    } else {
                        return format!(
                            "https://github.com/{repo_part}/tree/{branch}"
                        );
                    }
                } else {
                    return format!("https://github.com/{repo_part}");
                }
            }
        }

        url.to_string()
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn get_github_default_branch(&self, url: &str) -> Option<String> {
        let (owner, repo) = self.extract_github_owner_repo(url)?;

        let api_url = format!("https://api.github.com/repos/{owner}/{repo}");

        match self.client.get(&api_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<GitHubRepoInfo>().await {
                        Ok(repo_info) => {
                            #[cfg(feature = "cli")]
                            log::debug!(
                                "GitHub API: Found default branch '{}' for {owner}/{repo}",
                                repo_info.default_branch
                            );
                            Some(repo_info.default_branch)
                        }
                        Err(_) => {
                            #[cfg(feature = "cli")]
                            log::debug!(
                                "GitHub API: Failed to parse response for {owner}/{repo}"
                            );
                            None
                        }
                    }
                } else {
                    #[cfg(feature = "cli")]
                    log::debug!(
                        "GitHub API: Request failed with status {} for {}/{}",
                        response.status(),
                        owner,
                        repo
                    );
                    None
                }
            }
            Err(_) => {
                #[cfg(feature = "cli")]
                log::debug!("GitHub API: Network error for {owner}/{repo}");
                None
            }
        }
    }

    fn extract_github_owner_repo(&self, url: &str) -> Option<(String, String)> {
        let url = url.trim_end_matches('/');

        if let Some(github_url) = url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = github_url.split('/').collect();
            if parts.len() >= 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        }

        if url.contains("github.com") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(github_pos) =
                parts.iter().position(|&x| x == "github.com")
            {
                if github_pos + 2 < parts.len() {
                    return Some((
                        parts[github_pos + 1].to_string(),
                        parts[github_pos + 2].to_string(),
                    ));
                }
            }
        }

        let parts: Vec<&str> = url.split('@').collect();
        let repo_part = parts[0];
        let path_parts: Vec<&str> = repo_part.split('/').collect();
        if path_parts.len() == 2 {
            return Some((
                path_parts[0].to_string(),
                path_parts[1].to_string(),
            ));
        }

        None
    }

    fn parse_github_url(&self, url: &str, branch: &str) -> Option<String> {
        let url = url.trim_end_matches('/');

        if url.contains("github.com") {
            if let Some(commit_url) = self.extract_github_commit_url(url) {
                return Some(commit_url);
            }

            if let Some(repo_url) = self.extract_github_repo_url(url, branch) {
                return Some(repo_url);
            }
        }

        None
    }

    fn extract_github_commit_url(&self, url: &str) -> Option<String> {
        if url.contains("/commit/") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(commit_pos) = parts.iter().position(|&x| x == "commit")
            {
                if commit_pos + 1 < parts.len() {
                    let owner = parts.get(parts.len() - 4)?;
                    let repo = parts.get(parts.len() - 3)?;
                    let commit = parts.get(commit_pos + 1)?;
                    return Some(format!(
                        "https://github.com/{owner}/{repo}/archive/{commit}.tar.gz"
                    ));
                }
            }
        }
        None
    }

    fn extract_github_repo_url(
        &self,
        url: &str,
        branch: &str,
    ) -> Option<String> {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 && parts.contains(&"github.com") {
            if let Some(github_pos) =
                parts.iter().position(|&x| x == "github.com")
            {
                if github_pos + 2 < parts.len() {
                    let owner = parts[github_pos + 1];
                    let repo = parts[github_pos + 2];
                    return Some(format!(
                        "https://github.com/{owner}/{repo}/archive/refs/heads/{branch}.tar.gz"
                    ));
                }
            }
        }
        None
    }

    fn parse_gitlab_url(&self, url: &str, branch: &str) -> Option<String> {
        let url = url.trim_end_matches('/');

        if url.contains("gitlab.com") || url.contains("gitlab.") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(gitlab_pos) =
                parts.iter().position(|&x| x.contains("gitlab"))
            {
                if gitlab_pos + 2 < parts.len() {
                    let host = parts[gitlab_pos];
                    let owner = parts[gitlab_pos + 1];
                    let repo = parts[gitlab_pos + 2];
                    return Some(format!(
                        "https://{host}/{owner}{}/-/archive/{branch}/{repo}-{branch}.tar.gz",
                        if parts.len() > gitlab_pos + 3 {
                            format!("/{}", parts[gitlab_pos + 3..].join("/"))
                        } else {
                            String::new()
                        }
                    ));
                }
            }
        }

        None
    }

    async fn analyze_tarball_with_name(
        &self,
        download_url: &str,
        original_url: &str,
    ) -> Result<ProjectAnalysis> {
        let project_name =
            self.extract_project_name_from_original(original_url);
        let mut project_analysis = ProjectAnalysis::new(project_name);

        let response =
            self.client.get(download_url).send().await.map_err(|e| {
                AnalysisError::network(format!("Failed to fetch URL: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(AnalysisError::network(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let total_size = response.content_length();

        #[cfg(feature = "cli")]
        if let Some(pb) = &self.progress_bar {
            if let Some(size) = total_size {
                use indicatif::ProgressStyle;
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {decimal_bytes_per_sec} {binary_bytes}/{binary_total_bytes} ({eta}) {msg}")
                        .unwrap_or_else(|_| ProgressStyle::default_bar())
                        .progress_chars("#>-"),
                );
                pb.set_length(size);
                pb.set_message("Downloading and processing...");
            } else {
                pb.set_message("Downloading and processing...");
                pb.enable_steady_tick(std::time::Duration::from_millis(120));
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let bytes = response.bytes().await.map_err(|e| {
                AnalysisError::network(format!(
                    "Failed to read response: {}",
                    e
                ))
            })?;

            #[cfg(feature = "cli")]
            if let Some(pb) = &self.progress_bar {
                pb.set_message("Processing archive...");
            }

            self.process_tarball_bytes(&bytes, &mut project_analysis)
                .await?;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let stream = response.bytes_stream();
            let stream_reader = StreamReader::new(
                stream,
                #[cfg(feature = "cli")]
                self.progress_bar.clone(),
                total_size,
            );

            #[cfg(feature = "cli")]
            if let Some(pb) = &self.progress_bar {
                pb.set_message("Processing archive...");
            }

            self.process_tarball_stream(stream_reader, &mut project_analysis)
                .await?;
        }

        Ok(project_analysis)
    }

    async fn process_tarball_stream(
        &self,
        stream_reader: StreamReader,
        project_analysis: &mut ProjectAnalysis,
    ) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let filter = self.filter.clone();
            let metrics_result = task::spawn_blocking(move || {
                let decoder = GzDecoder::new(stream_reader);
                let mut archive = Archive::new(decoder);

                let entries = archive.entries().map_err(|e| {
                    AnalysisError::archive(format!("Failed to read tar entries: {e}"))
                })?;

                let mut collected_metrics = Vec::new();
                let mut stats = FilterStats::new();

                for entry in entries {
                    let entry = entry.map_err(|e| {
                        AnalysisError::archive(format!("Failed to read tar entry: {e}"))
                    })?;

                    if let Ok(metrics) = Self::process_tar_entry_sync(entry, &filter, &mut stats) {
                        collected_metrics.push(metrics);
                    }
                }

                #[cfg(feature = "cli")]
                log::info!(
                    "Filter stats: processed {}/{} files ({:.1}% filtered), saved {}",
                    stats.processed,
                    stats.total_entries,
                    stats.filter_ratio() * 100.0,
                    stats.format_bytes_saved()
                );

                Ok::<Vec<FileMetrics>, AnalysisError>(collected_metrics)
            })
            .await
            .map_err(|e| AnalysisError::archive(format!("Task join error: {e}")))??;

            for metrics in metrics_result {
                project_analysis.add_file_metrics(metrics)?;
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let decoder = GzDecoder::new(stream_reader);
            let mut archive = Archive::new(decoder);

            let entries = archive.entries().map_err(|e| {
                AnalysisError::archive(format!(
                    "Failed to read tar entries: {e}"
                ))
            })?;

            let mut stats = FilterStats::new();

            for entry in entries {
                let entry = entry.map_err(|e| {
                    AnalysisError::archive(format!(
                        "Failed to read tar entry: {e}"
                    ))
                })?;

                if let Ok(metrics) = Self::process_tar_entry_sync(
                    entry,
                    &self.filter,
                    &mut stats,
                ) {
                    project_analysis.add_file_metrics(metrics)?;
                }
            }

            web_sys::console::log_1(
                &format!(
                    "Filter stats: processed {}/{} files ({:.1}% filtered), saved {}",
                    stats.processed,
                    stats.total_entries,
                    stats.filter_ratio() * 100.0,
                    stats.format_bytes_saved()
                )
                .into(),
            );
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    async fn process_tarball_bytes(
        &self,
        bytes: &bytes::Bytes,
        project_analysis: &mut ProjectAnalysis,
    ) -> Result<()> {
        let cursor = Cursor::new(bytes.as_ref());
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);

        let entries = archive.entries().map_err(|e| {
            AnalysisError::archive(format!("Failed to read tar entries: {e}"))
        })?;

        let mut stats = FilterStats::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                AnalysisError::archive(format!("Failed to read tar entry: {e}"))
            })?;

            if let Ok(metrics) =
                Self::process_tar_entry_sync(entry, &self.filter, &mut stats)
            {
                project_analysis.add_file_metrics(metrics)?;
            }
        }

        web_sys::console::log_1(
            &format!(
                "Filter stats: processed {}/{} files ({:.1}% filtered), saved {}",
                stats.processed,
                stats.total_entries,
                stats.filter_ratio() * 100.0,
                stats.format_bytes_saved()
            )
            .into(),
        );

        Ok(())
    }

    fn process_tar_entry_sync<R: Read>(
        mut entry: tar::Entry<'_, R>,
        filter: &IntelligentFilter,
        stats: &mut FilterStats,
    ) -> Result<FileMetrics> {
        let header = entry.header();
        let path = header.path().map_err(|e| {
            AnalysisError::archive(format!("Invalid path in tar entry: {e}"))
        })?;

        let file_path = path.to_string_lossy().to_string();

        if !header.entry_type().is_file() || header.size().unwrap_or(0) == 0 {
            return Err(AnalysisError::archive("Not a file or empty"));
        }

        let file_size = header.size().unwrap_or(0);

        let should_process = filter.should_process_file(&file_path, file_size);
        stats.record_entry(file_size, !should_process);

        if !should_process {
            return Err(AnalysisError::archive("File filtered out"));
        }

        let language = LanguageRegistry::detect_by_path(&file_path)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "Text".to_string());

        let mut content = String::new();
        if entry.read_to_string(&mut content).is_err() {
            return Err(AnalysisError::archive("Failed to read file content"));
        }

        analyze_file_content(&file_path, &content, &language, file_size)
    }

    fn extract_project_name_from_original(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            let url = url.trim_end_matches('/');

            if url.contains("/tree/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(tree_pos) = parts.iter().position(|&x| x == "tree")
                {
                    if tree_pos > 1 {
                        let repo = parts[tree_pos - 1];
                        let branch =
                            parts.get(tree_pos + 1).unwrap_or(&"unknown");
                        return format!("{repo}@{branch}");
                    }
                }
            }

            if url.contains("/commit/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(commit_pos) =
                    parts.iter().position(|&x| x == "commit")
                {
                    if commit_pos > 1 {
                        let repo = parts[commit_pos - 1];
                        let commit =
                            parts.get(commit_pos + 1).unwrap_or(&"unknown");
                        return format!(
                            "{repo}@{}",
                            &commit[..7.min(commit.len())]
                        );
                    }
                }
            }

            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 2 {
                let repo = parts[parts.len() - 1];
                return format!("{repo}@main");
            }
        } else if url.contains('/') && !url.contains('.') {
            let parts: Vec<&str> = url.split('@').collect();
            let repo_part = parts[0];
            let branch = parts.get(1).unwrap_or(&"main");

            if let Some(repo_name) = repo_part.split('/').last() {
                return format!("{repo_name}@{branch}");
            }
        }

        "remote-project".to_string()
    }

    #[allow(dead_code)]
    fn extract_project_name(&self, url: &str) -> String {
        let url_path = url.trim_end_matches('/');

        if let Some(filename) = url_path.split('/').last() {
            if filename.ends_with(".tar.gz") {
                return filename.trim_end_matches(".tar.gz").to_string();
            }
            if filename.ends_with(".tgz") {
                return filename.trim_end_matches(".tgz").to_string();
            }
            return filename.to_string();
        }

        "remote-project".to_string()
    }

    fn format_bytes_simple(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
        const THRESHOLD: f64 = 1024.0;

        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

impl Default for RemoteAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

fn analyze_file_content(
    file_path: &str,
    content: &str,
    language: &str,
    file_size: u64,
) -> Result<FileMetrics> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    let lang_def = LanguageRegistry::get_language(language);
    let empty_line_comments = vec![];
    let empty_multi_line_comments = vec![];
    let line_comments = lang_def
        .map(|l| &l.line_comments)
        .unwrap_or(&empty_line_comments);
    let multi_line_comments = lang_def
        .map(|l| &l.multi_line_comments)
        .unwrap_or(&empty_multi_line_comments);

    let mut in_multi_line_comment = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        let mut is_comment = false;

        if !in_multi_line_comment {
            for comment_start in line_comments {
                if trimmed.starts_with(comment_start) {
                    is_comment = true;
                    break;
                }
            }

            for (start, end) in multi_line_comments {
                if trimmed.starts_with(start) {
                    is_comment = true;
                    if !trimmed.ends_with(end) {
                        in_multi_line_comment = true;
                    }
                    break;
                }
            }
        } else {
            is_comment = true;
            for (_, end) in multi_line_comments {
                if trimmed.ends_with(end) {
                    in_multi_line_comment = false;
                    break;
                }
            }
        }

        if is_comment {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    let metrics = FileMetrics::new(
        file_path,
        language.to_string(),
        total_lines,
        code_lines,
        comment_lines,
        blank_lines,
    )?
    .with_size_bytes(file_size);

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_url_parsing() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer.parse_github_url("https://github.com/user/repo", "main"),
            Some(
                "https://github.com/user/repo/archive/refs/heads/main.tar.gz"
                    .to_string()
            )
        );

        assert_eq!(
            analyzer.parse_github_url(
                "https://github.com/user/repo/commit/abc123",
                "main"
            ),
            Some(
                "https://github.com/user/repo/archive/abc123.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_bitbucket_url_parsing() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer
                .parse_bitbucket_url("https://bitbucket.org/user/repo", "main"),
            Some("https://bitbucket.org/user/repo/get/main.tar.gz".to_string())
        );
    }

    #[test]
    fn test_codeberg_url_parsing() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer
                .parse_codeberg_url("https://codeberg.org/user/repo", "main"),
            Some(
                "https://codeberg.org/user/repo/archive/main.tar.gz"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_extract_project_name() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer.extract_project_name("https://example.com/project.tar.gz"),
            "project"
        );

        assert_eq!(
            analyzer.extract_project_name(
                "https://github.com/user/repo/archive/main.tar.gz"
            ),
            "main"
        );
    }
}

use tokio::sync::mpsc;

struct StreamReader {
    receiver: mpsc::Receiver<std::io::Result<bytes::Bytes>>,
    current_chunk: Option<Cursor<bytes::Bytes>>,
    finished: bool,
}

impl StreamReader {
    #[cfg(not(target_arch = "wasm32"))]
    fn new(
        stream: impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>>
        + Send
        + 'static,
        #[cfg(feature = "cli")] progress_bar: Option<ProgressBar>,
        total_size: Option<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut downloaded = 0u64;
            let mut stream = Box::pin(stream);

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        downloaded += chunk.len() as u64;

                        #[cfg(feature = "cli")]
                        if let Some(pb) = &progress_bar {
                            if let Some(_total) = total_size {
                                pb.set_position(downloaded);
                            } else {
                                let formatted =
                                    RemoteAnalyzer::format_bytes_simple(
                                        downloaded,
                                    );
                                pb.set_message(format!(
                                    "Downloaded {formatted}..."
                                ));
                            }
                        }

                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(std::io::Error::other(format!(
                            "Stream error: {e}"
                        ))));
                        break;
                    }
                }
            }
        });

        Self {
            receiver: rx,
            current_chunk: None,
            finished: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn new(
        stream: impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>>
        + 'static,
        #[cfg(feature = "cli")] _progress_bar: Option<ProgressBar>,
        _total_size: Option<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(32);

        wasm_bindgen_futures::spawn_local(async move {
            let mut downloaded = 0u64;
            let mut stream = Box::pin(stream);

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        downloaded += chunk.len() as u64;

                        #[cfg(feature = "cli")]
                        if let Some(pb) = &_progress_bar {
                            if let Some(_total) = _total_size {
                                pb.set_position(downloaded);
                            } else {
                                let formatted =
                                    RemoteAnalyzer::format_bytes_simple(
                                        downloaded,
                                    );
                                pb.set_message(format!(
                                    "Downloaded {}...",
                                    formatted
                                ));
                            }
                        }

                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Stream error: {}", e),
                            )))
                            .await;
                        break;
                    }
                }
            }
        });

        Self {
            receiver: rx,
            current_chunk: None,
            finished: false,
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut cursor) = self.current_chunk {
            let read = cursor.read(buf)?;
            if read > 0 {
                return Ok(read);
            }
            self.current_chunk = None;
        }

        if self.finished {
            return Ok(0);
        }

        match self.receiver.try_recv() {
            Ok(Ok(chunk)) => {
                self.current_chunk = Some(Cursor::new(chunk));
                if let Some(ref mut cursor) = self.current_chunk {
                    cursor.read(buf)
                } else {
                    Ok(0)
                }
            }
            Ok(Err(e)) => {
                self.finished = true;
                Err(e)
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    match self.receiver.blocking_recv() {
                        Some(Ok(chunk)) => {
                            self.current_chunk = Some(Cursor::new(chunk));
                            if let Some(ref mut cursor) = self.current_chunk {
                                cursor.read(buf)
                            } else {
                                Ok(0)
                            }
                        }
                        Some(Err(e)) => {
                            self.finished = true;
                            Err(e)
                        }
                        None => {
                            self.finished = true;
                            Ok(0)
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "Would block in WASM",
                    ))
                }
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.finished = true;
                Ok(0)
            }
        }
    }
}
