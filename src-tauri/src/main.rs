#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aiSales::marketing::{ChromeService, run_zhihu_task, run_xiaohongshu_task};
use tauri::State;
use log::info;
use serde_json::json;
use std::sync::Mutex;

/// 应用状态 - 组合配置状态和Chrome服务
pub struct AppState {
    pub config_state: aiSales::AppState,
    pub chrome_service: Mutex<ChromeService>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config_state: aiSales::AppState::default(),
            chrome_service: Mutex::new(ChromeService::new()),
        }
    }
}

/// 获取配置
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<aiSales::AppConfig, String> {
    aiSales::get_config(&state.config_state)
}

/// 保存配置
#[tauri::command]
async fn save_config(config: aiSales::AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    aiSales::save_config(&config, &state.config_state)
}

/// 获取平台配置
#[tauri::command]
async fn get_platform_config(platform: String, state: State<'_, AppState>) -> Result<aiSales::PlatformConfig, String> {
    aiSales::get_platform_config(&platform, &state.config_state)
}

/// 保存平台配置
#[tauri::command]
async fn save_platform_config(platform: String, config: aiSales::PlatformConfig, state: State<'_, AppState>) -> Result<(), String> {
    aiSales::save_platform_config(&platform, &config, &state.config_state)
}

/// 启动Chrome实例
#[tauri::command]
async fn start_chrome(
    state: State<'_, AppState>,
    account: String,
) -> Result<serde_json::Value, String> {
    info!("API调用: start_chrome, account={}", account);

    let mut service = state.chrome_service.lock().map_err(|e| e.to_string())?;
    let result = service.start_instance(&account);

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// 停止Chrome实例
#[tauri::command]
async fn stop_chrome(
    state: State<'_, AppState>,
    account: String,
) -> Result<serde_json::Value, String> {
    info!("API调用: stop_chrome, account={}", account);

    let mut service = state.chrome_service.lock().map_err(|e| e.to_string())?;
    let result = service.stop_instance(&account);

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// 获取Chrome实例状态
#[tauri::command]
async fn get_chrome_status(
    state: State<'_, AppState>,
    account: String,
) -> Result<serde_json::Value, String> {
    info!("API调用: get_chrome_status, account={}", account);

    let service = state.chrome_service.lock().map_err(|e| e.to_string())?;
    let result = service.get_instance_status(&account);

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// 列出所有Chrome实例
#[tauri::command]
async fn list_chrome_instances(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    info!("API调用: list_chrome_instances");

    let service = state.chrome_service.lock().map_err(|e| e.to_string())?;
    let result = service.list_instances();

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

/// 执行任务
#[tauri::command]
async fn run_task(
    state: State<'_, AppState>,
    account: String,
    task_type: String,
    keywords: Vec<String>,
    content: String,
    max_count: u32,
) -> Result<serde_json::Value, String> {
    info!("API调用: run_task, account={}, task_type={}", account, task_type);

    let ws_url = {
        let mut service = state.chrome_service.lock().map_err(|e| e.to_string())?;
        let result = service.get_or_start_instance(&account);

        if result.get("res").and_then(|v| v.as_i64()) != Some(0) {
            return Err(result.get("errmsg").and_then(|v| v.as_str()).unwrap_or("未知错误").to_string());
        }

        result.get("datawf")
            .and_then(|d| d.get("ws_url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
            .ok_or("无法获取WebSocket URL")?
    };

    let (browser, mut handler) = chromiumoxide::Browser::connect(&ws_url)
        .await
        .map_err(|e| format!("连接Chrome失败: {}", e))?;

    tokio::spawn(async move {
        use futures::StreamExt;
        while let Some(h) = handler.next().await {
            let _ = h;
        }
    });

    let result = if account.starts_with("zhihu_") {
        run_zhihu_task(&browser, &task_type, keywords, &content, max_count).await
    } else {
        run_xiaohongshu_task(&browser, &task_type, keywords, &content, max_count).await
    };

    Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_platform_config,
            save_platform_config,
            start_chrome,
            stop_chrome,
            get_chrome_status,
            list_chrome_instances,
            run_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {}
