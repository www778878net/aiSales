#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;
use log::info;

use aiSales::{AppConfig, PlatformConfig, AppState as ConfigState};
use marketing::MarketingController;

/// 运行状态
pub struct AppState {
    pub config_state: ConfigState,
    pub running: AtomicBool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config_state: ConfigState::default(),
            running: AtomicBool::new(false),
        }
    }
}

/// 获取配置
#[tauri::command]
async fn get_config_command(state: State<'_, AppState>) -> Result<AppConfig, String> {
    aiSales::get_config(&state.config_state)
}

/// 保存配置
#[tauri::command]
async fn save_config_command(config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    aiSales::save_config(&config, &state.config_state)
}

/// 获取平台配置
#[tauri::command]
async fn get_platform_config_command(platform: String, state: State<'_, AppState>) -> Result<PlatformConfig, String> {
    aiSales::get_platform_config(&platform, &state.config_state)
}

/// 保存平台配置
#[tauri::command]
async fn save_platform_config_command(platform: String, config: PlatformConfig, state: State<'_, AppState>) -> Result<(), String> {
    aiSales::save_platform_config(&platform, &config, &state.config_state)
}

/// 启动 - 直接调用 marketing::auto_run
#[tauri::command]
async fn start_marketing(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("启动 marketing auto");

    // 检查是否已运行
    if state.running.load(Ordering::SeqCst) {
        return Err("任务已在运行中".to_string());
    }

    // 先停止已有的 Chrome
    let controller = MarketingController::new();
    controller.stop_all();

    state.running.store(true, Ordering::SeqCst);

    // 直接调用 marketing::auto_run
    tokio::spawn(async move {
        let result = marketing::auto_run().await;
        match result {
            Ok(_) => info!("marketing auto 完成"),
            Err(e) => info!("marketing auto 错误: {}", e),
        }
    });

    Ok(serde_json::json!({
        "res": 0,
        "errmsg": "",
        "datawf": { "started": true }
    }))
}

/// 停止
#[tauri::command]
async fn stop_marketing(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("停止 marketing");

    state.running.store(false, Ordering::SeqCst);

    // 停止 Chrome
    let controller = MarketingController::new();
    controller.stop_all();

    Ok(serde_json::json!({
        "res": 0,
        "errmsg": "",
        "datawf": { "stopped": true }
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_config_command,
            save_config_command,
            get_platform_config_command,
            save_platform_config_command,
            start_marketing,
            stop_marketing,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}
