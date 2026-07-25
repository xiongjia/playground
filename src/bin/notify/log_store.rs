// ─── Local audit log store (JSON Lines) ───────────────────

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::message::Message;

/// A single log entry written to the audit trail.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub channel: String,
    pub level: String,
    pub title: String,
    pub body: String,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Local audit log — append-only JSON Lines file.
#[derive(Debug)]
pub struct LogStore {
    path: PathBuf,
}

impl LogStore {
    /// Open or create the log file at the given path.
    pub fn new(path: PathBuf) -> Self {
        LogStore { path }
    }

    /// Return the log file path.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Determine the default log path (project-local `modules/notify/notify.log`).
    ///
    /// Resolves the project root from the binary path (works from any CWD).
    pub fn default_path() -> PathBuf {
        if let Some(root) = crate::config::find_project_root() {
            return root.join("modules/notify/notify.log");
        }
        // Fallback: CWD-relative (for test environments)
        PathBuf::from("modules/notify/notify.log")
    }

    /// Append one log entry.
    pub fn write(
        &self,
        msg: &Message,
        channel: &str,
        success: bool,
        error: Option<String>,
        duration_ms: u64,
    ) -> std::io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let entry = LogEntry {
            timestamp: msg.timestamp.to_rfc3339(),
            channel: channel.to_string(),
            level: format!("{:?}", msg.level).to_lowercase(),
            title: msg.title.clone(),
            body: msg.body.clone(),
            success,
            error,
            duration_ms,
        };

        let line = serde_json::to_string(&entry)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// List recent entries, newest first.
    pub fn list(&self, limit: usize) -> std::io::Result<Vec<LogEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries: Vec<LogEntry> = reader
            .lines()
            .filter_map(|line| line.ok().and_then(|l| serde_json::from_str(&l).ok()))
            .collect();

        // Newest first
        entries.reverse();
        entries.truncate(limit);
        Ok(entries)
    }

    /// Filter entries by channel name.
    pub fn filter_channel(entries: Vec<LogEntry>, channel: &str) -> Vec<LogEntry> {
        entries.into_iter().filter(|e| e.channel == channel).collect()
    }

    /// Filter entries by success status.
    pub fn filter_status(entries: Vec<LogEntry>, success: bool) -> Vec<LogEntry> {
        entries.into_iter().filter(|e| e.success == success).collect()
    }

    /// Compute summary statistics from all entries.
    pub fn status(&self) -> std::io::Result<LogStatus> {
        if !self.path.exists() {
            return Ok(LogStatus::default());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut status = LogStatus::default();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                status.total += 1;
                if entry.success {
                    status.success += 1;
                } else {
                    status.failed += 1;
                }
                *status.by_channel.entry(entry.channel).or_insert(0) += 1;
            }
        }

        Ok(status)
    }
}

/// Summary statistics from the log.
#[derive(Debug, Default, serde::Serialize)]
pub struct LogStatus {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub by_channel: std::collections::HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Level;
    use chrono::Local;

    fn sample_msg(channel: &str) -> Message {
        Message {
            title: "Test".into(),
            body: "Hello".into(),
            level: Level::Info,
            timestamp: Local::now(),
            channel: channel.into(),
        }
    }

    #[test]
    fn test_write_and_list() {
        let dir = std::env::temp_dir().join("notify-test-write-list");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("notify.log");
        let store = LogStore::new(path.clone());

        store.write(&sample_msg("telegram"), "telegram", true, None, 100).unwrap();
        store.write(&sample_msg("stdout"), "stdout", true, None, 50).unwrap();
        store.write(&sample_msg("telegram"), "telegram", false, Some("fail".into()), 200).unwrap();

        let entries = store.list(10).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(!entries[0].success);  // newest first: the failed entry

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_filter_channel() {
        let all = vec![
            LogEntry { channel: "telegram".into(), ..sample_entry() },
            LogEntry { channel: "stdout".into(), ..sample_entry() },
        ];
        let filtered = LogStore::filter_channel(all, "telegram");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_status() {
        let dir = std::env::temp_dir().join("notify-test-status");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("notify.log");
        let store = LogStore::new(path.clone());

        store.write(&sample_msg("telegram"), "telegram", true, None, 100).unwrap();
        store.write(&sample_msg("telegram"), "telegram", false, Some("err".into()), 50).unwrap();

        let status = store.status().unwrap();
        assert_eq!(status.total, 2);
        assert_eq!(status.success, 1);
        assert_eq!(status.failed, 1);
        assert_eq!(*status.by_channel.get("telegram").unwrap(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    fn sample_entry() -> LogEntry {
        LogEntry {
            timestamp: Local::now().to_rfc3339(),
            channel: "test".into(),
            level: "info".into(),
            title: "".into(),
            body: "".into(),
            success: true,
            error: None,
            duration_ms: 0,
        }
    }
}
