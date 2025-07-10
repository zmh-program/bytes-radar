# Deployment Guide

## Cloudflare Workers

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/zmh-program/bytes-radar)

> [!TIP]
> The Free Tier of Cloudflare Workers has a **20s request timeout limit**. Analysis of large repositories may fail due to this limitation. Consider upgrading to Cloudflare Workers Pro or using alternative deployment methods for processing large repositories.

The server component is automatically built and pushed to the `cf-worker` branch whenever changes are made to the server code. You can deploy to Cloudflare Workers with one click using the button above. This will:

1. Fork the repository to your GitHub account (using the pre-built worker from cf-worker branch)
2. Connect it to your Cloudflare Workers account
3. Deploy the worker to your chosen environment

### Manual Deployment (Wrangler)

If you prefer to deploy manually:

1. Clone the cf-worker branch which contains the pre-built worker:

```bash
git clone -b cf-worker https://github.com/zmh-program/bytes-radar.git
cd bytes-radar/server
```

2. Install Wrangler CLI:

```bash
pnpm install -g wrangler
```

3. Authenticate with Cloudflare:

```bash
wrangler login
```

4. Deploy to staging environment:

```bash
wrangler deploy --env staging
```

5. Deploy to production:

```bash
wrangler deploy --env production
```

### Environment Configuration

The worker supports two environments:

- `staging`: For testing and development (bytes-radar-staging.workers.dev)
- `production`: For production use (bytes-radar-prod.workers.dev)

See `server/wrangler.toml` for environment-specific configurations.

## API Documentation

The Bytes Radar API provides code analysis capabilities through a simple HTTP interface.

### Base URL

```
https://bradar.zmh.me
```

### Endpoints

#### Analyze Repository

```http
GET /{repository_path}
```

Analyzes a repository and returns detailed statistics about its codebase.

##### Repository Path Formats

- GitHub repository: `owner/repo` or `owner/repo@branch`
- Full GitHub URL: `https://github.com/owner/repo`
- GitLab URL: `https://gitlab.com/owner/repo`
- Direct archive URL: `https://example.com/archive.tar.gz`

##### Query Parameters

- `ignore_hidden` (boolean, default: true) - Whether to ignore hidden files/directories
- `ignore_gitignore` (boolean, default: true) - Whether to respect .gitignore rules
- `max_file_size` (number, default: -1) - Maximum file size to analyze in bytes (-1 for no limit)

##### Example Request

```http
GET /zmh-program/bytes-radar
```

##### Example Response

```json
{
  "project_name": "bytes-radar@main",
  "summary": {
    "project_name": "bytes-radar@main",
    "total_files": 37,
    "total_lines": 10255,
    "total_code_lines": 8944,
    "total_comment_lines": 0,
    "total_blank_lines": 1311,
    "total_size_bytes": 303101,
    "language_count": 8,
    "primary_language": "Rust",
    "overall_complexity_ratio": 0.872,
    "overall_documentation_ratio": 0
  },
  "language_statistics": [
    {
      "language_name": "Rust",
      "file_count": 22,
      "total_lines": 3892,
      "code_lines": 3298,
      "comment_lines": 0,
      "blank_lines": 594,
      "total_size_bytes": 123310,
      "average_file_size": 176.91,
      "complexity_ratio": 0.847,
      "documentation_ratio": 0
    }
    // ... other languages ...
  ],
  "debug_info": {
    "timestamp": "2025-07-06T23:59:50.662Z",
    "wasm_initialized": true,
    "target_url": "zmh-program/bytes-radar",
    "options": {
      "ignore_hidden": true,
      "ignore_gitignore": true,
      "max_file_size": -1
    },
    "analysis_duration_ms": 669,
    "total_duration_ms": 669,
    "total_languages": 8,
    "total_files": 37
  }
}
```

##### Error Response

```json
{
  "error": "Error message",
  "error_type": "NetworkError | AnalysisError",
  "error_category": "URL_PARSING | NETWORK | BRANCH_ACCESS | UNKNOWN",
  "suggested_fix": "Human-readable suggestion to fix the error",
  "debug_info": {
    // Debug information about the error
  }
}
```

### Rate Limits and Timeouts

- Request timeout: 20~30 seconds (Free tier)
- Rate limits: Based on Cloudflare Workers limits

### Notes

- Large repositories may hit the 20-second timeout limit on the free tier
- For analyzing large repositories, consider using the CLI tool or upgrading to Cloudflare Workers Pro
- The service automatically tries common branch names (main, master, develop, dev) if not specified
