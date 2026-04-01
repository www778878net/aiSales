//! 集成测试 - 验证前端配置保存功能
//!
//! 这个测试模拟前端调用 Tauri 命令的完整流程

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// 测试：前端保存配置 → 后端读取配置
    #[test]
    fn test_frontend_config_flow() {
        // 创建临时目录和配置文件
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.ini");

        // 创建 AppState（模拟 Tauri 应用状态）
        let state = Arc::new(aiSales::AppState {
            running: Mutex::new(false),
            config_path: config_path.clone(),
        });

        // 1. 模拟前端传入的配置对象
        let frontend_config = aiSales::AppConfig {
            page_wait_time: 15,
            comment_interval: 20,
            max_notes: 10,
            keywords: vec!["测试关键词".to_string(), "Rust".to_string()],
            current_provider: "openai".to_string(),
            providers: {
                let mut map = std::collections::HashMap::new();
                map.insert("openai".to_string(), aiSales::LlmProvider {
                    api_url: "https://api.openai.com/v1".to_string(),
                    api_key: "sk-test-key".to_string(),
                    model: "gpt-4".to_string(),
                });
                map
            },
            platforms: {
                let mut map = std::collections::HashMap::new();
                map.insert("xiaohongshu".to_string(), true);
                map.insert("zhihu".to_string(), true);
                map
            },
            accounts: vec!["xiaohongshu_13800138000".to_string()],
        };

        // 2. 调用保存命令（模拟前端 save_config_command）
        let save_result = aiSales::save_config(&frontend_config, &state);
        assert!(save_result.is_ok(), "保存配置应该成功: {:?}", save_result.err());

        // 3. 调用加载命令（模拟前端 get_config_command）
        let loaded_config = aiSales::get_config(&state);
        assert!(loaded_config.is_ok(), "加载配置应该成功");

        let loaded = loaded_config.unwrap();

        // 4. 验证所有字段都被正确保存和加载
        assert_eq!(loaded.page_wait_time, frontend_config.page_wait_time);
        assert_eq!(loaded.comment_interval, frontend_config.comment_interval);
        assert_eq!(loaded.max_notes, frontend_config.max_notes);
        assert_eq!(loaded.keywords, frontend_config.keywords);
        assert_eq!(loaded.current_provider, frontend_config.current_provider);
        assert_eq!(loaded.providers["openai"].api_url, frontend_config.providers["openai"].api_url);
        assert_eq!(loaded.providers["openai"].api_key, frontend_config.providers["openai"].api_key);
        assert_eq!(loaded.providers["openai"].model, frontend_config.providers["openai"].model);
        assert_eq!(loaded.platforms["xiaohongshu"], frontend_config.platforms["xiaohongshu"]);
        assert_eq!(loaded.platforms["zhihu"], frontend_config.platforms["zhihu"]);
        assert_eq!(loaded.accounts, frontend_config.accounts);

        println!("✅ 前端配置流程测试通过！");
    }

    /// 测试：平台配置保存和读取
    #[test]
    fn test_platform_config_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("platform_test.ini");

        let state = Arc::new(aiSales::AppState {
            running: Mutex::new(false),
            config_path: config_path.clone(),
        });

        // 先保存一个基础配置
        let base_config = aiSales::AppConfig::default();
        aiSales::save_config(&base_config, &state).unwrap();

        // 平台配置
        let platform_config = aiSales::PlatformConfig {
            keywords: vec!["小红书".to_string(), "营销".to_string()],
            comment: "这是一条测试评论".to_string(),
            enabled: true,
        };

        // 保存平台配置
        let save_result = aiSales::save_platform_config("xiaohongshu", &platform_config, &state);
        assert!(save_result.is_ok(), "保存平台配置应该成功");

        // 读取平台配置
        let loaded = aiSales::get_platform_config("xiaohongshu", &state);
        assert!(loaded.is_ok(), "读取平台配置应该成功");

        let loaded = loaded.unwrap();
        assert_eq!(loaded.keywords, platform_config.keywords);
        assert_eq!(loaded.comment, platform_config.comment);
        assert_eq!(loaded.enabled, platform_config.enabled);

        println!("✅ 平台配置集成测试通过！");
    }
}
