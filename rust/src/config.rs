use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled: bool,
    pub keywords: Vec<String>,
    pub auto_click_delay_ms: u64,
    pub app_whitelist: Vec<String>,
    pub app_blacklist: Vec<String>,
    pub log_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig { enabled: true, keywords: vec![], auto_click_delay_ms: 0,
            app_whitelist: vec![], app_blacklist: vec![], log_enabled: true }
    }
}
