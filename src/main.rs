mod dateutil;
mod config;
mod analyzer;
mod placeholder;
use clap::Parser;
use config::Settings;
use analyzer::LogAnalyzer;
use placeholder::{replace_placeholders, init_global, add_global_mapping};
mod mail_util;


#[derive(Parser, Debug)]
#[command(name = "nginx-log-analyzer", about = "Analyze nginx log files")]
struct Args {
    #[arg(short, long)]
    config: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let settings = Settings::from_file(&args.config).expect("failed to read config");
    
    // 初始化全局占位符映射
    init_global(settings.placeholder.mapping);
    
    // 处理多个文件路径模板
    let mut resolved_paths = Vec::new();
    for path_template in &settings.log.path_templates {
        let resolved_path = replace_placeholders(path_template);
        resolved_paths.push(resolved_path);
    }

    let analyzer: LogAnalyzer = LogAnalyzer::from_files(&settings.log.pattern, &resolved_paths)?;
    

    add_global_mapping("get_analysis_results_detail_markdown_cn".to_string(),analyzer.get_analysis_results_detail_markdown_cn());
    add_global_mapping("get_analysis_results_summary_markdown_cn".to_string(),analyzer.get_analysis_results_summary_markdown_cn());
    
    let analysis_results = replace_placeholders(&settings.mail.content);

    let mail = &settings.mail;


    let title = replace_placeholders(&settings.mail.title); 
    // 发送邮件
    mail_util::send_mail(
        &mail.smtp.host,
        &mail.sender,
        &mail.password,
        &mail.recipients,
        &title,
        analysis_results
    )?;

    Ok(())
}
