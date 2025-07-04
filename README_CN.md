# nginx-log-analyzer

一个高性能、可配置的 Nginx 日志分析工具，支持多日志文件、灵活的占位符模板、Markdown/HTML邮件报告、响应时间分位数统计等特性。适合安全、性能、可用性等多场景的日志分析与自动化报告。

## 特性

- **多日志文件支持**：可同时分析多个 Nginx 日志文件，自动合并统计结果。
- **灵活的占位符系统**：支持自定义 UDF（用户自定义函数）和多种占位符格式，配置文件中可灵活引用。
- **Markdown/HTML 邮件报告**：分析结果以 Markdown 格式生成，并自动转换为 HTML，通过 SMTP 发送邮件。
- **响应时间分位数统计**：支持平均、最大、最小、P90、P95、P99 等多种响应时间统计。
- **配置驱动**：所有分析逻辑、报告内容、邮件内容均可通过配置文件灵活调整，无需改动代码。
- **高性能**：使用 Rust 编写，支持大规模日志文件的高效分析。

## 快速开始

### 1. 配置文件示例

`config/config.toml`：

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
检测时间：{{placeholder|:|get_time|:|%Y-%m-%d %H:%M:%S}}

## Nginx 日志分析结果

{{placeholder|:|simple_mapping|:|get_analysis_results_detail_markdown_cn}}

## 汇总统计

{{placeholder|:|simple_mapping|:|get_analysis_results_summary_markdown_cn}}
"""

[mail.smtp]
host = "smtp.example.com"

[placeholder]
title_prefix = "Nginx 日志分析报告"
```

### 2. 编译与运行

```bash
cargo build --release
./target/release/nginx-log-analyzer --config config/config.toml
```

### 3. 主要用法

- 支持通过 `--config` 参数指定配置文件路径。
- 日志文件路径支持占位符和日期动态生成。
- 分析结果自动通过邮件发送，内容和格式可完全自定义。

### 4. 占位符与 UDF

- `{{placeholder|:|get_time|:|%Y%m%d}}`：插入当前日期
- `{{placeholder|:|simple_mapping|:|key}}`：插入配置中 `placeholder` 部分的映射值
- 可扩展自定义 UDF

### 5. 统计指标

- 每个 URL 路径的请求总数、2xx/3xx/4xx/5xx 状态码数
- 平均、最大、最小响应时间
- P90、P95、P99 响应时间分位数

### 6. 依赖

- Rust 2021+
- [lettre](https://crates.io/crates/lettre)（邮件发送）
- [regex](https://crates.io/crates/regex)
- [hdrhistogram](https://crates.io/crates/hdrhistogram)
- [serde](https://crates.io/crates/serde) + [toml](https://crates.io/crates/toml)

### 7. 典型输出

邮件内容示例（Markdown）：

```
检测时间：2025-07-03 03:18:23

## Nginx 日志分析结果

| 路径 | 总请求 | 2xx | 3xx | 4xx | 5xx | 平均耗时 | 最大耗时 | 最小耗时 | p90耗时 | p95耗时 | p99耗时 |
|:---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| /api/v1/search/reputation/fileReputation | 123 | 120 | 1 | 2 | 0 | 0.123 | 1.234 | 0.001 | 0.200 | 0.300 | 0.400 |

## 汇总统计

- 总请求数: 123
- 2xx 状态码: 120 (97.6%)
- 3xx 状态码: 1 (0.8%)
- 4xx 状态码: 2 (1.6%)
- 5xx 状态码: 0 (0.0%)
```

---

## 贡献与反馈

欢迎提交 issue 或 PR，提出你的建议和需求！
