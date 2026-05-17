use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipLogEntry {
    pub timestamp: u64,
    pub package: String,
    pub keyword_matched: String,
    pub clicked: bool,
    pub result: String,
}

/// 内存中的日志缓冲区（最近 200 条）
const MAX_LOG_ENTRIES: usize = 200;

pub struct SkipLogger {
    entries: Vec<SkipLogEntry>,
}

impl SkipLogger {
    pub fn new() -> Self {
        SkipLogger {
            entries: Vec::new(),
        }
    }

    pub fn log(&mut self, package: &str, keyword: &str, clicked: bool, result: &str) {
        let entry = SkipLogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            package: package.to_string(),
            keyword_matched: keyword.to_string(),
            clicked,
            result: result.to_string(),
        };

        self.entries.push(entry);
        if self.entries.len() > MAX_LOG_ENTRIES {
            self.entries.remove(0);
        }
    }

    pub fn entries(&self) -> &[SkipLogEntry] {
        &self.entries
    }

    pub fn recent(&self, count: usize) -> &[SkipLogEntry] {
        let len = self.entries.len();
        if len <= count {
            &self.entries
        } else {
            &self.entries[len - count..]
        }
    }

    pub fn today_count(&self) -> u64 {
        let today_start = today_start_secs();
        self.entries
            .iter()
            .filter(|e| e.timestamp >= today_start && e.clicked)
            .count() as u64
    }
}

fn today_start_secs() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 按 UTC+8 计算（中国大陆时区）
    let offset = 8 * 3600;
    let local = now + offset;
    (local / 86400) * 86400 - offset
}
