# 项目路线图与指南 - ReplyNow 配置助手 (ReplyNow Config GUI)

## 1. 开发环境搭建指南 (Environment Setup)

### 1.1 安装依赖环境
1.  **Node.js**: 安装 v18.0 或更高版本。
2.  **Rust 编译器**:
    *   在 macOS/Linux 上：运行 `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    *   在 Windows 上：下载并运行 [rustup-init.exe](https://rustup.rs/)，并确保安装了 C++ Build Tools。

### 1.2 初始化 Tauri 项目
在当前项目根目录下，执行以下命令初始化 Tauri 项目结构（本软件为全新项目，不可直接二次开发 `cc-switch`）：

```bash
# 使用 create-tauri-app 初始化全新项目
npm create tauri-app@latest
# 交互选项推荐：
# ? Project name: replynow-codex-config
# ? Choose which package manager to use: npm
# ? Choose your UI template: React
# ? Choose your programming language: TypeScript

# 进入目录并安装前端图标依赖
cd replynow-codex-config
npm install lucide-react
```

---

## 2. 开发里程碑划分 (Milestones)

由于需求进行了极简优化，去除了侧边栏导航、MCP 管理和多参数选择，开发工期显著缩短。

### 阶段 1：项目骨架与系统集成 (MVP)
*   **交付物**：全新项目搭建完毕，前端能渲染基本单页布局，Rust 后端能够识别操作系统路径，并与前端建立基本 IPC 连接。
*   **工期估算**：1 天

### 阶段 2：后端文件操作逻辑 (核心读写)
*   **交付物**：
    *   实现 Rust 侧利用 `toml_edit` 库解析与修改 `~/.codex/config.toml`（写入 `[model_providers.replynow]` 并不破坏原有 TOML 的其他字段）。
    *   实现 Rust 侧读写 `~/.codex/auth.json`，写入 `REPLYNOW_API_KEY`。
    *   建立安全机制：写入前自动在 `~/.codex/backups/` 目录下生成带时间戳的文件备份，并能一键回滚。
*   **工期估算**：2 天

### 阶段 3：接口连通性测试 (网络联调)
*   **交付物**：
    *   实现 `test_connection` 接口，前端点击测试后，向用户填写的 URL 发送握手请求，计算高精度延迟（Latency）并返回。
    *   前端集成加载动画、成功 Pulse 绿波脉冲及失败 Shake 边框动效。
*   **工期估算**：1 天

### 阶段 4：打包发布与 CI/CD 流程
*   **交付物**：
    *   配置好 `.github/workflows/release.yml` 脚本。
    *   在合并主分支时，自动打包生成 macOS `.dmg` 与 Windows `.exe` / `.msi` 安装包。
*   **工期估算**：1 天

---

## 3. 测试策略与验证方案 (Testing Strategy)

### 3.1 单元测试 (Unit Testing - Rust)
开发人员需在 `src-tauri/src` 下编写针对配置解析的单元测试：
*   **配置合并测试**：提供包含用户原有自定义配置的 TOML，调用 `update_config_toml` 写入，验证生成的 TOML 是否完整保留了原有其他字段，只修改了 `model_provider` 和 `model_providers.replynow`。
*   **备份机制测试**：确保在写入前，原有文件成功复制 to 备份目录。

### 3.2 手动测试场景矩阵 (Manual Testing Matrix)
在测试阶段，需手动模拟以下边缘情况：
1.  **首发无目录测试**：首次安装 Codex 后，`.codex` 目录尚不存在。验证软件是否可以检测并提示用户点击“一键初始化”，并能正确自动创建该目录和空白配置文件。
2.  **API URL 尾部斜杠兼容性**：用户在输入框内填写的 API 地址，例如 `https://api.replynow.cn:6688/v1/`，软件应能自动截断尾部的 `/` 或进行兼容处理，确保写入配置及测试请求的正确性。
