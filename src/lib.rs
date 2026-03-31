//! AI Sales Assistant - 开源库
//!
//! 提供配置管理功能
//! 注意：业务逻辑由 marketingPrivate crate 提供

pub mod config;

// 重导出配置管理类型
pub use config::{
    AppState, AppConfig, LlmProvider, PlatformConfig,
    get_config, save_config, get_platform_config, save_platform_config,
};
