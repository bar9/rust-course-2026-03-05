// Chapter 11: Iterators Exercises Solutions

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogEntry {
    pub fn parse(line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 3 {
            return None;
        }

        let timestamp = parts[0].parse().ok()?;
        let level = match parts[1] {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            _ => return None,
        };

        Some(LogEntry {
            timestamp,
            level,
            message: parts[2].to_string(),
        })
    }

    pub fn new(timestamp: u64, level: LogLevel, message: String) -> Self {
        LogEntry {
            timestamp,
            level,
            message,
        }
    }
}

pub struct LogAnalyzer<'a> {
    lines: &'a [String],
}

impl<'a> LogAnalyzer<'a> {
    pub fn new(lines: &'a [String]) -> Self {
        LogAnalyzer { lines }
    }

    pub fn parse_entries(&self) -> impl Iterator<Item = LogEntry> + '_ {
        self.lines.iter()
            .filter_map(|line| LogEntry::parse(line))
    }

    pub fn errors_only(&self) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(|entry| entry.level == LogLevel::Error)
    }

    pub fn in_time_range(&self, start: u64, end: u64) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(move |entry| entry.timestamp >= start && entry.timestamp <= end)
    }

    pub fn count_by_level(&self) -> HashMap<LogLevel, usize> {
        self.parse_entries()
            .fold(HashMap::new(), |mut acc, entry| {
                *acc.entry(entry.level).or_insert(0) += 1;
                acc
            })
    }

    pub fn most_recent(&self, n: usize) -> Vec<LogEntry> {
        let mut entries: Vec<_> = self.parse_entries().collect();
        entries.sort_by_key(|entry| entry.timestamp);
        entries.into_iter().rev().take(n).collect()
    }

    // Additional helper methods for testing
    pub fn total_entries(&self) -> usize {
        self.parse_entries().count()
    }

    pub fn filter_by_level(&self, level: LogLevel) -> impl Iterator<Item = LogEntry> + '_ {
        self.parse_entries()
            .filter(move |entry| entry.level == level)
    }

    pub fn messages_containing(&self, substring: &str) -> impl Iterator<Item = LogEntry> + '_ {
        let substring = substring.to_string();
        self.parse_entries()
            .filter(move |entry| entry.message.contains(&substring))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_logs() -> Vec<String> {
        vec![
            "1000|INFO|Server started".to_string(),
            "1001|DEBUG|Connection received".to_string(),
            "1002|ERROR|Failed to connect to database".to_string(),
            "invalid line".to_string(),
            "1003|WARNING|High memory usage".to_string(),
            "1004|INFO|Request processed".to_string(),
            "1005|ERROR|Timeout error".to_string(),
        ]
    }

    #[test]
    fn test_log_entry_parse_valid() {
        let valid_line = "1000|INFO|Server started";
        let entry = LogEntry::parse(valid_line).unwrap();

        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Server started");
    }

    #[test]
    fn test_log_entry_parse_invalid() {
        let invalid_lines = vec![
            "invalid line",
            "1000|INVALID_LEVEL|Message",
            "not_a_timestamp|INFO|Message",
            "1000|INFO", // Missing message
        ];

        for line in invalid_lines {
            assert!(LogEntry::parse(line).is_none());
        }
    }

    #[test]
    fn test_log_entry_parse_all_levels() {
        let test_cases = vec![
            ("1000|DEBUG|Debug message", LogLevel::Debug),
            ("1001|INFO|Info message", LogLevel::Info),
            ("1002|WARNING|Warning message", LogLevel::Warning),
            ("1003|ERROR|Error message", LogLevel::Error),
        ];

        for (line, expected_level) in test_cases {
            let entry = LogEntry::parse(line).unwrap();
            assert_eq!(entry.level, expected_level);
        }
    }

    #[test]
    fn test_log_analyzer_parse_entries() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let entries: Vec<_> = analyzer.parse_entries().collect();
        assert_eq!(entries.len(), 6); // 7 lines, 1 invalid

        assert_eq!(entries[0].timestamp, 1000);
        assert_eq!(entries[0].level, LogLevel::Info);
        assert_eq!(entries[1].level, LogLevel::Debug);
    }

    #[test]
    fn test_log_analyzer_total_entries() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        assert_eq!(analyzer.total_entries(), 6);
    }

    #[test]
    fn test_log_analyzer_errors_only() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let errors: Vec<_> = analyzer.errors_only().collect();
        assert_eq!(errors.len(), 2);

        assert_eq!(errors[0].timestamp, 1002);
        assert_eq!(errors[0].message, "Failed to connect to database");
        assert_eq!(errors[1].timestamp, 1005);
        assert_eq!(errors[1].message, "Timeout error");
    }

    #[test]
    fn test_log_analyzer_time_range() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let range_entries: Vec<_> = analyzer.in_time_range(1002, 1004).collect();
        assert_eq!(range_entries.len(), 3);

        assert_eq!(range_entries[0].timestamp, 1002);
        assert_eq!(range_entries[1].timestamp, 1003);
        assert_eq!(range_entries[2].timestamp, 1004);
    }

    #[test]
    fn test_log_analyzer_count_by_level() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let counts = analyzer.count_by_level();

        assert_eq!(counts.get(&LogLevel::Info), Some(&2));
        assert_eq!(counts.get(&LogLevel::Debug), Some(&1));
        assert_eq!(counts.get(&LogLevel::Warning), Some(&1));
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
    }

    #[test]
    fn test_log_analyzer_most_recent() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let recent = analyzer.most_recent(3);
        assert_eq!(recent.len(), 3);

        // Should be in reverse chronological order (most recent first)
        assert_eq!(recent[0].timestamp, 1005);
        assert_eq!(recent[1].timestamp, 1004);
        assert_eq!(recent[2].timestamp, 1003);
    }

    #[test]
    fn test_log_analyzer_most_recent_more_than_available() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let recent = analyzer.most_recent(10);
        assert_eq!(recent.len(), 6); // Should return all available entries
    }

    #[test]
    fn test_log_analyzer_filter_by_level() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let info_entries: Vec<_> = analyzer.filter_by_level(LogLevel::Info).collect();
        assert_eq!(info_entries.len(), 2);

        let warning_entries: Vec<_> = analyzer.filter_by_level(LogLevel::Warning).collect();
        assert_eq!(warning_entries.len(), 1);
        assert_eq!(warning_entries[0].message, "High memory usage");
    }

    #[test]
    fn test_log_analyzer_messages_containing() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        let connection_entries: Vec<_> = analyzer.messages_containing("connect").collect();
        assert_eq!(connection_entries.len(), 1); // "Failed to connect to database" (case sensitive)

        let server_entries: Vec<_> = analyzer.messages_containing("Server").collect();
        assert_eq!(server_entries.len(), 1);
        assert_eq!(server_entries[0].message, "Server started");

        let nonexistent_entries: Vec<_> = analyzer.messages_containing("nonexistent").collect();
        assert_eq!(nonexistent_entries.len(), 0);
    }

    #[test]
    fn test_log_analyzer_empty_logs() {
        let logs = vec![];
        let analyzer = LogAnalyzer::new(&logs);

        assert_eq!(analyzer.total_entries(), 0);
        assert_eq!(analyzer.errors_only().count(), 0);
        assert_eq!(analyzer.most_recent(5).len(), 0);
        assert!(analyzer.count_by_level().is_empty());
    }

    #[test]
    fn test_log_analyzer_all_invalid_logs() {
        let logs = vec![
            "invalid line 1".to_string(),
            "invalid line 2".to_string(),
            "also invalid".to_string(),
        ];
        let analyzer = LogAnalyzer::new(&logs);

        assert_eq!(analyzer.total_entries(), 0);
        assert_eq!(analyzer.errors_only().count(), 0);
        assert_eq!(analyzer.most_recent(5).len(), 0);
        assert!(analyzer.count_by_level().is_empty());
    }

    #[test]
    fn test_log_analyzer_chaining_operations() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        // Chain multiple operations: get errors in a time range
        let errors_in_range: Vec<_> = analyzer
            .in_time_range(1000, 1003)
            .filter(|entry| entry.level == LogLevel::Error)
            .collect();

        assert_eq!(errors_in_range.len(), 1);
        assert_eq!(errors_in_range[0].timestamp, 1002);
        assert_eq!(errors_in_range[0].message, "Failed to connect to database");
    }

    #[test]
    fn test_iterator_lazy_evaluation() {
        let logs = create_test_logs();
        let analyzer = LogAnalyzer::new(&logs);

        // Creating iterators should not consume the data
        let _errors_iter = analyzer.errors_only();
        let _time_range_iter = analyzer.in_time_range(1000, 1005);

        // We should still be able to use the analyzer
        assert_eq!(analyzer.total_entries(), 6);
    }

    #[test]
    fn test_log_level_equality() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Error);
        assert_ne!(LogLevel::Debug, LogLevel::Warning);
    }

    #[test]
    fn test_log_entry_equality() {
        let entry1 = LogEntry::new(1000, LogLevel::Info, "Test message".to_string());
        let entry2 = LogEntry::new(1000, LogLevel::Info, "Test message".to_string());
        let entry3 = LogEntry::new(1001, LogLevel::Info, "Test message".to_string());

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }
}