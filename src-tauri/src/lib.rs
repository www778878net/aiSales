//! AI Sales Assistant - 开源库
//!
//! 职责：
//! 1. 读写配置文件
//! 2. 启动和停止 marketing.exe
//!
//! 业务逻辑由 marketingPrivate crate 提供

// 重导出 marketingPrivate 的类型，方便使用
pub use marketing::{MarketingController, TaskResult, BrowserService};
