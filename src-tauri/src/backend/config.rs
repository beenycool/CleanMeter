use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use serde_json::Value;

pub struct ConfigManager {
    file_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut path = if let Ok(app_data) = std::env::var("APPDATA") {
            PathBuf::from(app_data)
        } else {
            PathBuf::from(".")
        };
        path.push("CleanMeter");
        
        // Ensure parent directory exists
        let _ = fs::create_dir_all(&path);
        
        path.push("config.json");
        Self { file_path: path }
    }

    pub fn read_all(&self) -> HashMap<String, Value> {
        if !self.file_path.exists() {
            return HashMap::new();
        }

        let content = match fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(_) => return HashMap::new(),
        };

        serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
    }

    pub fn save_all(&self, config: &HashMap<String, Value>) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&self.file_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    pub fn get_preference_string(&self, key: &str) -> Option<String> {
        let config = self.read_all();
        config.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    pub fn get_preference_boolean(&self, key: &str, default_value: bool) -> bool {
        let config = self.read_all();
        config.get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default_value)
    }

    pub fn set_preference(&self, key: String, value: Value) -> Result<(), String> {
        let mut config = self.read_all();
        config.insert(key, value);
        self.save_all(&config)
    }
}
