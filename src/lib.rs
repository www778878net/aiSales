//! AI Sales Assistant - 开源库
//!
//! 提供配置管理、任务调度、平台集成等功能

pub mod config;
pub mod marketing;

// 重导出常用类型
pub use config::{
    AppState, AppConfig, LlmProvider, PlatformConfig,
    get_config, save_config, get_platform_config, save_platform_config,
};

// 重导出营销自动化功能
pub use marketing::{
    ChromeService, TaskResult, run_zhihu_task, run_xiaohongshu_task,
};
