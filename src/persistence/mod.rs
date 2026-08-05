use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// File format for persistent high score data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScoreFile {
    /// Best score ever achieved.
    pub best: u32,
}

impl HighScoreFile {
    /// Create a new record with the given score.
    #[allow(dead_code)]
    pub fn new(score: u32) -> Self {
        HighScoreFile { best: score }
    }

    /// Convert to JSON string.
    #[allow(dead_code)]
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Parse from a JSON string. Returns `None` if invalid or missing.
    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// Save the high score to disk at the platform-appropriate path.
#[allow(dead_code)]
pub fn save_high_score(path: &PathBuf, score: u32) -> Result<(), String> {
    let record = HighScoreFile::new(score);
    let json = record.to_json()?;

    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))
}

/// Load the high score from disk. Returns `None` if file missing or corrupted.
pub fn load_high_score(path: &PathBuf) -> Option<u32> {
    let json = fs::read_to_string(path).ok()?;
    let record = HighScoreFile::from_json(&json)?;
    Some(record.best)
}

/// Get the default persistence path using `dirs-next`.
pub fn default_path() -> PathBuf {
    dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rust_2048")
        .join("high_score.json")
}
