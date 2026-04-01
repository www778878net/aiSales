#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::State;
use log::info;
use serde_json::json;

// 只导入类型，不导入函数（避免冲突）
use aiSales::{AppConfig, PlatformConfig, AppState as ConfigState};

// 导入 marketingPrivate 的库
use marketingPrivate::BrowserService;
use marketingPrivate::taskmanage::{TaskRunner, StepResult};
use zhihu::steps::{SearchStep, AnswerStep};
use xiaohongshu::steps::{SearchStep as XhsSearchStep, CommentStep as XhsCommentStep};
use chromiumoxide::Browser;
use futures::StreamExt;

/// 应用状态
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

/// 启动Chrome实例
#[tauri::command]
async fn start_chrome(account: String) -> Result<serde_json::Value, String> {
    info!("API调用: start_chrome, account={}", account);
    let mut service = BrowserService::default();
    let port = service.start_chrome_with_account(&account, None)
        .map_err(|e| format!("启动Chrome失败: {}", e))?;
    Ok(json!({ "res": 0, "errmsg": "", "datawf": { "account": account, "port": port, "started": true, "status": "running" } }))
}

/// 停止Chrome实例
#[tauri::command]
async fn stop_chrome(port: u16) -> Result<serde_json::Value, String> {
    info!("API调用: stop_chrome, port={}", port);
    let mut service = BrowserService::default();
    service.stop_instance(port);
    Ok(json!({ "res": 0, "errmsg": "", "datawf": { "port": port, "stopped": true } }))
}

/// 获取Chrome实例状态
#[tauri::command]
async fn get_chrome_status(port: u16) -> Result<serde_json::Value, String> {
    info!("API调用: get_chrome_status, port={}", port);
    let service = BrowserService::default();
    let instance = service.get_instance(port)
        .ok_or_else(|| format!("实例 {} 不存在", port))?;
    Ok(json!({ "res": 0, "errmsg": "", "datawf": { "port": instance.port, "account": instance.account, "status": format!("{:?}", instance.status) } }))
}

/// 列出所有Chrome实例
#[tauri::command]
async fn list_chrome_instances() -> Result<serde_json::Value, String> {
    info!("API调用: list_chrome_instances");
    let service = BrowserService::default();
    let instances = service.list_instances();
    let data: Vec<serde_json::Value> = instances.iter().map(|i| {
        json!({ "port": i.port, "account": i.account, "status": format!("{:?}", i.status) })
    }).collect();
    Ok(json!({ "res": 0, "errmsg": "", "data": data }))
}

/// 任务结果
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub processed: u32,
    pub success_count: u32,
    pub failed_count: u32,
}

/// 执行知乎任务
async fn run_zhihu_task(browser: &Browser, keywords: Vec<String>, content: &str, max_count: u32) -> TaskResult {
    let mut runner = TaskRunner::new("zhihu_task".to_string(), "zhihu".to_string());
    runner.register(SearchStep::new(keywords.clone()));
    runner.register(AnswerStep::new(content.to_string(), max_count));
    let result = runner.run(browser, "zhihu_search").await;
    match result {
        StepResult::Over => TaskResult { success: true, message: "任务完成".to_string(), processed: 1, success_count: 1, failed_count: 0 },
        StepResult::Error(e) => TaskResult { success: false, message: e, processed: 0, success_count: 0, failed_count: 1 },
        StepResult::DeviceLost(e) => TaskResult { success: false, message: format!("设备丢失: {}", e), processed: 0, success_count: 0, failed_count: 1 },
        _ => TaskResult { success: true, message: "任务完成".to_string(), processed: 1, success_count: 1, failed_count: 0 },
    }
}

/// 执行小红书任务
async fn run_xiaohongshu_task(browser: &Browser, keywords: Vec<String>, content: &str, max_count: u32) -> TaskResult {
    let mut runner = TaskRunner::new("xiaohongshu_task".to_string(), "xiaohongshu".to_string());
    runner.register(XhsSearchStep::new(keywords.clone()));
    runner.register(XhsCommentStep::new(content.to_string(), max_count));
    let result = runner.run(browser, "xiaohongshu_search").await;
    match result {
        StepResult::Over => TaskResult { success: true, message: "小红书任务完成".to_string(), processed: 1, success_count: 1, failed_count: 0 },
        StepResult::Error(e) => TaskResult { success: false, message: e, processed: 0, success_count: 0, failed_count: 1 },
        StepResult::DeviceLost(e) => TaskResult { success: false, message: format!("设备丢失: {}", e), processed: 0, success_count: 0, failed_count: 1 },
        _ => TaskResult { success: true, message: "小红书任务完成".to_string(), processed: 1, success_count: 1, failed_count: 0 },
    }
}

/// 执行任务
#[tauri::command]
async fn run_task(
    account: String,
    task_type: String,
    keywords: Vec<String>,
    content: String,
    max_count: u32,
) -> Result<serde_json::Value, String> {
    info!("API调用: run_task, account={}, task_type={}", account, task_type);

    let ws_url = {
        let mut service = BrowserService::default();
        let start_url = if account.starts_with("zhihu_") {
            "https://www.zhihu.com".to_string()
        } else {
            "https://www.xiaohongshu.com".to_string()
        };
        let port = service.start_chrome_with_account(&account, Some(start_url))
            .map_err(|e| format!("启动Chrome失败: {}", e))?;
        let instance = service.get_instance(port)
            .ok_or_else(|| "无法获取Chrome实例".to_string())?;
        instance.ws_url.clone().ok_or_else(|| "无法获取WebSocket URL".to_string())?
    };

    let (browser, mut handler) = chromiumoxide::Browser::connect(&ws_url)
        .await
        .map_err(|e| format!("连接Chrome失败: {}", e))?;

    tokio::spawn(async move {
        while let Some(_h) = handler.next().await {}
    });

    let result = if account.starts_with("zhihu_") {
        run_zhihu_task(&browser, keywords, &content, max_count).await
    } else {
        run_xiaohongshu_task(&browser, keywords, &content, max_count).await
    };

    Ok(json!(result))
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
            start_chrome,
            stop_chrome,
            get_chrome_status,
            list_chrome_instances,
            run_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}
