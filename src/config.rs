use config::Config;
use serde::Deserialize;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub path_templates: Vec<String>,
    pub pattern: String,
}

#[derive(Debug, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct MailConfig {
    pub smtp: SmtpConfig,
    pub sender: String,
    pub password: String,
    pub recipients: Vec<String>,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaceholderConfig {
    #[serde(flatten)]
    pub mapping: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub log: LogConfig,
    pub mail: MailConfig,
    pub placeholder: PlaceholderConfig,
}

impl Settings {
    pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
        let s = Config::builder()
            .add_source(config::File::from(Path::new(path)))
            .build()?;

        s.try_deserialize::<Settings>()
    }
}
