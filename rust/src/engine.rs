use crate::config::AppConfig;

pub struct AdEngine {
    config: AppConfig,
}

impl AdEngine {
    pub fn new() -> Self { AdEngine { config: AppConfig::default() } }
    pub fn update_config(&mut self, c: AppConfig) { self.config = c; }
}
