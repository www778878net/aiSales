//! 营销自动化微服务
//!
//! 提供Chrome实例管理和任务执行API

pub mod chrome;

use serde::{Deserialize, Serialize};

pub use chrome::ChromeService;

/// 任务结果
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub processed: u32,
    pub success_count: u32,
    pub failed_count: u32,
}


