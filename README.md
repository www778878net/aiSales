# AI Sales Assistant / 开源AI销售助手

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-green)](LICENSE)

**AI驱动的多平台社交媒体营销自动化平台 | AI-Powered Multi-Platform Social Media Marketing Automation**

---

## 中文简介

AI Sales Assistant 是一款基于 **Rust + Tauri** 构建的现代化桌面应用程序，专为社交媒体营销人员设计。它集成了知乎、小红书等主流平台的自动化营销能力，通过 AI 技术实现内容发布、用户互动、数据分析等核心营销功能的自动化。

### 核心特性

- 🚀 **跨平台支持**: 支持知乎、小红书等主流社交平台（持续扩展中）
- 🤖 **AI驱动**: 集成大语言模型，智能生成营销内容
- ⚡ **高性能**: 基于 Rust 构建，内存占用小，运行效率高
- 🖥️ **原生体验**: Tauri 原生桌面应用，响应迅速，体验流畅
- 🔧 **可扩展架构**: 模块化设计，易于添加新平台和新功能
- 📊 **数据本地化**: 数据存储于本地，安全可控

### 技术栈

- **后端**: Rust (async/await, tokio)
- **前端**: HTML/CSS/JavaScript (WebView)
- **框架**: Tauri 2.0
- **数据库**: 本地 SQLite (通过 LocalDB)
- **AI能力**: marketingPrivate 私有核心库

---

## English Introduction

AI Sales Assistant is a modern desktop application built with **Rust + Tauri**, designed specifically for social media marketers. It integrates automated marketing capabilities for major platforms like Zhihu and Xiaohongshu, leveraging AI technology to automate core marketing functions such as content publishing, user engagement, and data analysis.

### Key Features

- 🚀 **Multi-Platform Support**: Zhihu, Xiaohongshu, and more (expanding continuously)
- 🤖 **AI-Powered**: LLM integration for intelligent content generation
- ⚡ **High Performance**: Rust-based, low memory footprint, efficient execution
- 🖥️ **Native Experience**: Tauri native desktop app, responsive and smooth
- 🔧 **Extensible Architecture**: Modular design, easy to add new platforms and features
- 📊 **Local Data Storage**: SQLite database, secure and controllable

### Tech Stack

- **Backend**: Rust (async/await, tokio)
- **Frontend**: HTML/CSS/JavaScript (WebView)
- **Framework**: Tauri 2.0
- **Database**: Local SQLite (via LocalDB)
- **AI Capabilities**: marketingPrivate core library

---

## 快速开始 | Quick Start

### 环境要求

- Rust 1.75+
- Node.js 18+ (用于 Tauri 依赖)
- 目标平台构建工具:
  - **Windows**: Visual Studio C++ Build Tools / MSVC
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file` 等

### 安装与构建

```bash
# 克隆仓库
git clone https://github.com/yourorg/aiSales.git
cd aiSales

# 初始化子模块
git submodule update --init --recursive

# 构建项目
cargo build --release

# 运行应用
cargo run
```

### Tauri 开发模式

```bash
# 前端开发模式 (热重载)
cargo tauri dev

# 构建生产版本
cargo tauri build
```

---

## 项目结构 | Project Structure

```
crates/aiSales/
├── src/
│   ├── lib.rs          # 主库入口 - 配置管理
│   ├── config/         # 配置模块
│   └── ...
├── src-tauri/
│   ├── src/
│   │   ├── main.rs    # 应用入口
│   │   ├── lib.rs     # Tauri 库 - Chrome 服务
│   │   └── chrome.rs  # Chrome 实例管理
│   ├── tauri.conf.json # Tauri 配置
│   └── icons/         # 应用图标
├── resources/
│   └── index.html     # 前端界面
├── Cargo.toml         # 包配置
└── README.md          # 本文档
```

### 核心 Crate

| Crate | 作用 | Purpose |
|-------|------|---------|
| `marketingPrivate` | 私有核心营销逻辑库 | Private core marketing logic |
| `zhihu` | 知乎平台实现 | Zhihu platform implementation |
| `xiaohongshu` | 小红书平台实现 | Xiaohongshu platform implementation |
| `marketingbase` | 共享营销工具和类型 | Shared marketing utilities |
| `base` | 基础架构支持 | Base infrastructure support |

---

## 贡献 | Contributing

欢迎提交 Issue 和 Pull Request！请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

---

## 许可证 | License

本项目采用 MIT 或 Apache-2.0 双许可证，详见 [LICENSE](LICENSE) 文件。

This project is dual-licensed under MIT or Apache-2.0, see [LICENSE](LICENSE) for details.

---

## 相关链接 | Links

- **主页**: https://github.com/yourorg/aiSales
- **问题追踪**: https://github.com/yourorg/aiSales/issues
- **文档**: 待补充

---

**Built with ❤️ using Rust and Tauri**
