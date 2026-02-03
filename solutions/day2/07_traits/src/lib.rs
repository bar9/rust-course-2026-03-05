// Chapter 7: Traits Exercise Solution

// =============================================================================
// Exercise: Trait Objects with Multiple Behaviors (Plugin System)
// =============================================================================

pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self);
}

pub trait Configurable {
    fn configure(&mut self, config: &str);
}

// Create different plugin types
pub struct LogPlugin {
    name: String,
    level: String,
}

impl LogPlugin {
    pub fn new(name: String) -> Self {
        LogPlugin {
            name,
            level: "info".to_string()
        }
    }

    pub fn level(&self) -> &str {
        &self.level
    }
}

impl Plugin for LogPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self) {
        println!("Logging at {} level", self.level);
    }
}

impl Configurable for LogPlugin {
    fn configure(&mut self, config: &str) {
        // Parse config and update log level
        if config.contains("level=") {
            let level = config.split("level=").nth(1).unwrap_or("info").trim();
            self.level = level.to_string();
        }
    }
}

pub struct MetricsPlugin {
    name: String,
    interval: u32,
}

impl MetricsPlugin {
    pub fn new(name: String, interval: u32) -> Self {
        MetricsPlugin { name, interval }
    }

    pub fn interval(&self) -> u32 {
        self.interval
    }
}

impl Plugin for MetricsPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self) {
        println!("Collecting metrics every {} seconds", self.interval);
    }
}

impl Configurable for MetricsPlugin {
    fn configure(&mut self, config: &str) {
        // Parse config and update interval
        if config.contains("interval=") {
            let interval_str = config.split("interval=").nth(1).unwrap_or("60").trim();
            if let Ok(interval) = interval_str.parse::<u32>() {
                self.interval = interval;
            }
        }
    }
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn run_all(&self) {
        for plugin in &self.plugins {
            plugin.execute();
        }
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn get_plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_plugin_creation() {
        let log_plugin = LogPlugin::new("MyLogger".to_string());
        assert_eq!(log_plugin.name(), "MyLogger");
        assert_eq!(log_plugin.level(), "info");
    }

    #[test]
    fn test_log_plugin_configuration() {
        let mut log_plugin = LogPlugin::new("Logger".to_string());
        log_plugin.configure("level=debug");
        assert_eq!(log_plugin.level(), "debug");

        log_plugin.configure("level=error");
        assert_eq!(log_plugin.level(), "error");
    }

    #[test]
    fn test_metrics_plugin_creation() {
        let metrics_plugin = MetricsPlugin::new("MyMetrics".to_string(), 30);
        assert_eq!(metrics_plugin.name(), "MyMetrics");
        assert_eq!(metrics_plugin.interval(), 30);
    }

    #[test]
    fn test_metrics_plugin_configuration() {
        let mut metrics_plugin = MetricsPlugin::new("Metrics".to_string(), 60);
        metrics_plugin.configure("interval=120");
        assert_eq!(metrics_plugin.interval(), 120);

        // Invalid config should not change interval
        metrics_plugin.configure("interval=invalid");
        assert_eq!(metrics_plugin.interval(), 120);
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.plugin_count(), 0);

        let log_plugin = LogPlugin::new("Logger".to_string());
        let metrics_plugin = MetricsPlugin::new("Metrics".to_string(), 60);

        manager.register(Box::new(log_plugin));
        manager.register(Box::new(metrics_plugin));

        assert_eq!(manager.plugin_count(), 2);

        let names = manager.get_plugin_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Logger"));
        assert!(names.contains(&"Metrics"));
    }

    #[test]
    fn test_trait_objects() {
        let mut manager = PluginManager::new();

        // Test that we can store different types implementing Plugin
        manager.register(Box::new(LogPlugin::new("Log".to_string())));
        manager.register(Box::new(MetricsPlugin::new("Metrics".to_string(), 30)));

        // This should work without panicking - testing dynamic dispatch
        manager.run_all();

        assert_eq!(manager.plugin_count(), 2);
    }

    #[test]
    fn test_multiple_trait_implementations() {
        let mut log_plugin = LogPlugin::new("TestLogger".to_string());

        // Test Plugin trait
        assert_eq!(log_plugin.name(), "TestLogger");

        // Test Configurable trait
        log_plugin.configure("level=warn");
        assert_eq!(log_plugin.level(), "warn");

        // Both traits work on the same object
        log_plugin.execute(); // Should work without panicking
    }

    #[test]
    fn test_configuration_edge_cases() {
        let mut log_plugin = LogPlugin::new("Logger".to_string());
        let mut metrics_plugin = MetricsPlugin::new("Metrics".to_string(), 60);

        // Test empty config
        log_plugin.configure("");
        assert_eq!(log_plugin.level(), "info"); // Should remain default

        // Test invalid config
        log_plugin.configure("invalid_config");
        assert_eq!(log_plugin.level(), "info");

        // Test partial config
        metrics_plugin.configure("interval=");
        assert_eq!(metrics_plugin.interval(), 60); // Should remain unchanged

        // Test config without equals
        metrics_plugin.configure("interval");
        assert_eq!(metrics_plugin.interval(), 60);
    }

    #[test]
    fn test_plugin_polymorphism() {
        // Test that we can treat different plugins polymorphically
        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(LogPlugin::new("Logger1".to_string())),
            Box::new(LogPlugin::new("Logger2".to_string())),
            Box::new(MetricsPlugin::new("Metrics1".to_string(), 30)),
            Box::new(MetricsPlugin::new("Metrics2".to_string(), 60)),
        ];

        // Collect names through trait object
        let names: Vec<&str> = plugins.iter().map(|p| p.name()).collect();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"Logger1"));
        assert!(names.contains(&"Logger2"));
        assert!(names.contains(&"Metrics1"));
        assert!(names.contains(&"Metrics2"));

        // Execute all through trait object
        for plugin in &plugins {
            plugin.execute(); // Should work without panicking
        }
    }

    #[test]
    fn test_configurable_plugins_with_manager() {
        let mut manager = PluginManager::new();

        // Create configurable plugins
        let mut log_plugin = LogPlugin::new("ConfigurableLogger".to_string());
        let mut metrics_plugin = MetricsPlugin::new("ConfigurableMetrics".to_string(), 60);

        // Configure them
        log_plugin.configure("level=error");
        metrics_plugin.configure("interval=30");

        // Verify configuration took effect
        assert_eq!(log_plugin.level(), "error");
        assert_eq!(metrics_plugin.interval(), 30);

        // Add to manager
        manager.register(Box::new(log_plugin));
        manager.register(Box::new(metrics_plugin));

        assert_eq!(manager.plugin_count(), 2);
    }
}