// Chapter 10: Error Handling Exercise Solution

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

// =============================================================================
// Exercise: Build a Configuration Parser
// =============================================================================

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

// impl PartialEq for ConfigError {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (ConfigError::IoError(e1), ConfigError::IoError(e2)) => {
//                 e1.kind() == e2.kind() && e1.to_string() == e2.to_string()
//             },
//             (ConfigError::ParseError(msg1), ConfigError::ParseError(msg2)) => msg1 == msg2,
//             (ConfigError::ValidationError(msg1), ConfigError::ValidationError(msg2)) => msg1 == msg2,
//             _ => false,
//         }
//     }
// }

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// Automatic conversion from io::Error to ConfigError
impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

#[derive(Debug, PartialEq)]
pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
        }
    }

    pub fn from_string(contents: &str) -> Result<Self, ConfigError> {
        let mut config = Config::new();

        // Parse each line
        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value pairs
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ParseError(format!(
                    "Invalid format on line {}: '{}'",
                    line_num + 1,
                    line
                )));
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            if key.is_empty() {
                return Err(ConfigError::ParseError(format!(
                    "Empty key on line {}",
                    line_num + 1
                )));
            }

            config.settings.insert(key.to_string(), value.to_string());
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        // Read file using the ? operator for automatic error conversion
        let contents = fs::read_to_string(path)?;
        Self::from_string(&contents)
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_required(&self, key: &str) -> Result<&String, ConfigError> {
        self.settings.get(key).ok_or_else(|| {
            ConfigError::ValidationError(format!("Required key '{}' not found", key))
        })
    }

    pub fn get_int(&self, key: &str) -> Result<i32, ConfigError> {
        let value = self.get_required(key)?;
        value.parse::<i32>().map_err(|_| {
            ConfigError::ParseError(format!(
                "Value '{}' for key '{}' is not a valid integer",
                value, key
            ))
        })
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self.get_required(key)?;
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ConfigError::ParseError(format!(
                "Value '{}' for key '{}' is not a valid boolean",
                value, key
            ))),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.settings.keys()
    }

    pub fn len(&self) -> usize {
        self.settings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.settings.is_empty()
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // Validate port ranges if port is specified
        if let Some(port_str) = self.get("port") {
            if let Ok(port) = port_str.parse::<u16>() {
                if port == 0 {
                    return Err(ConfigError::ValidationError("Port cannot be 0".to_string()));
                }
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Load configuration with error context
pub fn load_config_with_context<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
    Config::from_file(&path).map_err(|e| match e {
        ConfigError::IoError(_) => ConfigError::ValidationError(format!(
            "Failed to load config from '{}'",
            path.as_ref().display()
        )),
        other => other,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.is_empty());
        assert_eq!(config.len(), 0);
    }

    #[test]
    fn test_valid_config_string() {
        let content = r#"
# This is a comment
app_name=MyApp
port=8080
debug=true

# Another comment
host=localhost
"#;
        let config = Config::from_string(content).unwrap();

        assert_eq!(config.get("app_name"), Some(&"MyApp".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("debug"), Some(&"true".to_string()));
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.len(), 4);
    }

    #[test]
    fn test_get_required() {
        let content = "app_name=TestApp\nport=3000";
        let config = Config::from_string(content).unwrap();

        assert_eq!(config.get_required("app_name").unwrap(), "TestApp");
        assert!(config.get_required("nonexistent").is_err());
    }

    #[test]
    fn test_get_int() {
        let content = "port=8080\ninvalid_port=abc";
        let config = Config::from_string(content).unwrap();

        assert_eq!(config.get_int("port").unwrap(), 8080);

        let result = config.get_int("invalid_port");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }

    #[test]
    fn test_get_bool() {
        let content = r#"
debug=true
verbose=false
enabled=yes
disabled=no
active=1
inactive=0
invalid=maybe
"#;
        let config = Config::from_string(content).unwrap();

        assert_eq!(config.get_bool("debug").unwrap(), true);
        assert_eq!(config.get_bool("verbose").unwrap(), false);
        assert_eq!(config.get_bool("enabled").unwrap(), true);
        assert_eq!(config.get_bool("disabled").unwrap(), false);
        assert_eq!(config.get_bool("active").unwrap(), true);
        assert_eq!(config.get_bool("inactive").unwrap(), false);

        let result = config.get_bool("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }

    #[test]
    fn test_file_not_found() {
        let result = Config::from_file("nonexistent_file.conf");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::IoError(_)));
    }

    #[test]
    fn test_invalid_format() {
        let content = r#"
app_name=TestApp
invalid_line_without_equals
port=8080
"#;
        let result = Config::from_string(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }

    #[test]
    fn test_empty_key() {
        let content = "app_name=TestApp\n=value_without_key";
        let result = Config::from_string(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }

    #[test]
    fn test_validation_error() {
        let content = "port=0"; // Invalid port
        let result = Config::from_string(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::ValidationError(_)
        ));
    }

    #[test]
    fn test_comments_and_empty_lines() {
        let content = r#"
# Configuration file
# This is a comment

app_name=TestApp

# Server configuration
port=8080

# Debug settings
debug=true
"#;
        let config = Config::from_string(content).unwrap();

        assert_eq!(config.len(), 3); // Only non-comment, non-empty lines
        assert_eq!(config.get("app_name"), Some(&"TestApp".to_string()));
    }

    #[test]
    fn test_config_modification() {
        let mut config = Config::new();
        config.set("test_key".to_string(), "test_value".to_string());

        assert_eq!(config.get("test_key"), Some(&"test_value".to_string()));
        assert_eq!(config.len(), 1);
        assert!(!config.is_empty());
    }

    #[test]
    fn test_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_error = ConfigError::IoError(io_error);

        let display = format!("{}", config_error);
        assert!(display.contains("IO error"));

        let parse_error = ConfigError::ParseError("invalid format".to_string());
        let display = format!("{}", parse_error);
        assert!(display.contains("Parse error"));

        let validation_error = ConfigError::ValidationError("missing key".to_string());
        let display = format!("{}", validation_error);
        assert!(display.contains("Validation error"));
    }

    #[test]
    fn test_error_source() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_error = ConfigError::IoError(io_error);

        assert!(config_error.source().is_some());

        let parse_error = ConfigError::ParseError("invalid".to_string());
        assert!(parse_error.source().is_none());
    }

    #[test]
    fn test_from_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let config_error: ConfigError = io_error.into();

        assert!(matches!(config_error, ConfigError::IoError(_)));
    }

    #[test]
    fn test_config_keys_iteration() {
        let content = "app_name=TestApp\nport=8080\ndebug=true";
        let config = Config::from_string(content).unwrap();

        let keys: Vec<&String> = config.keys().collect();
        assert_eq!(keys.len(), 3);

        let key_strs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        assert!(key_strs.contains(&"app_name"));
        assert!(key_strs.contains(&"port"));
        assert!(key_strs.contains(&"debug"));
    }

    #[test]
    fn test_load_config_with_context() {
        let result = load_config_with_context("nonexistent.conf");
        assert!(result.is_err());

        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("Failed to load config from"));
        } else {
            panic!("Expected ValidationError with context");
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test with spaces around equals
        let content = "app_name = TestApp\n port = 8080 ";
        let config = Config::from_string(content).unwrap();

        assert_eq!(config.get("app_name"), Some(&"TestApp".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_empty_config() {
        let content = "# Only comments\n\n# No actual config";
        let config = Config::from_string(content).unwrap();
        assert!(config.is_empty());
    }

    #[test]
    fn test_error_propagation() {
        // Test that ? operator works correctly
        fn parse_and_get_port(content: &str) -> Result<i32, ConfigError> {
            let config = Config::from_string(content)?;
            config.get_int("port")
        }

        let valid_content = "port=8080";
        assert_eq!(parse_and_get_port(valid_content).unwrap(), 8080);

        let invalid_content = "port=invalid";
        assert!(parse_and_get_port(invalid_content).is_err());

        let malformed_content = "invalid_line";
        assert!(parse_and_get_port(malformed_content).is_err());
    }

    #[test]
    fn test_multiple_error_types() {
        // Test different error types in sequence
        let mut errors = Vec::new();

        // Parse error
        if let Err(e) = Config::from_string("invalid_line") {
            errors.push(e);
        }

        // Validation error
        if let Err(e) = Config::from_string("port=0") {
            errors.push(e);
        }

        // IO error
        if let Err(e) = Config::from_file("nonexistent.conf") {
            errors.push(e);
        }

        assert_eq!(errors.len(), 3);
        assert!(matches!(errors[0], ConfigError::ParseError(_)));
        assert!(matches!(errors[1], ConfigError::ValidationError(_)));
        assert!(matches!(errors[2], ConfigError::IoError(_)));
    }
}
