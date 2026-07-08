import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Eye,
  EyeOff,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Save,
  RefreshCw,
  Play,
  ChevronDown,
  ChevronUp,
  Minus,
  X
} from "lucide-react";
import "./App.css";

interface CodexStatus {
  exists: boolean;
  config_exists: boolean;
  auth_exists: boolean;
}

interface TestResult {
  success: boolean;
  latency_ms: number;
  error: string | null;
}

interface AppConfig {
  base_url: string;
  api_key: string;
  raw_toml: string;
}

function App() {
  const [baseUrl, setBaseUrl] = useState("https://api.replynow.cn:6688/v1");
  const [apiKey, setApiKey] = useState("");
  const [showApiKey, setShowApiKey] = useState(false);
  const [isWindows, setIsWindows] = useState(false);
  const [showGuideModal, setShowGuideModal] = useState(false);
  
  // Advanced fields state
  const [rawToml, setRawToml] = useState("");

  const [isAdvancedOpen, setIsAdvancedOpen] = useState(false);
  
  // Codex status: 'checking' | 'ready' | 'missing'
  const [codexStatus, setCodexStatus] = useState<"checking" | "ready" | "missing">("checking");
  
  // Test status: 'idle' | 'testing' | 'success' | 'error'
  const [testStatus, setTestStatus] = useState<"idle" | "testing" | "success" | "error">("idle");
  const [latency, setLatency] = useState<number | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [shake, setShake] = useState(false);
  
  // Toast notifications
  const [toast, setToast] = useState<{ message: string; isError: boolean } | null>(null);

  // Load status and config on startup
  useEffect(() => {
    checkStatus();
    loadConfig();

    // Detect OS
    const userAgent = navigator.userAgent.toLowerCase();
    if (userAgent.includes("win")) {
      setIsWindows(true);
    }
  }, []);

  // Auto-hide toast after 3 seconds
  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => setToast(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [toast]);

  async function checkStatus() {
    try {
      const status = await invoke<CodexStatus>("check_codex_status");
      if (status.exists && status.config_exists && status.auth_exists) {
        setCodexStatus("ready");
      } else {
        setCodexStatus("missing");
      }
    } catch (err) {
      console.error("Failed to check codex status", err);
      setCodexStatus("missing");
    }
  }


  async function loadConfig() {
    try {
      const config = await invoke<AppConfig>("load_config");
      if (config.base_url) setBaseUrl(config.base_url);
      if (config.api_key) setApiKey(config.api_key);
      if (config.raw_toml !== undefined) setRawToml(config.raw_toml);
    } catch (err) {
      console.error("Failed to load configuration", err);
    }
  }

  async function handleSave() {
    try {
      const config = {
        base_url: baseUrl,
        api_key: apiKey,
        raw_toml: rawToml
      };
      await invoke("save_config", { config });
      setToast({ message: "Configuration saved and backed up!", isError: false });
      await checkStatus();
      await loadConfig();
    } catch (err: any) {
      setToast({ message: `Failed to save: ${err}`, isError: true });
    }
  }

  async function handleRestore() {
    try {
      await invoke("restore_last_backup");
      await loadConfig();
      await checkStatus();
      setToast({ message: "Configuration rolled back to last backup!", isError: false });
    } catch (err: any) {
      setToast({ message: `Rollback failed: ${err}`, isError: true });
    }
  }

  async function handleTest() {
    setTestStatus("testing");
    setLatency(null);
    setErrorMessage(null);
    try {
      const result = await invoke<TestResult>("test_connection", { baseUrl, apiKey });
      if (result.success) {
        setTestStatus("success");
        setLatency(result.latency_ms);
      } else {
        setTestStatus("error");
        setErrorMessage(result.error);
        triggerShake();
      }
    } catch (err: any) {
      setTestStatus("error");
      setErrorMessage(err.toString());
      triggerShake();
    }
  }

  function triggerShake() {
    setShake(true);
    setTimeout(() => setShake(false), 350);
  }

  return (
    <div className="app-wrapper">
      {isWindows && (
        <div className="custom-titlebar" data-tauri-drag-region>
          <div className="titlebar-drag" data-tauri-drag-region />
          <div className="titlebar-controls">
            <button
              onClick={() => getCurrentWindow().minimize()}
              className="titlebar-btn"
              title="最小化"
            >
              <Minus size={14} />
            </button>
            <button
              onClick={() => getCurrentWindow().close()}
              className="titlebar-btn titlebar-btn-close"
              title="关闭"
            >
              <X size={14} />
            </button>
          </div>
        </div>
      )}

      <div className="app-container">
        {/* Header */}
        <header className="header" data-tauri-drag-region>
          <div className="brand" data-tauri-drag-region>
            <img src="/logo.svg" className="brand-logo" alt="Logo" />
            <span className="brand-title">ReplyNow 配置助手</span>
          </div>
        
        <div 
          className={`status-badge ${codexStatus === "missing" ? "status-badge-clickable" : ""}`}
          onClick={() => { if (codexStatus === "missing") setShowGuideModal(true); }}
          title={codexStatus === "missing" ? "点击查看安装及登录指引" : undefined}
        >
          <div className={`status-dot ${codexStatus === "ready" ? "ready" : "missing"} ${testStatus === "success" ? "pulse-success-dot" : ""}`} />
          {codexStatus === "checking" && <span>检测中...</span>}
          {codexStatus === "ready" && <span>● Codex 已就绪</span>}
          {codexStatus === "missing" && (
            <span style={{ color: "#f59e0b" }}>● 未检测到 Codex 配置 (点击修复)</span>
          )}
        </div>
      </header>

      {/* Form */}
      <main className="config-form">
        {codexStatus === "missing" && (
          <div className="codex-warning-box codex-warning-box-clickable" onClick={() => setShowGuideModal(true)}>
            <AlertCircle size={16} />
            <span>未检测到 Codex 配置。点击查看 Codex 安装及登录指引解锁软件。</span>
          </div>
        )}
        <div className="form-group">
          <label className="form-label">API 地址 (API Base URL)</label>
          <div className="input-container">
            <input
              type="text"
              className={`form-input ${shake ? "shake-error" : ""}`}
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder="https://api.replynow.cn:6688/v1"
            />
          </div>
        </div>

        <div className="form-group">
          <label className="form-label">API 密钥 (API Key)</label>
          <div className="input-container">
            <input
              type={showApiKey ? "text" : "password"}
              className={`form-input ${shake ? "shake-error" : ""}`}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="sk-..."
            />
            <button
              type="button"
              className="password-toggle"
              onClick={() => setShowApiKey(!showApiKey)}
            >
              {showApiKey ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>
        </div>

        {/* Advanced Settings Toggle */}
        <div className="advanced-toggle" onClick={() => setIsAdvancedOpen(!isAdvancedOpen)}>
          <span className="advanced-toggle-text">高级参数设置 (Advanced Settings)</span>
          {isAdvancedOpen ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        </div>

        {/* Advanced Settings Section */}
        {isAdvancedOpen && (
          <div className="advanced-textarea-container">
            <label className="form-label font-xs">高级参数编辑器 (config.toml)</label>
            <textarea
              className="advanced-textarea"
              value={rawToml}
              onChange={(e) => setRawToml(e.target.value)}
              placeholder="# 写入或编辑 config.toml 参数..."
              spellCheck={false}
            />
          </div>
        )}

        {/* Actions */}
        <div className="actions-panel">
          <div className="actions-left">
            <button
              type="button"
              className="btn btn-secondary"
              onClick={handleTest}
              disabled={testStatus === "testing"}
            >
              {testStatus === "testing" ? (
                <div className="spinner" />
              ) : (
                <Play size={14} />
              )}
              测试连接
            </button>
            
            <button
              type="button"
              className="btn btn-secondary"
              onClick={handleRestore}
              title="还原上一次保存的配置文件"
              disabled={codexStatus === "missing"}
            >
              <RefreshCw size={14} />
              恢复上一次配置
            </button>
          </div>

          <button
            type="button"
            className="btn btn-primary"
            onClick={handleSave}
            disabled={codexStatus === "missing"}
          >
            <Save size={14} />
            一键保存并应用
          </button>
        </div>

        {/* Feedback Panel */}
        <section className="feedback-panel">
          <div className="feedback-header">状态反馈面板</div>
          {testStatus === "idle" && (
            <div className="feedback-content" style={{ color: "var(--text-secondary)" }}>
              <AlertCircle size={14} />
              <span>输入配置信息后，点击“测试连接”按钮以验证 API 连通性。</span>
            </div>
          )}
          {testStatus === "testing" && (
            <div className="feedback-content">
              <div className="spinner" style={{ color: "var(--color-secondary)" }} />
              <span>正在向 API 发送连接探测，请稍候...</span>
            </div>
          )}
          {testStatus === "success" && (
            <div className="feedback-content" style={{ color: "#10b981" }}>
              <CheckCircle2 size={14} />
              <span>连接成功! 响应延迟: {latency}ms</span>
            </div>
          )}
          {testStatus === "error" && (
            <div style={{ display: "flex", flexDirection: "column" }}>
              <div className="feedback-content" style={{ color: "#f87171" }}>
                <XCircle size={14} />
                <span>连接失败，请检查配置信息或网络连接。</span>
              </div>
              {errorMessage && (
                <div className="feedback-error-log">{errorMessage}</div>
              )}
            </div>
          )}
        </section>
      </main>

      {/* Codex Installation & Login Guide Modal */}
      {showGuideModal && (
        <div className="modal-backdrop">
          <div className="modal-content fade-in">
            <div className="modal-header">
              <h3 className="modal-title">Codex 安装与登录指引</h3>
              <button className="modal-close-btn" onClick={() => setShowGuideModal(false)}>
                <X size={16} />
              </button>
            </div>
            
            <div className="steps-container">
              <div className="step-item">
                <div className="step-badge">1</div>
                <div className="step-details">
                  <div className="step-title">下载并安装 Codex</div>
                  <div className="step-desc">
                    请前往官方网站下载并安装 <strong>Codex 桌面客户端</strong>：
                    <a 
                      href="#" 
                      className="modal-link" 
                      onClick={(e) => { 
                        e.preventDefault(); 
                        openUrl("https://chatgpt.com/codex").catch(console.error); 
                      }}
                    >
                      点击打开下载页面
                    </a>
                  </div>
                </div>
              </div>

              <div className="step-item">
                <div className="step-badge">2</div>
                <div className="step-details">
                  <div className="step-title">登录 GPT 账户</div>
                  <div className="step-desc">
                    启动 Codex 客户端，并登录您的 GPT 账户。登录成功后，系统会自动在本地创建认证文件 <code>auth.json</code> 和配置文件 <code>config.toml</code>。
                  </div>
                </div>
              </div>

              <div className="step-item">
                <div className="step-badge">3</div>
                <div className="step-details">
                  <div className="step-title">完成验证并解锁</div>
                  <div className="step-desc">
                    完成上述两步后，点击下方“重新检测配置”按钮验证环境并解锁软件。
                  </div>
                </div>
              </div>
            </div>

            <div className="modal-actions">
              <button
                type="button"
                className="btn btn-secondary btn-sm"
                onClick={async () => {
                  try {
                    await invoke("initialize_codex");
                    setToast({ message: "占位配置文件已成功创建！", isError: false });
                    await checkStatus();
                    await loadConfig();
                    setShowGuideModal(false);
                  } catch (err: any) {
                    setToast({ message: `初始化失败: ${err}`, isError: true });
                  }
                }}
                title="为您直接生成基础占位配置文件（绕过客户端）"
              >
                极速初始化
              </button>
              
              <button
                type="button"
                className="btn btn-primary"
                onClick={async () => {
                  try {
                    const status = await invoke<CodexStatus>("check_codex_status");
                    if (status.exists && status.config_exists && status.auth_exists) {
                      setToast({ message: "检测成功，环境已就绪！", isError: false });
                      await checkStatus();
                      await loadConfig();
                      setShowGuideModal(false);
                    } else {
                      setToast({ message: "仍然未检测到配置文件，请确认您已完成登录并生成配置文件。", isError: true });
                    }
                  } catch (err: any) {
                    setToast({ message: `检测出错: ${err}`, isError: true });
                  }
                }}
              >
                重新检测配置
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Toast Notification */}
      {toast && (
        <div className={`toast ${toast.isError ? "error" : ""}`}>
          {toast.isError ? <XCircle size={16} /> : <CheckCircle2 size={16} />}
          <span>{toast.message}</span>
        </div>
      )}
    </div>
    </div>
  );
}

export default App;
