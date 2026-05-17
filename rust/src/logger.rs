use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipLogEntry {
    pub timestamp: u64,
    pub package: String,
    pub keyword_matched: String,
    pub clicked: bool,
    pub result: String,
}

pub struct SkipLogger { entries: Vec<SkipLogEntry> }

impl SkipLogger {
    pub fn new() -> Self { SkipLogger { entries: vec![] } }
    pub fn log(&mut self, pkg: &str, kw: &str, clicked: bool, result: &str) {
        use std::time::{SystemTime, UNIX_EPOCH};
        self.entries.push(SkipLogEntry {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            package: pkg.into(), keyword_matched: kw.into(), clicked, result: result.into()
        });
    }
    pub fn entries(&self) -> &[SkipLogEntry] { &self.entries }
    pub fn recent(&self, n: usize) -> &[SkipLogEntry] {
        let len = self.entries.len();
        if len <= n { &self.entries } else { &self.entries[len-n..] }
    }
    pub fn today_count(&self) -> u64 { self.entries.iter().filter(|e| e.clicked).count() as u64 }
}
