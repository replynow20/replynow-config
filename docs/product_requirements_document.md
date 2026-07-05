# 产品需求文档 (PRD) - ReplyNow 配置助手 (ReplyNow Config GUI)

## 1. 软件概述
**ReplyNow 配置助手** 是一款运行在 macOS 和 Windows 系统上的跨平台桌面客户端软件。该软件专门针对使用 OpenAI Codex CLI 并希望配置自定义 API 接口和 API Key 的开发者设计。

用户无需手动寻找并使用文本编辑器打开 `config.toml` 和 `auth.json` 配置文件，只需在软件界面中输入 API Base URL（默认填入 ReplyNow 接口地址）与 API Key，软件便会自动匹配运行环境路径，在后台安全地写入配置，使得小白用户也能一键轻松接入自定义 API 接口。

---

## 2. 目标用户与核心痛点
*   **目标用户**：
    *   使用 Codex CLI 的开发者，特别是需要接入自定义/中转 API 服务的用户。
    *   不熟悉文本编辑器（如 JSON / TOML 语法）的“电脑小白”。
*   **核心痛点**：
    *   **寻找路径繁琐**：隐藏的目录（Mac 的 `~/.codex` 或 Windows 的 `%USERPROFILE%\.codex`）对于小白用户来说很难找到。
    *   **语法极易出错**：手动编辑 TOML 格式的 `config.toml` 和 JSON 格式的 `auth.json` 时，缩进、双引号或键值写错会导致 CLI 运行崩溃。
    *   **无直观连通测试**：手动配置后无法得知 API 是否可用，需要去命令行运行测试，体验较差。

---

## 3. 核心功能需求 (Functional Requirements)

### 3.1 环境与路径自动匹配 (Environment Detection & Auto-Matching)
*   **操作系统检测**：软件启动时自动识别当前系统为 macOS 还是 Windows。
*   **配置目录检索**：
    *   **macOS / Linux**: 定位 `~/.codex/`。
    *   **Windows**: 定位 `%USERPROFILE%\.codex\`。
*   **若未检测到目录**：显示友好提示，说明未检测到 Codex CLI 默认配置，可点击“一键初始化”创建默认文件夹及基础文件。

### 3.2 极简配置表单 (Simplified Config Form)
表单仅保留最核心的自定义 API 输入项，其余参数（如 Model 选择、安全沙箱等）均在 Codex 终端内部配置，此处不再冗余：
*   **API Base URL**（文本输入框）：
    *   **默认值**：`https://api.replynow.cn:6688/v1`。
    *   支持用户自定义输入其他接口地址。
*   **API Key**（密码输入框）：
    *   带“眼睛”图标切换显隐。
    *   用户输入后，软件自动写入 `auth.json` 及 `config.toml` 中的对应字段。

### 3.3 连接性与延迟测试 (Latency & Connection Testing)
*   **连通性测试按钮**：用户填写 API Base URL 和 API Key 后，点击“测试连接”。
*   **请求逻辑**：软件向用户填写的 URL 发送一个低消耗的探测请求，模拟 Codex 接口的握手（测试接口连通性）。
*   **结果反馈**：显示测试结果（绿灯/红灯）以及响应延迟（如 `Latency: 85ms`）。若失败，则在下方展示错误详情。

### 3.4 安全备份与还原 (Backup & Restore Management)
*   **自动备份机制**：每次在写入配置文件之前，软件自动在本地备份当前文件（如 `config.toml.20260705.bak`）。
*   **一键还原**：界面提供“恢复上一次配置”按钮，防止配置意外损坏。

---

## 4. 非功能性需求 (Non-Functional Requirements)

### 4.1 极致的性能与体积 (Performance & Bundle Size)
*   **包体积控制**：Windows 和 macOS 的安装包体积应保持在 15MB 以下（推荐使用 Tauri 2.0）。
*   **极佳视觉**：默认暗黑科技风（Sleek Dark Mode），带渐变色激活状态和流畅微动效，让小白用户感知到软件的高级与易用。

### 4.2 安全性 (Security & Privacy)
*   **本地运行**：软件为纯客户端，不收集、不上传任何 API Key。
*   **权限最小化**：仅申请读写 `.codex` 目录的权限，绝不越权。
