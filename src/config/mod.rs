use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;
use configparser::ini::Ini;
use log::{info, error, debug};

/// 应用状态
pub struct AppState {
    pub running: Mutex<bool>,
    pub config_path: PathBuf,
}

impl Default for AppState {
    fn default() -> Self {
        let config_path = find_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        Self {
            running: Mutex::new(false),
            config_path,
        }
    }
}

fn find_config_path() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    info!("Current dir: {:?}", current);

    loop {
        debug!("Checking path: {:?}", current);

        if current.join("apps").exists() && current.join("Cargo.toml").exists() {
            let config_path = current.join("docs/config/config.ini");
            info!("Found project root, config path: {:?}", config_path);
            return config_path;
        }

        if !current.pop() {
            break;
        }
    }

    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current.ends_with("src-tauri") {
        if current.pop() {
            if current.pop() {
                if current.pop() {
                    let config_path = current.join("docs/config/config.ini");
                    info!("Found project root via src-tauri fallback, config path: {:?}", config_path);
                    return config_path;
                }
            }
        }
    }

    let fallback = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("docs/config/config.ini");
    info!("Using fallback config path: {:?}", fallback);
    fallback
}

/// LLM 提供商配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmProvider {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

/// 应用配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub page_wait_time: u32,
    pub comment_interval: u32,
    pub max_notes: u32,
    pub keywords: Vec<String>,
    pub current_provider: String,
    pub providers: std::collections::HashMap<String, LlmProvider>,
    pub platforms: std::collections::HashMap<String, bool>,
    pub accounts: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut providers = std::collections::HashMap::new();
        providers.insert("modelscope".to_string(), LlmProvider {
            api_url: "https://api-inference.modelscope.cn/v1".to_string(),
            api_key: String::new(),
            model: "ZhipuAI/GLM-5".to_string(),
        });

        let mut platforms = std::collections::HashMap::new();
        platforms.insert("xiaohongshu".to_string(), true);
        platforms.insert("zhihu".to_string(), false);

        Self {
            page_wait_time: 10,
            comment_interval: 10,
            max_notes: 5,
            keywords: Vec::new(),
            current_provider: "modelscope".to_string(),
            providers,
            platforms,
            accounts: Vec::new(),
        }
    }
}

/// 平台配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlatformConfig {
    pub keywords: Vec<String>,
    pub comment: String,
    pub enabled: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            keywords: vec![],
            comment: String::new(),
            enabled: false,
        }
    }
}

/// 获取配置
pub fn get_config(state: &AppState) -> Result<AppConfig, String> {
    let mut conf = Ini::new();

    if state.config_path.exists() {
        let config_path_str = state.config_path.to_str()
            .ok_or_else(|| "Config path contains invalid UTF-8".to_string())?;
        conf.load(config_path_str)
            .map_err(|e| format!("Failed to read config: {}", e))?;
    }

    let current_provider = conf.get("llm", "current_provider")
        .unwrap_or_else(|| "modelscope".to_string());

    info!("Current provider: {}", current_provider);

    let mut providers = std::collections::HashMap::new();
    let provider_names = ["modelscope", "openai", "deepseek", "anthropic", "qwen"];

    for name in provider_names.iter() {
        let api_url = conf.get(name, "api_url").unwrap_or_default();
        let api_key = conf.get(name, "api_key").unwrap_or_default();
        let model = conf.get(name, "model").unwrap_or_default();

        if !api_url.is_empty() {
            info!("Provider {}: url={}, model={}", name, api_url, model);
            providers.insert(name.to_string(), LlmProvider {
                api_url,
                api_key,
                model,
            });
        }
    }

    if providers.is_empty() {
        providers.insert("modelscope".to_string(), LlmProvider {
            api_url: "https://api-inference.modelscope.cn/v1".to_string(),
            api_key: String::new(),
            model: "ZhipuAI/GLM-5".to_string(),
        });
    }

    let keywords_str = conf.get("general", "keywords").unwrap_or_default();
    let keywords: Vec<String> = keywords_str.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut platforms = std::collections::HashMap::new();
    platforms.insert("xiaohongshu".to_string(), conf.get("platforms", "xiaohongshu").map(|v| v == "true").unwrap_or(true));
    platforms.insert("zhihu".to_string(), conf.get("platforms", "zhihu").map(|v| v == "true").unwrap_or(false));

    let accounts_str = conf.get("general", "accounts").unwrap_or_default();
    let accounts: Vec<String> = accounts_str.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(AppConfig {
        page_wait_time: conf.getuint("general", "page_wait_time")
            .map_err(|e| e.to_string())?
            .unwrap_or(10) as u32,
        comment_interval: conf.getuint("general", "comment_interval")
            .map_err(|e| e.to_string())?
            .unwrap_or(10) as u32,
        max_notes: conf.getuint("general", "max_notes")
            .map_err(|e| e.to_string())?
            .unwrap_or(5) as u32,
        keywords,
        current_provider,
        providers,
        platforms,
        accounts,
    })
}

/// 保存配置
pub fn save_config(config: &AppConfig, state: &AppState) -> Result<(), String> {
    info!("save_config called, config_path: {:?}", state.config_path);

    if let Some(parent) = state.config_path.parent() {
        info!("Creating directory: {:?}", parent);
        fs::create_dir_all(parent)
            .map_err(|e| {
                error!("Failed to create config directory: {}", e);
                format!("Failed to create config directory: {}", e)
            })?;
    }

    let mut conf = Ini::new();

    conf.set("general", "page_wait_time", Some(config.page_wait_time.to_string()));
    conf.set("general", "comment_interval", Some(config.comment_interval.to_string()));
    conf.set("general", "max_notes", Some(config.max_notes.to_string()));
    conf.set("general", "keywords", Some(config.keywords.join(",")));
    conf.set("general", "accounts", Some(config.accounts.join(",")));

    for (name, enabled) in &config.platforms {
        conf.set("platforms", name, Some(enabled.to_string()));
    }

    conf.set("llm", "current_provider", Some(config.current_provider.clone()));

    for (name, provider) in &config.providers {
        conf.set(name, "api_url", Some(provider.api_url.clone()));
        conf.set(name, "api_key", Some(provider.api_key.clone()));
        conf.set(name, "model", Some(provider.model.clone()));
    }

    info!("Writing config to: {:?}", state.config_path);
    let config_path_str = state.config_path.to_str()
        .ok_or_else(|| "Config path contains invalid UTF-8".to_string())?;
    conf.write(config_path_str)
        .map_err(|e| {
            error!("Failed to save config: {}", e);
            format!("Failed to save config: {}", e)
        })?;

    info!("Config saved successfully");
    Ok(())
}

/// 获取平台配置
pub fn get_platform_config(platform: &str, state: &AppState) -> Result<PlatformConfig, String> {
    let mut conf = Ini::new();

    if state.config_path.exists() {
        let config_path_str = state.config_path.to_str()
            .ok_or_else(|| "Config path contains invalid UTF-8".to_string())?;
        conf.load(config_path_str)
            .map_err(|e| format!("Failed to read config: {}", e))?;
    }

    let keywords_str = conf.get(platform, "keywords").unwrap_or_default();
    let keywords: Vec<String> = keywords_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let comment = conf.get(platform, "comment").unwrap_or_default();
    let enabled = conf.getbool(platform, "enabled")
        .map_err(|e| e.to_string())?
        .unwrap_or(false);

    Ok(PlatformConfig {
        keywords,
        comment,
        enabled,
    })
}

/// 保存平台配置
pub fn save_platform_config(platform: &str, config: &PlatformConfig, state: &AppState) -> Result<(), String> {
    if let Some(parent) = state.config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let mut conf = Ini::new();

    if state.config_path.exists() {
        let config_path_str = state.config_path.to_str()
            .ok_or_else(|| "Config path contains invalid UTF-8".to_string())?;
        conf.load(config_path_str)
            .map_err(|e| format!("Failed to read config: {}", e))?;
    }

    conf.set(platform, "keywords", Some(config.keywords.join(",")));
    conf.set(platform, "comment", Some(config.comment.clone()));
    conf.set(platform, "enabled", Some(config.enabled.to_string()));

    let config_path_str = state.config_path.to_str()
        .ok_or_else(|| "Config path contains invalid UTF-8".to_string())?;
    conf.write(config_path_str)
        .map_err(|e| format!("Failed to save config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_roundtrip() {
        // Create temporary config file
        let file = NamedTempFile::new().unwrap();
        let config_path = file.path().to_path_buf();

        // Create state with temp config path
        let state = AppState {
            running: Mutex::new(false),
            config_path: config_path.clone(),
        };

        // Create a sample config
        let config = AppConfig {
            page_wait_time: 15,
            comment_interval: 20,
            max_notes: 10,
            keywords: vec!["test".to_string(), "example".to_string()],
            current_provider: "openai".to_string(),
            providers: {
                let mut map = std::collections::HashMap::new();
                map.insert("openai".to_string(), LlmProvider {
                    api_url: "https://api.openai.com/v1".to_string(),
                    api_key: "sk-test".to_string(),
                    model: "gpt-4".to_string(),
                });
                map
            },
            platforms: {
                let mut map = std::collections::HashMap::new();
                map.insert("xiaohongshu".to_string(), true);
                map.insert("zhihu".to_string(), false);
                map
            },
            accounts: vec!["xiaohongshu_123".to_string()],
        };

        // Save config
        save_config(&config, &state).expect("Failed to save config");

        // Load config
        let loaded = get_config(&state).expect("Failed to load config");

        // Verify fields
        assert_eq!(loaded.page_wait_time, config.page_wait_time);
        assert_eq!(loaded.comment_interval, config.comment_interval);
        assert_eq!(loaded.max_notes, config.max_notes);
        assert_eq!(loaded.keywords, config.keywords);
        assert_eq!(loaded.current_provider, config.current_provider);
        assert_eq!(loaded.providers["openai"].api_url, config.providers["openai"].api_url);
        assert_eq!(loaded.providers["openai"].api_key, config.providers["openai"].api_key);
        assert_eq!(loaded.providers["openai"].model, config.providers["openai"].model);
        assert_eq!(loaded.platforms["xiaohongshu"], config.platforms["xiaohongshu"]);
        assert_eq!(loaded.platforms["zhihu"], config.platforms["zhihu"]);
        assert_eq!(loaded.accounts, config.accounts);
    }

    #[test]
    fn test_platform_config_roundtrip() {
        let file = NamedTempFile::new().unwrap();
        let config_path = file.path().to_path_buf();

        let state = AppState {
            running: Mutex::new(false),
            config_path,
        };

        // First save a base config to avoid file not found issues
        let base_config = AppConfig::default();
        save_config(&base_config, &state).unwrap();

        let platform_config = PlatformConfig {
            keywords: vec!["rust".to_string()],
            comment: "Great post!".to_string(),
            enabled: true,
        };

        save_platform_config("xiaohongshu", &platform_config, &state).expect("Failed to save platform config");

        let loaded = get_platform_config("xiaohongshu", &state).expect("Failed to load platform config");

        assert_eq!(loaded.keywords, platform_config.keywords);
        assert_eq!(loaded.comment, platform_config.comment);
        assert_eq!(loaded.enabled, platform_config.enabled);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.page_wait_time, 10);
        assert_eq!(config.comment_interval, 10);
        assert_eq!(config.max_notes, 5);
        assert!(config.keywords.is_empty());
        assert_eq!(config.current_provider, "modelscope");
        assert!(config.providers.contains_key("modelscope"));
        assert!(config.platforms.contains_key("xiaohongshu"));
        assert!(config.platforms["xiaohongshu"]);
        assert!(!config.platforms["zhihu"]);
        assert!(config.accounts.is_empty());
    }
}

