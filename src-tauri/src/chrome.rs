//! Chrome实例管理服务
//!
//! 管理Chrome实例的启动、停止和状态查询
//! 只管启动实例、分配端口，不关心任务类型

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde_json::{json, Value};
use log::info;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Chrome实例信息
#[derive(Debug, Clone)]
pub struct ChromeInstance {
    pub account: String,
    pub port: u16,
    pub pid: Option<u32>,
    pub user_data_dir: String,
    pub status: String,
    pub ws_url: Option<String>,
}

/// Chrome实例管理服务
pub struct ChromeService {
    instances: HashMap<String, ChromeInstance>,
}

impl ChromeService {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// 根据账号计算端口
    pub fn get_port_for_account(account: &str) -> u16 {
        let mut hasher = DefaultHasher::new();
        account.hash(&mut hasher);
        let hash = hasher.finish();
        9222 + (hash % 78) as u16
    }

    /// 查找Chrome路径
    fn find_chrome_path() -> Option<PathBuf> {
        let possible_paths: Vec<&str> = vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        ];
        
        for path_str in possible_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// 列出所有实例
    pub fn list_instances(&self) -> HashMap<String, Value> {
        let instances_json: Vec<Value> = self.instances
            .values()
            .map(|i| {
                json!({
                    "account": i.account,
                    "port": i.port,
                    "pid": i.pid,
                    "user_data_dir": i.user_data_dir,
                    "status": i.status,
                    "ws_url": i.ws_url
                })
            })
            .collect();

        HashMap::from([
            ("res".to_string(), json!(0)),
            ("errmsg".to_string(), json!("")),
            ("datawf".to_string(), json!({
                "total": instances_json.len(),
                "instances": instances_json
            })),
        ])
    }

    /// 检查实例是否运行
    pub fn is_instance_running(&self, account: &str) -> bool {
        if let Some(instance) = self.instances.get(account) {
            if instance.status == "running" {
                // 检查端口是否响应
                let url = format!("http://localhost:{}/json/version", instance.port);
                if let Ok(resp) = ureq::get(&url).timeout(std::time::Duration::from_secs(2)).call() {
                    return resp.status() == 200;
                }
            }
        }
        false
    }

    /// 启动实例
    pub fn start_instance(&mut self, account: &str) -> HashMap<String, Value> {
        // 检查是否已在运行
        if self.is_instance_running(account) {
            info!("Chrome实例 {} 已在运行，复用现有实例", account);
            return HashMap::from([
                ("res".to_string(), json!(0)),
                ("errmsg".to_string(), json!(format!("实例 {} 已在运行", account))),
                ("datawf".to_string(), json!({
                    "account": account,
                    "started": false,
                    "reused": true,
                    "status": "running"
                })),
            ]);
        }

        let chrome_path = match Self::find_chrome_path() {
            Some(p) => p,
            None => {
                return HashMap::from([
                    ("res".to_string(), json!(-1)),
                    ("errmsg".to_string(), json!("未找到Chrome")),
                    ("datawf".to_string(), json!({"account": account, "started": false})),
                ]);
            }
        };

        let port = Self::get_port_for_account(account);
        let user_data_dir = format!("C:\\chrome\\{}", account);

        // 创建目录
        if let Err(e) = std::fs::create_dir_all(&user_data_dir) {
            return HashMap::from([
                ("res".to_string(), json!(-1)),
                ("errmsg".to_string(), json!(format!("创建目录失败: {}", e))),
                ("datawf".to_string(), json!({"account": account, "started": false})),
            ]);
        }

        // 删除锁文件
        let lock_files = ["SingletonLock", "SingletonCookie", "SingletonSocket"];
        for lock_file in lock_files {
            let lock_path = std::path::Path::new(&user_data_dir).join(lock_file);
            let _ = std::fs::remove_file(lock_path);
        }

        // 根据账号类型选择URL
        let url = if account.starts_with("zhihu_") {
            "https://www.zhihu.com"
        } else {
            "https://www.xiaohongshu.com"
        };

        info!("启动Chrome: account={}, port={}, dir={}", account, port, user_data_dir);

        // 启动Chrome
        let child = Command::new(&chrome_path)
            .args([
                format!("--remote-debugging-port={}", port),
                format!("--user-data-dir={}", user_data_dir),
                url.to_string(),
            ])
            .spawn();

        match child {
            Ok(c) => {
                let pid = c.id();
                // 防止进程被杀
                std::mem::forget(c);

                // 等待Chrome启动
                std::thread::sleep(std::time::Duration::from_secs(3));

                // 获取WebSocket URL
                let ws_url = self.get_ws_url(port);

                let instance = ChromeInstance {
                    account: account.to_string(),
                    port,
                    pid: Some(pid),
                    user_data_dir: user_data_dir.clone(),
                    status: "running".to_string(),
                    ws_url: ws_url.clone(),
                };

                self.instances.insert(account.to_string(), instance);

                info!("Chrome启动成功: account={}, port={}, pid={}", account, port, pid);

                HashMap::from([
                    ("res".to_string(), json!(0)),
                    ("errmsg".to_string(), json!("")),
                    ("datawf".to_string(), json!({
                        "account": account,
                        "port": port,
                        "pid": pid,
                        "started": true,
                        "reused": false,
                        "status": "running",
                        "ws_url": ws_url
                    })),
                ])
            }
            Err(e) => {
                HashMap::from([
                    ("res".to_string(), json!(-1)),
                    ("errmsg".to_string(), json!(format!("启动失败: {}", e))),
                    ("datawf".to_string(), json!({"account": account, "started": false})),
                ])
            }
        }
    }

    /// 获取WebSocket URL
    fn get_ws_url(&self, port: u16) -> Option<String> {
        let url = format!("http://localhost:{}/json/version", port);
        match ureq::get(&url).timeout(std::time::Duration::from_secs(5)).call() {
            Ok(resp) => {
                if resp.status() == 200 {
                    if let Ok(json_str) = resp.into_string() {
                        if let Ok(json) = serde_json::from_str::<Value>(&json_str) {
                            return json["webSocketDebuggerUrl"].as_str().map(|s| s.to_string());
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    /// 停止实例
    pub fn stop_instance(&mut self, account: &str) -> HashMap<String, Value> {
        if let Some(instance) = self.instances.get(account) {
            if let Some(pid) = instance.pid {
                // 杀掉进程
                let _ = Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .output();
            }
            self.instances.remove(account);
            info!("Chrome实例已停止: {}", account);
        }

        HashMap::from([
            ("res".to_string(), json!(0)),
            ("errmsg".to_string(), json!("")),
            ("datawf".to_string(), json!({
                "account": account,
                "stopped": true
            })),
        ])
    }

    /// 获取实例状态
    pub fn get_instance_status(&self, account: &str) -> HashMap<String, Value> {
        if let Some(instance) = self.instances.get(account) {
            return HashMap::from([
                ("res".to_string(), json!(0)),
                ("errmsg".to_string(), json!("")),
                ("datawf".to_string(), json!({
                    "account": instance.account,
                    "port": instance.port,
                    "pid": instance.pid,
                    "status": instance.status,
                    "ws_url": instance.ws_url
                })),
            ]);
        }

        HashMap::from([
            ("res".to_string(), json!(-1)),
            ("errmsg".to_string(), json!(format!("实例 {} 不存在", account))),
            ("datawf".to_string(), json!({"account": account, "status": "not_found"})),
        ])
    }

    /// 获取或启动实例
    pub fn get_or_start_instance(&mut self, account: &str) -> HashMap<String, Value> {
        if self.is_instance_running(account) {
            self.get_instance_status(account)
        } else {
            self.start_instance(account)
        }
    }
}

impl Default for ChromeService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_port_for_account() {
        let port1 = ChromeService::get_port_for_account("xiaohongshu_18169240160");
        let port2 = ChromeService::get_port_for_account("zhihu_18169240160");
        assert_ne!(port1, port2);
    }

    #[test]
    fn test_chrome_service_new() {
        let service = ChromeService::new();
        assert!(service.instances.is_empty());
    }
}
