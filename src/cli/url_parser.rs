pub fn expand_url(url: &str) -> String {
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

pub fn show_usage_examples() {
    println!("Error: URL argument is required");
    println!();
    println!("Usage: bradar <URL>");
    println!();
    println!("Examples:");
    println!("  # GitHub repositories");
    println!("  bradar user/repo                    # Default branch");
    println!("  bradar user/repo@master             # Specific branch");
    println!("  bradar user/repo@abc123             # Specific commit");
    println!("  bradar https://github.com/user/repo # Full GitHub URL");
    println!();
    println!("  # Other platforms");
    println!("  bradar https://gitlab.com/user/repo # GitLab");
    println!("  bradar https://bitbucket.org/user/repo # Bitbucket");
    println!("  bradar https://codeberg.org/user/repo # Codeberg");
    println!();
    println!("  # Output formats");
    println!("  bradar -f json user/repo            # JSON output");
    println!("  bradar -f csv user/repo             # CSV output");
    println!("  bradar -f xml user/repo             # XML output");
    println!();
    println!("  # Authentication & options");
    println!("  bradar --token ghp_xxx user/repo    # GitHub token");
    println!("  bradar --timeout 600 user/repo      # Custom timeout");
    println!("  bradar --quiet user/repo            # Minimal output");
    println!();
    println!("For more information try --help");
}
