use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use toml;

/// Application configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Directory where chunks are stored
    pub chunk_dir: PathBuf,
    
    /// Maximum number of tokens per chunk (8192 = ~6K words)
    pub max_tokens_per_chunk: usize,
    
    /// Enable debug features (like UI state dump)
    pub enable_debug: bool,
    
    /// Directory to save debug output files
    pub debug_dir: PathBuf,
    
    /// Default source directory to open on startup
    pub source_dir: PathBuf,
    
    /// Vim keybindings enabled
    pub vim_keybindings: bool,
    
    /// Auto-save chunks when reaching max token count
    pub auto_save_chunks: bool,
}

impl Default for Config {
    fn default() -> Self {
        // Default configuration values
        Self {
            // Default to "chunks" subdirectory in current directory
            chunk_dir: PathBuf::from("chunks"),
            
            // Claude model context size (8192 tokens ≈ 6K words)
            max_tokens_per_chunk: 8192,
            
            // Debug features disabled by default in production
            enable_debug: false,
            
            // Debug output directory
            debug_dir: PathBuf::from("debug"),
            
            // Default to current directory
            source_dir: PathBuf::from("."),
            
            // Vim keybindings enabled by default
            vim_keybindings: true,
            
            // Don't auto-save chunks by default
            auto_save_chunks: false,
        }
    }
}

impl Config {
    /// Load configuration from the default locations
    /// 
    /// Searches in the following order:
    /// 1. ./packrat.toml (current directory)
    /// 2. $XDG_CONFIG_HOME/packrat/config.toml (or equivalent on other platforms)
    /// 3. Falls back to default config if none found
    pub fn load() -> Result<Self> {
        // Try current directory first
        let local_config = Path::new("packrat.toml");
        if local_config.exists() {
            return Self::load_from_file(local_config)
                .context("Failed to load config from current directory");
        }
        
        // Try user config directory
        if let Some(project_dirs) = ProjectDirs::from("com", "packrat", "packrat") {
            let config_dir = project_dirs.config_dir();
            let user_config = config_dir.join("config.toml");
            
            if user_config.exists() {
                return Self::load_from_file(&user_config)
                    .context("Failed to load config from user config directory");
            }
        }
        
        // No config file found, return default
        Ok(Self::default())
    }
    
    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse TOML config from: {}", path.display()))
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Serialize to TOML
        let toml_str = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        
        // Write to file
        let mut file = File::create(path)
            .with_context(|| format!("Failed to create config file: {}", path.display()))?;
        
        file.write_all(toml_str.as_bytes())
            .with_context(|| format!("Failed to write config to: {}", path.display()))?;
        
        Ok(())
    }
    
    /// Generate a default configuration file in the user's config directory
    pub fn create_default_config() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("com", "packrat", "packrat")
            .ok_or_else(|| anyhow!("Could not determine config directory"))?;
        
        let config_dir = project_dirs.config_dir();
        let config_path = config_dir.join("config.toml");
        
        let config = Config::default();
        config.save_to_file(&config_path)?;
        
        Ok(config_path)
    }
    
    /// Get the absolute path for the chunk directory
    pub fn absolute_chunk_dir(&self) -> PathBuf {
        if self.chunk_dir.is_absolute() {
            self.chunk_dir.clone()
        } else {
            // Get the current directory and join with the relative path
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            current_dir.join(&self.chunk_dir)
        }
    }
    
    /// Get the absolute path for the source directory
    pub fn absolute_source_dir(&self) -> PathBuf {
        if self.source_dir.is_absolute() {
            self.source_dir.clone()
        } else {
            // Get the current directory and join with the relative path
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            current_dir.join(&self.source_dir)
        }
    }
}