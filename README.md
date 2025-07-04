# nginx-log-analyzer

[中文说明 (Chinese README)](README_CN.md)

A high-performance, configurable Nginx log analyzer supporting multiple log files, flexible placeholder templates, Markdown/HTML email reports, response time percentiles, and more. Suitable for security, performance, availability, and automated reporting scenarios.

## Features

- **Multiple Log File Support**: Analyze multiple Nginx log files at once and merge statistics automatically.
- **Flexible Placeholder System**: Supports custom UDFs (user-defined functions) and various placeholder formats, easily referenced in the config file.
- **Markdown/HTML Email Reports**: Generates analysis results in Markdown, auto-converts to HTML, and sends via SMTP.
- **Response Time Percentiles**: Calculates average, max, min, P90, P95, P99 response times.
- **Config-Driven**: All analysis logic, report content, and email content are fully configurable—no code changes required.
- **High Performance**: Written in Rust, efficient for large-scale log analysis.

## Quick Start

### 1. Example Config File

`config/config.toml`:

```toml
[log]
path_templates = [
    "/tmp/nginx-access-{{placeholder|:|get_time|:|%y%m%d|:|-86400000}}.log",
    "/tmp/nginx-access-{{placeholder|:|get_time|:|%y%m%d}}.log"
]
pattern = '^(\S+) - - \[([^\]]+)\] "([A-Z]+) (?P<path>[^ ?"]+)[^"]*" (?P<status>\d{3}) (\d+) "([^"]*)" "([^"]*)" "([^"]*)" *rt=(?P<rt>\d+\.\d+) uct=(?:-|\d+\.\d+) urt=(?:-|\d+\.\d+)$'

[mail]
recipients = ["your@email.com"]
sender = "your_sender@email.com"
password = "your_password"
title = "{{placeholder|:|simple_mapping|:|title_prefix}}-{{placeholder|:|get_time|:|%Y%m%d}}"
content = """
Detection Time: {{placeholder|:|get_time|:|%Y-%m-%d %H:%M:%S}}

## Nginx Log Analysis Results

{{placeholder|:|simple_mapping|:|get_analysis_results_detail_markdown_cn}}

## Summary Statistics

{{placeholder|:|simple_mapping|:|get_analysis_results_summary_markdown_cn}}
"""

[mail.smtp]
host = "smtp.example.com"

[placeholder]
title_prefix = "Nginx Log Analysis Report"
```

### 2. Build & Run

```bash
cargo build --release
./target/release/nginx-log-analyzer --config config/config.toml
```

### 3. Usage

- Use the `--config` argument to specify the config file path.
- Log file paths support placeholders and dynamic date generation.
- Analysis results are sent via email; content and format are fully customizable.

### 4. Placeholders & UDFs

- `{{placeholder|:|get_time|:|%Y%m%d}}`: Insert current date
- `{{placeholder|:|simple_mapping|:|key}}`: Insert value from the `[placeholder]` section in config
- Custom UDFs are extensible

### 5. Statistics

- Total requests, 2xx/3xx/4xx/5xx status counts per URL path
- Average, max, min response time
- P90, P95, P99 response time percentiles

### 6. Dependencies

- Rust 2021+
- [lettre](https://crates.io/crates/lettre) (email sending)
- [regex](https://crates.io/crates/regex)
- [hdrhistogram](https://crates.io/crates/hdrhistogram)
- [serde](https://crates.io/crates/serde) + [toml](https://crates.io/crates/toml)

### 7. Example Output

Sample email content (Markdown):

```
Detection Time: 2025-07-03 03:18:23

## Nginx Log Analysis Results

| Path | Total | 2xx | 3xx | 4xx | 5xx | Avg RT | Max RT | Min RT | P90 RT | P95 RT | P99 RT |
|:---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| /api/v1/search/reputation/fileReputation | 123 | 120 | 1 | 2 | 0 | 0.123 | 1.234 | 0.001 | 0.200 | 0.300 | 0.400 |

## Summary Statistics

- Total requests: 123
- 2xx: 120 (97.6%)
- 3xx: 1 (0.8%)
- 4xx: 2 (1.6%)
- 5xx: 0 (0.0%)
```

---

## Contributing & Feedback

Feel free to submit issues or PRs with your suggestions and needs!

