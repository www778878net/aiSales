#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::State;
use log::info;

use aiSales::{AppConfig, PlatformConfig, AppState as ConfigState};

/// 运行状态（使用静态变量确保全局唯一）
static RUNNING: AtomicBool = AtomicBool::new(false);

/// 运行状态
pub struct AppState {
    pub config_state: ConfigState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config_state: ConfigState::default(),
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
async fn start_marketing() -> Result<serde_json::Value, String> {
    info!("启动 marketing auto");

    // 使用 compare_exchange 防止并发
    match RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            // 重置停止标志
            marketing::set_stopped(false);

            // 成功获取锁，启动任务
            tokio::spawn(async {
                let result = marketing::auto_run().await;
                match result {
                    Ok(_) => info!("marketing auto 完成"),
                    Err(e) => info!("marketing auto 错误: {}", e),
                }
                RUNNING.store(false, Ordering::SeqCst);
            });

            Ok(serde_json::json!({
                "res": 0,
                "errmsg": "",
                "datawf": { "started": true }
            }))
        }
        Err(_) => {
            // 已经在运行
            Err("任务已在运行中".to_string())
        }
    }
}

/// 执行任务（前端调用的命令）
/// 第一次调用会真正执行，后续调用会被忽略
#[tauri::command]
async fn run_marketing_task(
    _account: String,
    _keywords: Vec<String>,
    _comment: String,
    _max_notes: u32,
) -> Result<serde_json::Value, String> {
    info!("run_marketing_task 被调用");

    // 使用 compare_exchange 防止并发
    match RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            // 成功获取锁，启动任务
            info!("开始执行 marketing::auto_run()");

            tokio::spawn(async {
                let result = marketing::auto_run().await;
                match result {
                    Ok(_) => info!("marketing auto 完成"),
                    Err(e) => info!("marketing auto 错误: {}", e),
                }
                RUNNING.store(false, Ordering::SeqCst);
            });

            Ok(serde_json::json!({
                "res": 0,
                "errmsg": "",
                "stats": { "success": 1, "failed": 0 }
            }))
        }
        Err(_) => {
            // 已经在运行，返回成功但不执行
            info!("任务已在运行中，忽略");
            Ok(serde_json::json!({
                "res": 0,
                "errmsg": "任务已在运行中",
                "stats": { "success": 0, "failed": 0 }
            }))
        }
    }
}

/// 停止
#[tauri::command]
async fn stop_marketing() -> Result<serde_json::Value, String> {
    info!("停止 marketing");

    RUNNING.store(false, Ordering::SeqCst);
    marketing::set_stopped(true);

    Ok(serde_json::json!({
        "res": 0,
        "errmsg": "",
        "datawf": { "stopped": true }
    }))
}

/// 检查是否正在运行
pub fn is_running() -> bool {
    RUNNING.load(Ordering::SeqCst)
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
            run_marketing_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}
