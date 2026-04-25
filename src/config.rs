//! Configuration management module

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ui::tui::theme::ColorScheme;

/// Kloud configuration
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM configuration
    #[serde(default)]
    pub llm: LlmConfig,

    /// Agent configuration
    #[serde(default)]
    pub agent: AgentConfig,

    /// Tools configuration
    #[serde(default)]
    pub tools: ToolsConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
}

impl Config {
    /// Load environment variables into config
    pub fn load_from_env(&mut self) {
        self.llm.load_from_env();
        self.agent.load_from_env();
        self.tools.load_from_env();
        self.ui.load_from_env();
    }

    /// Load configuration from file and environment variables
    ///
    /// Priority:
    /// 1. `KLOUD_CONFIG_FILE` env var (must exist, or error)
    /// 2. XDG path ~/.config/kloud/settings.toml (create if not exist)
    pub fn load() -> Result<Self, crate::Error> {
        // Check KLOUD_CONFIG_FILE env var first
        if let Ok(path) = std::env::var("KLOUD_CONFIG_FILE") {
            let path = PathBuf::from(path);
            if !path.exists() {
                return Err(crate::Error::Config(crate::error::ConfigError::FileNotFound(
                    path.to_string_lossy().to_string(),
                )));
            }
            let mut config = Self::load_from_file(&path)?;
            config.load_from_env();
            return Ok(config);
        }

        // Use XDG config path: $XDG_CONFIG_HOME/kloud/settings.toml
        // Fallback to $HOME/.config/kloud/settings.toml
        let config_path = if let Ok(xdg_home) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_home).join("kloud").join("settings.toml")
        } else if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.home_dir().join(".config").join("kloud").join("settings.toml")
        } else {
            return Ok(Config::default());
        };

        if config_path.exists() {
            let mut config = Self::load_from_file(&config_path)?;
            config.load_from_env();
            tracing::debug!(?config_path, "loaded config");
            return Ok(config);
        }

        // File doesn't exist, create default config
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(crate::error::ConfigError::CreateDirError(e.to_string()))
            })?;
        }

        let default_config = Config::default();
        let toml_str = toml::to_string_pretty(&default_config).map_err(|e| {
            crate::Error::Config(crate::error::ConfigError::SerializeError(e.to_string()))
        })?;

        std::fs::write(&config_path, &toml_str).map_err(|e| {
            crate::Error::Config(crate::error::ConfigError::WriteError(e.to_string()))
        })?;

        tracing::debug!(?config_path, "created default config");
        Ok(default_config)
    }

    /// Load configuration from a specific file path
    fn load_from_file(path: &PathBuf) -> Result<Self, crate::Error> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::Error::Config(crate::error::ConfigError::ReadError(e.to_string()))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            crate::Error::Config(crate::error::ConfigError::ParseError(e.to_string()))
        })?;

        Ok(config)
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API key for LLM service
    #[serde(default)]
    pub api_key: Option<String>,

    /// Base URL for API endpoint
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,

    /// Model name to use
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum tokens in context
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base_url: default_api_base_url(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

impl LlmConfig {
    fn load_from_env(&mut self) {
        if let Ok(v) = std::env::var("KLOUD_API_KEY") {
            self.api_key = Some(v);
        }
        if let Ok(v) = std::env::var("KLOUD_API_BASE_URL") {
            self.api_base_url = v;
        }
        if let Ok(v) = std::env::var("KLOUD_MODEL") {
            self.model = v;
        }
    }
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum concurrent tasks
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,

    /// Default task timeout in seconds
    #[serde(default = "default_task_timeout")]
    pub task_timeout: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            task_timeout: default_task_timeout(),
        }
    }
}

impl AgentConfig {
    fn load_from_env(&mut self) {
        if let Ok(v) = std::env::var("KLOUD_MAX_CONCURRENT")
            && let Ok(n) = v.parse()
        {
            self.max_concurrent = n;
        }
        if let Ok(v) = std::env::var("KLOUD_TASK_TIMEOUT")
            && let Ok(n) = v.parse()
        {
            self.task_timeout = n;
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Built-in color scheme for the TUI.
    #[serde(default)]
    pub color_scheme: ColorScheme,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Dark,
        }
    }
}

impl UiConfig {
    fn load_from_env(&mut self) {
        // UiConfig currently has no env overrides
    }
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Commands explicitly allowed for shell-style tools.
    #[serde(default)]
    pub allowed_commands: Vec<String>,

    /// Whether path-based tool security checks are enabled.
    #[serde(default = "default_true")]
    pub path_security: bool,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            allowed_commands: Vec::new(),
            path_security: true,
        }
    }
}

impl ToolsConfig {
    fn load_from_env(&mut self) {
        // ToolsConfig currently has no env overrides
        // This method is here for future extensibility
    }
}

// Default value functions
fn default_api_base_url() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_max_tokens() -> u32 {
    8096
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_concurrent() -> usize {
    5
}

fn default_task_timeout() -> u64 {
    300
}

fn default_true() -> bool {
    true
}
