# ReplyNow 配置助手 (ReplyNow Config GUI)

<p align="center">
  <img src="public/logo.svg" width="120" height="120" alt="ReplyNow Config GUI Logo" />
</p>

**ReplyNow 配置助手** 是一款基于 **Tauri 2.0 + React + TypeScript** 构建的轻量级、跨平台桌面客户端工具。旨在为使用 OpenAI Codex CLI 的开发者提供可视化的自定义 API 接入与认证配置服务，简化繁琐的文件寻找与手动编写配置文件的流程。

---

## ✨ 核心特性

1.  **🚀 环境与路径自动匹配 (Cross-Platform Path Auto-Detection)**
    *   启动时自动检测当前操作系统 (macOS / Windows)。
    *   自动定位 Codex 默认配置目录：
        *   **macOS / Linux**: `~/.codex/`
        *   **Windows**: `%USERPROFILE%\.codex\`
    *   如果环境未就绪，界面将提供 **一键初始化** 按钮自动生成必需的文件夹与基础配置文件。

2.  **⚙️ 可视化配置表单 & 高级配置**
    *   **基础模式**：仅需输入 API Base URL 与 API Key（支持隐藏显影切换），即可完成快速配置。
    *   **高级模式 (Advanced Settings)**：展开后支持定制高级内置字段，完全适应复杂的代理网关环境：
        *   `model`：自定义核心模型名称 (默认：`gpt-5.5`)。
        *   `model_reasoning_effort`：选择推理努力程度。
        *   `network_access`：设置网络连通策略 (`enabled`/`disabled`)。
        *   `model_verbosity`：调节输出冗余度。
        *   `disable_response_storage`：一键开启或禁用响应结果持久化存储。
        *   `requires_openai_auth`：是否强制 OpenAI 格式认证。
        *   `api_key_name`：自定义写往 `auth.json` 时的密钥键名（如 `OPENAI_API_KEY`）。

3.  **⚡ 高精度 API 连通性测试 (Handshake Connection Prober)**
    *   一键发送低延迟握手探测包。
    *   高精度计算网络往返时延（RTT Latency）。
    *   支持双重探测机制：优先获取 `/models`，并在 404 时优雅回退至 `/chat/completions` 以确保高度兼容各类代理中转站，测试失败时提供详细报错日志输出。

4.  **🛡️ 安全备份与一键恢复 (Auto-Backup & Rollback)**
    *   每次点击保存应用时，软件会在后台自动为原 `config.toml` 和 `auth.json` 生成带时间戳的备份文件，存储于 `~/.codex/backups/` 中。
    *   界面提供 **恢复上一次配置** 按钮，可随时一键回滚。

5.  **🎨 极致暗黑科技美学与微动效 (Sleek Dark Mode & Visuals)**
    *   专为开发者打造的暗黑渐变风（Sleek Space Dark Mode）。
    *   **成功 Pulse 波动效果**：网络测试连通成功时，状态圆点会触发绿色波纹扩散扩散动效。
    *   **失败 Shake 抖动反馈**：配置错误或测试不通时，输入框触发轻微左右抖动。
    *   **旋转加载**：按键与状态栏加载的动态 Spinner 效果。

---

## 🛠️ 技术栈说明

*   **跨平台容器**：Tauri 2.0 (Rust 强力驱动，极致包体积 ~15MB，运行内存极低 30MB+)
*   **前端逻辑**：React 19 + TypeScript + Vite
*   **配置文件修改器**：Rust `toml_edit` (非破坏性解析，只修改指定键，**完美保留您原有的 TOML 注释以及非冲突的高级配置**)
*   **网络客户端**：Rust `reqwest`
*   **图标资源**：Lucide React

---

## 💻 本地开发指南

### 前置依赖
*   **Node.js**: v18.0 或更高版本。
*   **Rust 环境**：
    *   macOS: 运行 `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` 或使用 `brew install rust`。
    *   Windows: 安装 [Rustup](https://rustup.rs/) 并确保配置 C++ 生成工具。

### 运行步骤
1.  **克隆并进入项目根目录**。
2.  **安装前端依赖**：
    ```bash
    npm install
    ```
3.  **启动开发环境 (Tauri Dev)**：
    ```bash
    npm run tauri dev
    ```
4.  **编译打包生产版本**：
    ```bash
    npm run tauri build
    ```

---

## 📦 自动化发布流程 (GitHub Actions CI/CD)

项目已集成 GitHub 工作流 [.github/workflows/release.yml](.github/workflows/release.yml)。
只要您将代码提交并推送到 `main` 分支，GitHub 会自动启动多平台构建节点，在线打包出 Windows 绿色版 `.exe` 与安装包 `.msi`、以及 macOS `.dmg`，并自动发布在仓库的 Releases 页面中。
