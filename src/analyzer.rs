use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use hdrhistogram::Histogram;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub path: String,
    pub status: u16,
    pub rt: f64,
}

#[derive(Debug)]
pub struct PathStats {
    pub total_requests: u64,
    pub status_2xx: u64,
    pub status_3xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
    pub total_rt: f64,
    pub max_rt: f64,
    pub min_rt: f64,
    pub rt_hist: Histogram<u64>,
}

impl PathStats {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            status_2xx: 0,
            status_3xx: 0,
            status_4xx: 0,
            status_5xx: 0,
            total_rt: 0.0,
            max_rt: 0.0,
            min_rt: f64::MAX,
            rt_hist: Histogram::new_with_bounds(1, 120_000, 3).unwrap(),
        }
    }

    pub fn add_entry(&mut self, entry: &LogEntry) {
        self.total_requests += 1;
        self.total_rt += entry.rt;
        if entry.rt > self.max_rt {
            self.max_rt = entry.rt;
        }
        if entry.rt < self.min_rt {
            self.min_rt = entry.rt;
        }
        match entry.status {
            200..=299 => self.status_2xx += 1,
            300..=399 => self.status_3xx += 1,
            400..=499 => self.status_4xx += 1,
            500..=599 => self.status_5xx += 1,
            _ => {}
        }
        let _ = self.rt_hist.record((entry.rt * 1000.0) as u64); // ms
    }

    pub fn avg_rt(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_rt / self.total_requests as f64
        } else {
            0.0
        }
    }

    pub fn p90(&self) -> f64 {
        self.rt_hist.value_at_quantile(0.90) as f64 / 1000.0
    }
    pub fn p95(&self) -> f64 {
        self.rt_hist.value_at_quantile(0.95) as f64 / 1000.0
    }
    pub fn p99(&self) -> f64 {
        self.rt_hist.value_at_quantile(0.99) as f64 / 1000.0
    }
}

pub struct LogAnalyzer {
    pattern: Regex,
    stats: HashMap<String, PathStats>,
}

impl LogAnalyzer {
    pub fn from_files(pattern: &str, file_paths: &[String]) -> Result<Self, io::Error> {
        let mut analyzer = Self::new(pattern)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        for file_path in file_paths {
            analyzer.load_file(file_path)?;
        }
        Ok(analyzer)
    }

    fn new(pattern: &str) -> Result<Self, regex::Error> {
        let pattern = Regex::new(pattern)?;
        Ok(Self {
            pattern,
            stats: HashMap::new(),
        })
    }


    fn load_file(&mut self, file_path: &str) -> io::Result<()> {
        if !Path::new(file_path).exists() {
            eprintln!("[warn]: the log file not exists: {}", file_path);
            return Ok(());
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            self.add_line(&line, Some(line_num + 1));
        }

        Ok(())
    }


    fn add_line(&mut self, line: &str, line_num: Option<usize>) -> bool {
        if let Some(entry) = self.parse_log_line(line) {
            self.stats.entry(entry.path.clone())
                .or_insert_with(PathStats::new)
                .add_entry(&entry);
            true
        } else {
            if let Some(num) = line_num {
                eprintln!("[warn] the line num: {} can not be parsed, line: {}", num, line);
            }
            false
        }
    }


    pub fn get_result(&self) -> &HashMap<String, PathStats> {
        &self.stats
    }


    pub fn total_requests(&self) -> u64 {
        self.stats.values().map(|s| s.total_requests).sum()
    }



    pub fn status_count(&self, status_range: &str) -> u64 {
        match status_range {
            "2xx" => self.stats.values().map(|s| s.status_2xx).sum(),
            "3xx" => self.stats.values().map(|s| s.status_3xx).sum(),
            "4xx" => self.stats.values().map(|s| s.status_4xx).sum(),
            "5xx" => self.stats.values().map(|s| s.status_5xx).sum(),
            _ => 0,
        }
    }

    fn parse_log_line(&self, line: &str) -> Option<LogEntry> {
        if let Some(captures) = self.pattern.captures(line) {
            let path = captures.name("path")
                .map(|m| self.extract_path_from_url(m.as_str()))
                .unwrap_or_default();
            
            let status: u16 = captures.name("status")
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            
            let rt: f64 = captures.name("rt")
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0.0);

            Some(LogEntry { path, status, rt })
        } else {
            None
        }
    }

    fn extract_path_from_url(&self, url: &str) -> String {
        if let Some(path) = url.split('?').next() {
            path.to_string()
        } else {
            url.to_string()
        }
    }



    pub fn get_analysis_results_detail_markdown_cn(&self) -> String {
        let stats = self.get_result();
        let mut md = String::new();
        md.push_str("| 路径 | 总请求 | 2xx | 3xx | 4xx | 5xx | 平均耗时 | 最大耗时 | 最小耗时 | p90耗时 | p95耗时 | p99耗时 |\n");
        md.push_str("|:---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");

        let mut sorted_stats: Vec<_> = stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.total_requests.cmp(&a.1.total_requests));

        for (path, stat) in sorted_stats {
            let path_disp = if path.len() > 48 { format!("{}...", &path[..45]) } else { path.to_string() };
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |\n",
                path_disp,
                stat.total_requests,
                stat.status_2xx,
                stat.status_3xx,
                stat.status_4xx,
                stat.status_5xx,
                stat.avg_rt(),
                stat.max_rt,
                if stat.min_rt == f64::MAX { 0.0 } else { stat.min_rt },
                stat.p90(),
                stat.p95(),
                stat.p99()
            ));
        }
        md
    }


    pub fn get_analysis_results_summary_markdown_cn(&self) -> String {
        let mut md = String::new();
        let total_requests = self.total_requests();
        let total_2xx = self.status_count("2xx");
        let total_3xx = self.status_count("3xx");
        let total_4xx = self.status_count("4xx");
        let total_5xx = self.status_count("5xx");
        md.push_str(&format!("- 总请求数: {}\n", total_requests));
        if total_requests > 0 {
            md.push_str(&format!("- 2xx 状态码: {} ({:.1}%)\n", total_2xx, (total_2xx as f64 / total_requests as f64) * 100.0));
            md.push_str(&format!("- 3xx 状态码: {} ({:.1}%)\n", total_3xx, (total_3xx as f64 / total_requests as f64) * 100.0));
            md.push_str(&format!("- 4xx 状态码: {} ({:.1}%)\n", total_4xx, (total_4xx as f64 / total_requests as f64) * 100.0));
            md.push_str(&format!("- 5xx 状态码: {} ({:.1}%)\n", total_5xx, (total_5xx as f64 / total_requests as f64) * 100.0));
        }
        md
    }
    
}

 