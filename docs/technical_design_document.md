# 技术设计文档 (TDD) - ReplyNow 配置助手 (ReplyNow Config GUI)

## 1. 技术栈选型 (Technology Stack)

*   **跨平台桌面框架**：**Tauri 2.0 (Rust backend + React/TypeScript frontend)**
    *   *说明*：利用系统原生 WebView 渲染，内存占用极低（30MB-50MB），打包体积在 10MB 左右，非常适合作为轻量级工具。
*   **前端框架**：**React 18 + Vite** (基于 TypeScript)
*   **样式库**：**Vanilla CSS**
*   **Rust 依赖包**：
    *   `toml_edit`：用于读写和解析 TOML，支持在写入时**完整保留文件中原有的注释和未修改字段**，避免覆盖 Codex 原本自带的默认配置。
    *   `serde` / `serde_json`：JSON 文件的序列化和反序列化。
    *   `reqwest`：发送 API 连通性测试请求。
    *   `dirs`：用于解析跨平台的用户家目录。

---

## 2. 配置文件写入机制与逻辑 (Config Writing Logic)

当用户在界面输入 API Base URL 与 API Key 并点击保存时，软件需要对 `config.toml` 和 `auth.json` 进行如下修改：

### 2.1 `config.toml` 写入格式
写入时只修改 `model_provider` 字段，并新增或修改 `[model_providers.replynow]` 块，以保留其他默认参数不被清空。

```toml
# 激活当前自定义服务商
model_provider = "replynow"
preferred_auth_method = "apikey"

[model_providers.replynow]
name = "replynow"
base_url = "https://api.replynow.cn:6688/v1" # 或者是用户修改后的自定义 URL
env_key = "REPLYNOW_API_KEY"
wire_api = "responses"
requires_openai_auth = false
```

### 2.2 `auth.json` 写入格式
将 API Key 写入以 `env_key` 为键名的项中：

```json
{
  "REPLYNOW_API_KEY": "sk-your-actual-api-key"
}
```

---

## 3. 数据模型设计 (Data Models)

### 3.1 TypeScript 侧接口定义
```typescript
interface AppConfig {
  baseUrl: string;  // 默认为 "https://api.replynow.cn:6688/v1"
  apiKey: string;   // 用户输入的 API Key
}
```

### 3.2 Rust 侧 TOML 解析模型 (通过 `toml_edit` 动态修改)
为了不破坏用户原有的 `config.toml` 中的其他高级配置（如沙箱模式等），Rust 端应使用 `toml_edit` 库进行解析，而非直接用 `serde` 强行反序列化：

```rust
use std::fs;
use toml_edit::{DocumentMut, Item, Table};

pub fn update_config_toml(path: &std::path::Path, base_url: &str) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    
    // 1. 设置当前的 provider
    doc["model_provider"] = toml_edit::value("replynow");
    doc["preferred_auth_method"] = toml_edit::value("apikey");
    
    // 2. 获取或创建 model_providers 表
    let providers = doc.entry("model_providers").or_insert(toml_edit::table());
    
    if let Some(providers_table) = providers.as_table_mut() {
        let mut replynow = Table::new();
        replynow.insert("name", toml_edit::value("replynow"));
        replynow.insert("base_url", toml_edit::value(base_url));
        replynow.insert("env_key", toml_edit::value("REPLYNOW_API_KEY"));
        replynow.insert("wire_api", toml_edit::value("responses"));
        replynow.insert("requires_openai_auth", toml_edit::value(false));
        
        providers_table.insert("replynow", Item::Table(replynow));
    }
    
    fs::write(path, doc.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}
```

---

## 4. 后端 API 接口设计 (Tauri Commands)

主进程向前端暴露 3 个核心 API：

### 4.1 读取配置 `load_config`
*   **输入**：无
*   **逻辑**：
    1. 定位 `~/.codex/config.toml` 和 `~/.codex/auth.json`。
    2. 若文件存在，解析并提取 `base_url`（从 `model_providers.replynow` 中提取）及 `api_key`（从 `auth.json` 中以 `REPLYNOW_API_KEY` 为键提取）。
    3. 若不存在，返回默认 URL：`https://api.replynow.cn:6688/v1`，API Key 为空。

### 4.2 保存配置 `save_config`
*   **输入**：`base_url: String`, `api_key: String`
*   **逻辑**：
    1. **安全备份**：先将当前的 `config.toml` 和 `auth.json` 复制备份到 `~/.codex/backups/` 中。
    2. 使用 `toml_edit` 写入 `config.toml`。
    3. 将 `{"REPLYNOW_API_KEY": api_key}` 写入 `auth.json`。
    4. 返回成功。

### 4.3 测试连接 `test_connection`
*   **输入**：`base_url: String`, `api_key: String`
*   **逻辑**：
    1. 构建 `reqwest` 请求，向 `${base_url}/chat/completions` 或兼容的探测接口发送请求（测试连通性与 API 响应延迟）。
    2. 记录请求前后时间差，计算延迟值（毫秒）。
    3. 返回连通状态与延迟结果。
