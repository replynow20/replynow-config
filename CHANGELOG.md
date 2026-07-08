# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2026-07-08

### ✨ 核心特性与修复 (Changelog)

#### 1. 🚀 引导初始化环境优化
* 优化未检测到 Codex 配置时的交互引导。在右上角状态栏新增“● 未检测到 Codex 配置 (点击修复)”状态按钮，点击该按钮能直接拉起引导弹窗。
* 弹出全新的 **Step-by-Step 引导弹窗**，指引用户下载安装 **Codex 桌面客户端**并登录。
* 整合 `@tauri-apps/plugin-opener` 插件，安全地在用户的默认系统浏览器中打开下载网页。
* 修复了指示圆点显示瑕疵，只保留 CSS 彩色指示小圆点，去除了文本内硬编码的 Unicode 字符点。

#### 2. ⚙️ 配置文件合并与切换逻辑修复
* 修复了 Rust 后端在保存时无法强制将根节点 `model_provider` 切换为 `"replynow"` 的问题。
* 确保 `[model_providers.replynow]` 作为一个独立的 provider 节点能独立完整补全所有关键项（`name`、`base_url`、`wire_api`、`requires_openai_auth`），不会因其他子表（如 `custom`）的存在而发生写出字段缺失。
* **去除了默认的 `env_key` 强行写入**：若用户的 TOML 缺失该字段，不再强行插入 `env_key = "OPENAI_API_KEY"` 以保持 TOML 文件的整洁干净，在缺失时内部继续默认读写 `"OPENAI_API_KEY"`。

#### 3. 🎨 字号优化与高级参数悬浮弹窗（零布局偏移）
* **字号整体调大**：将全局基准字号整体提升 `1px - 2px`，极大改善了 macOS 和 Windows 在高分屏下的可读性。
* **零布局偏移高级配置**：彻底移除折叠式的 inline 文本域，在主界面中仅保留一个文字链接样式的“高级参数设置”按钮。点击后，以**毛玻璃全屏覆盖弹窗 (Modal Editor)** 的形式渲染 config.toml 编辑器，彻底规避了折叠造成的界面推挤与滚动条问题，提供高度一致且静止的现代 UI 体验。
