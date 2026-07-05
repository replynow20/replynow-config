use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct CodexStatus {
    exists: bool,
    config_exists: bool,
    auth_exists: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    base_url: String,
    api_key: String,
    raw_toml: String,
}

#[derive(Serialize)]
pub struct TestResult {
    success: bool,
    latency_ms: u128,
    error: Option<String>,
}

fn get_codex_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".codex"))
        .ok_or_else(|| "Could not locate home directory".to_string())
}

#[tauri::command]
fn check_codex_status() -> Result<CodexStatus, String> {
    let dir = get_codex_dir()?;
    let exists = dir.exists();
    let config_exists = dir.join("config.toml").exists();
    let auth_exists = dir.join("auth.json").exists();
    Ok(CodexStatus {
        exists,
        config_exists,
        auth_exists,
    })
}

#[tauri::command]
fn initialize_codex() -> Result<(), String> {
    let dir = get_codex_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    
    let config_path = dir.join("config.toml");
    let content = if config_path.exists() {
        std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?
    } else {
        r#"# OpenAI Codex CLI Configuration
# This file was initialized by ReplyNow Config GUI
"#.to_string()
    };
    let mut doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    
    // Set root-level defaults if they don't exist
    if doc.get("model_provider").is_none() {
        doc["model_provider"] = toml_edit::value("replynow");
    }
    if doc.get("model").is_none() {
        doc["model"] = toml_edit::value("gpt-5.5");
    }
    if doc.get("model_reasoning_effort").is_none() {
        doc["model_reasoning_effort"] = toml_edit::value("high");
    }
    if doc.get("network_access").is_none() {
        doc["network_access"] = toml_edit::value("enabled");
    }
    if doc.get("disable_response_storage").is_none() {
        doc["disable_response_storage"] = toml_edit::value(true);
    }
    if doc.get("model_verbosity").is_none() {
        doc["model_verbosity"] = toml_edit::value("high");
    }

    let providers = doc.entry("model_providers").or_insert(toml_edit::table());
    if let Some(providers_table) = providers.as_table_mut() {
        let replynow_item = providers_table.entry("replynow").or_insert(toml_edit::table());
        if let Some(replynow_table) = replynow_item.as_table_mut() {
            if replynow_table.get("name").is_none() {
                replynow_table.insert("name", toml_edit::value("replynow"));
            }
            if replynow_table.get("base_url").is_none() {
                replynow_table.insert("base_url", toml_edit::value("https://api.replynow.cn:6688/v1"));
            }
            if replynow_table.get("env_key").is_none() {
                replynow_table.insert("env_key", toml_edit::value("OPENAI_API_KEY"));
            }
            if replynow_table.get("wire_api").is_none() {
                replynow_table.insert("wire_api", toml_edit::value("responses"));
            }
            if replynow_table.get("requires_openai_auth").is_none() {
                replynow_table.insert("requires_openai_auth", toml_edit::value(true));
            }
        }
    }
    std::fs::write(&config_path, doc.to_string()).map_err(|e| e.to_string())?;

    let auth_path = dir.join("auth.json");
    let mut auth_obj = if auth_path.exists() {
        let content = std::fs::read_to_string(&auth_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content)
            .unwrap_or_else(|_| serde_json::Map::new())
    } else {
        serde_json::Map::new()
    };
    auth_obj.entry("OPENAI_API_KEY".to_string()).or_insert(serde_json::Value::String("".to_string()));
    let content = serde_json::to_string_pretty(&auth_obj).map_err(|e| e.to_string())?;
    std::fs::write(&auth_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    let dir = get_codex_dir()?;
    let config_path = dir.join("config.toml");
    let auth_path = dir.join("auth.json");

    let raw_toml;
    let mut base_url = "https://api.replynow.cn:6688/v1".to_string();
    let mut api_key = "".to_string();
    let mut api_key_name = "OPENAI_API_KEY".to_string();

    if config_path.exists() {
        raw_toml = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        if let Ok(doc) = raw_toml.parse::<DocumentMut>() {
            if let Some(providers) = doc.get("model_providers").and_then(|p| p.as_table()) {
                if let Some(replynow) = providers.get("replynow").and_then(|r| r.as_table()) {
                    if let Some(url) = replynow.get("base_url").and_then(|u| u.as_str()) {
                        base_url = url.to_string();
                    }
                    if let Some(env) = replynow.get("env_key").and_then(|u| u.as_str()) {
                        api_key_name = env.to_string();
                    }
                }
            }
        }
    } else {
        raw_toml = r#"# OpenAI Codex CLI Configuration
# This file was initialized by ReplyNow Config GUI

model_provider = "replynow"
model = "gpt-5.5"
model_reasoning_effort = "high"
network_access = "enabled"
disable_response_storage = true
model_verbosity = "high"

[model_providers.replynow]
name = "replynow"
base_url = "https://api.replynow.cn:6688/v1"
env_key = "OPENAI_API_KEY"
wire_api = "responses"
requires_openai_auth = true
"#.to_string();
    }

    if auth_path.exists() {
        let content = std::fs::read_to_string(&auth_path).map_err(|e| e.to_string())?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(key) = json.get(&api_key_name).and_then(|k| k.as_str()) {
                api_key = key.to_string();
            } else if api_key_name == "OPENAI_API_KEY" {
                if let Some(key) = json.get("REPLYNOW_API_KEY").and_then(|k| k.as_str()) {
                    api_key = key.to_string();
                }
            }
        }
    }

    Ok(AppConfig {
        base_url,
        api_key,
        raw_toml,
    })
}

pub fn update_auth_json(path: &Path, api_key_name: &str, api_key: &str) -> Result<(), String> {
    let mut auth_obj = if path.exists() {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content)
            .unwrap_or_else(|_| serde_json::Map::new())
    } else {
        serde_json::Map::new()
    };

    auth_obj.insert(api_key_name.to_string(), serde_json::Value::String(api_key.to_string()));
    let content = serde_json::to_string_pretty(&auth_obj).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<(), String> {
    let dir = get_codex_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    let config_path = dir.join("config.toml");
    let auth_path = dir.join("auth.json");
    let backup_dir = dir.join("backups");

    // Ensure backup directory exists
    if !backup_dir.exists() {
        std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();

    // Copy active config.toml to backup
    if config_path.exists() {
        let backup_config_path = backup_dir.join(format!("config.toml.{}.bak", timestamp));
        std::fs::copy(&config_path, &backup_config_path).map_err(|e| e.to_string())?;
    }

    // Copy active auth.json to backup
    if auth_path.exists() {
        let backup_auth_path = backup_dir.join(format!("auth.json.{}.bak", timestamp));
        std::fs::copy(&auth_path, &backup_auth_path).map_err(|e| e.to_string())?;
    }

    // Clean up trailing slash from base_url if present
    let clean_url = config.base_url.trim_end_matches('/').to_string();

    // Parse raw_toml to verify valid TOML structure and sync base_url and get env_key
    let mut doc = config.raw_toml.parse::<DocumentMut>().map_err(|e| format!("Invalid TOML: {}", e))?;
    
    let mut api_key_name = "OPENAI_API_KEY".to_string();
    let providers = doc.entry("model_providers").or_insert(toml_edit::table());
    if let Some(providers_table) = providers.as_table_mut() {
        let replynow_item = providers_table.entry("replynow").or_insert(toml_edit::table());
        if let Some(replynow_table) = replynow_item.as_table_mut() {
            replynow_table.insert("base_url", toml_edit::value(clean_url));
            if let Some(env) = replynow_table.get("env_key").and_then(|u| u.as_str()) {
                api_key_name = env.to_string();
            } else {
                replynow_table.insert("env_key", toml_edit::value(&api_key_name));
            }
        }
    }

    std::fs::write(&config_path, doc.to_string()).map_err(|e| e.to_string())?;
    update_auth_json(&auth_path, &api_key_name, &config.api_key)?;

    Ok(())
}

#[tauri::command]
fn restore_last_backup() -> Result<(), String> {
    let dir = get_codex_dir()?;
    let backup_dir = dir.join("backups");
    if !backup_dir.exists() {
        return Err("No backups found".to_string());
    }

    let entries = std::fs::read_dir(&backup_dir).map_err(|e| e.to_string())?;
    let mut backups = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("config.toml.") && name.ends_with(".bak") {
                // Extract the timestamp part
                let timestamp = name
                    .replace("config.toml.", "")
                    .replace(".bak", "");
                backups.push(timestamp);
            }
        }
    }

    if backups.is_empty() {
        return Err("No backups found".to_string());
    }

    // Sort so that the latest timestamp is last
    backups.sort();
    let latest_timestamp = backups.last().ok_or_else(|| "No backups found".to_string())?;

    let backup_config_name = format!("config.toml.{}.bak", latest_timestamp);
    let backup_auth_name = format!("auth.json.{}.bak", latest_timestamp);

    let backup_config_path = backup_dir.join(&backup_config_name);
    let backup_auth_path = backup_dir.join(&backup_auth_name);

    let config_path = dir.join("config.toml");
    let auth_path = dir.join("auth.json");

    // Restore config.toml
    if backup_config_path.exists() {
        std::fs::copy(&backup_config_path, &config_path).map_err(|e| e.to_string())?;
    }

    // Restore auth.json
    if backup_auth_path.exists() {
        std::fs::copy(&backup_auth_path, &auth_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn test_connection(base_url: String, api_key: String) -> Result<TestResult, String> {
    let clean_url = base_url.trim_end_matches('/').to_string();
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let test_url = format!("{}/models", clean_url);
    let start = std::time::Instant::now();

    let response_res = client
        .get(&test_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await;

    let duration = start.elapsed().as_millis();

    match response_res {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                Ok(TestResult {
                    success: true,
                    latency_ms: duration,
                    error: None,
                })
            } else if status.as_u16() == 404 {
                // Fallback to /chat/completions for custom endpoints that might not implement /models
                let completions_url = format!("{}/chat/completions", clean_url);
                let body = serde_json::json!({
                    "model": "gpt-3.5-turbo",
                    "messages": [{"role": "user", "content": "ping"}],
                    "max_tokens": 1
                });
                let start_fallback = std::time::Instant::now();
                let fallback_res = client
                    .post(&completions_url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&body)
                    .send()
                    .await;
                let duration_fallback = start_fallback.elapsed().as_millis();
                match fallback_res {
                    Ok(fallback_resp) => {
                        let fallback_status = fallback_resp.status();
                        if fallback_status.is_success() {
                            Ok(TestResult {
                                success: true,
                                latency_ms: duration_fallback,
                                error: None,
                            })
                        } else {
                            let body_text = fallback_resp.text().await.unwrap_or_default();
                            Ok(TestResult {
                                success: false,
                                latency_ms: duration_fallback,
                                error: Some(format!("HTTP Error {}: {}", fallback_status, body_text)),
                            })
                        }
                    }
                    Err(e) => {
                        Ok(TestResult {
                            success: false,
                            latency_ms: duration_fallback,
                            error: Some(format!("Fallback request failed: {}", e)),
                        })
                    }
                }
            } else {
                let body_text = resp.text().await.unwrap_or_default();
                Ok(TestResult {
                    success: false,
                    latency_ms: duration,
                    error: Some(format!("HTTP Error {}: {}", status, body_text)),
                })
            }
        }
        Err(e) => {
            Ok(TestResult {
                success: false,
                latency_ms: duration,
                error: Some(format!("Network connection failed: {}", e)),
            })
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_codex_status,
            initialize_codex,
            load_config,
            save_config,
            restore_last_backup,
            test_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_config_raw_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let auth_path = dir.path().join("auth.json");

        let raw_toml = r#"model_provider = "replynow"
model = "gpt-5.5"

[model_providers.replynow]
name = "replynow"
base_url = "https://api.replynow.cn:6688/v1"
env_key = "REPLYNOW_API_KEY"
"#;

        let config = AppConfig {
            base_url: "https://new-url.com/v1".to_string(),
            api_key: "sk-test-key".to_string(),
            raw_toml: raw_toml.to_string(),
        };

        // Write initial files manually to test path behavior
        std::fs::write(&config_path, raw_toml).unwrap();

        // Test the TOML parsing and update logic directly
        let mut doc = config.raw_toml.parse::<DocumentMut>().unwrap();
        let providers = doc.entry("model_providers").or_insert(toml_edit::table());
        let replynow_item = providers.as_table_mut().unwrap().entry("replynow").or_insert(toml_edit::table());
        replynow_item.as_table_mut().unwrap().insert("base_url", toml_edit::value("https://new-url.com/v1"));
        let env_key = replynow_item.as_table_mut().unwrap().get("env_key").unwrap().as_str().unwrap().to_string();
        assert_eq!(env_key, "REPLYNOW_API_KEY");

        std::fs::write(&config_path, doc.to_string()).unwrap();
        update_auth_json(&auth_path, &env_key, &config.api_key).unwrap();

        let updated_toml = std::fs::read_to_string(&config_path).unwrap();
        assert!(updated_toml.contains("base_url = \"https://new-url.com/v1\""));
        assert!(updated_toml.contains("model = \"gpt-5.5\""));

        let auth_content = std::fs::read_to_string(&auth_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&auth_content).unwrap();
        assert_eq!(json["REPLYNOW_API_KEY"], "sk-test-key");
    }

    #[test]
    fn test_update_auth_json() {
        let dir = tempdir().unwrap();
        let auth_path = dir.path().join("auth.json");

        update_auth_json(&auth_path, "OPENAI_API_KEY", "sk-test-key").unwrap();
        let content = std::fs::read_to_string(&auth_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["OPENAI_API_KEY"], "sk-test-key");
    }
}

