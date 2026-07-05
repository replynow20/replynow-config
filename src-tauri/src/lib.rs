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
    model: String,
    model_reasoning_effort: String,
    network_access: String,
    disable_response_storage: bool,
    model_verbosity: String,
    wire_api: String,
    requires_openai_auth: bool,
    api_key_name: String,
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
    if !config_path.exists() {
        let initial_content = r#"# OpenAI Codex CLI Configuration
# This file was initialized by ReplyNow Config GUI
"#;
        std::fs::write(&config_path, initial_content).map_err(|e| e.to_string())?;
    }
    
    let default_config = AppConfig {
        base_url: "https://api.replynow.cn:6688/v1".to_string(),
        api_key: "".to_string(),
        model: "gpt-5.5".to_string(),
        model_reasoning_effort: "high".to_string(),
        network_access: "enabled".to_string(),
        disable_response_storage: true,
        model_verbosity: "high".to_string(),
        wire_api: "responses".to_string(),
        requires_openai_auth: true,
        api_key_name: "OPENAI_API_KEY".to_string(),
    };
    update_config_toml(&config_path, &default_config)?;

    let auth_path = dir.join("auth.json");
    if !auth_path.exists() {
        std::fs::write(&auth_path, "{}").map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    let dir = get_codex_dir()?;
    let config_path = dir.join("config.toml");
    let auth_path = dir.join("auth.json");

    let mut base_url = "https://api.replynow.cn:6688/v1".to_string();
    let mut api_key = "".to_string();
    let mut model = "gpt-5.5".to_string();
    let mut model_reasoning_effort = "high".to_string();
    let mut network_access = "enabled".to_string();
    let mut disable_response_storage = true;
    let mut model_verbosity = "high".to_string();
    let mut wire_api = "responses".to_string();
    let mut requires_openai_auth = true;
    let mut api_key_name = "OPENAI_API_KEY".to_string();

    // Try to load values from config.toml
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        if let Ok(doc) = content.parse::<DocumentMut>() {
            // Root-level fields
            if let Some(m) = doc.get("model").and_then(|v| v.as_str()) {
                model = m.to_string();
            }
            if let Some(r) = doc.get("model_reasoning_effort").and_then(|v| v.as_str()) {
                model_reasoning_effort = r.to_string();
            }
            if let Some(n) = doc.get("network_access").and_then(|v| v.as_str()) {
                network_access = n.to_string();
            }
            if let Some(d) = doc.get("disable_response_storage").and_then(|v| v.as_bool()) {
                disable_response_storage = d;
            }
            if let Some(v) = doc.get("model_verbosity").and_then(|v| v.as_str()) {
                model_verbosity = v.to_string();
            }

            // model_providers table fields
            if let Some(providers) = doc.get("model_providers").and_then(|p| p.as_table()) {
                if let Some(replynow) = providers.get("replynow").and_then(|r| r.as_table()) {
                    if let Some(url) = replynow.get("base_url").and_then(|u| u.as_str()) {
                        base_url = url.to_string();
                    }
                    if let Some(w) = replynow.get("wire_api").and_then(|u| u.as_str()) {
                        wire_api = w.to_string();
                    }
                    if let Some(req) = replynow.get("requires_openai_auth").and_then(|u| u.as_bool()) {
                        requires_openai_auth = req;
                    }
                    if let Some(env) = replynow.get("env_key").and_then(|u| u.as_str()) {
                        api_key_name = env.to_string();
                    }
                }
            }
        }
    }

    // Try to load api_key from auth.json
    if auth_path.exists() {
        let content = std::fs::read_to_string(&auth_path).map_err(|e| e.to_string())?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(key) = json.get(&api_key_name).and_then(|k| k.as_str()) {
                api_key = key.to_string();
            } else if api_key_name == "OPENAI_API_KEY" {
                // If it fails to find OPENAI_API_KEY, fallback to REPLYNOW_API_KEY just in case they used v1 before
                if let Some(key) = json.get("REPLYNOW_API_KEY").and_then(|k| k.as_str()) {
                    api_key = key.to_string();
                }
            }
        }
    }

    Ok(AppConfig {
        base_url,
        api_key,
        model,
        model_reasoning_effort,
        network_access,
        disable_response_storage,
        model_verbosity,
        wire_api,
        requires_openai_auth,
        api_key_name,
    })
}

pub fn update_config_toml(path: &Path, config: &AppConfig) -> Result<(), String> {
    let content = if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        "".to_string()
    };
    let mut doc = content.parse::<DocumentMut>().map_err(|e| e.to_string())?;
    
    // Root level fields
    doc["model_provider"] = toml_edit::value("replynow");
    doc["model"] = toml_edit::value(&config.model);
    doc["model_reasoning_effort"] = toml_edit::value(&config.model_reasoning_effort);
    doc["network_access"] = toml_edit::value(&config.network_access);
    doc["disable_response_storage"] = toml_edit::value(config.disable_response_storage);
    doc["model_verbosity"] = toml_edit::value(&config.model_verbosity);
    
    // model_providers Table
    let providers = doc.entry("model_providers").or_insert(toml_edit::table());
    
    if let Some(providers_table) = providers.as_table_mut() {
        let replynow_item = providers_table.entry("replynow").or_insert(toml_edit::table());
        if let Some(replynow_table) = replynow_item.as_table_mut() {
            replynow_table.insert("name", toml_edit::value("replynow"));
            replynow_table.insert("base_url", toml_edit::value(&config.base_url));
            replynow_table.insert("env_key", toml_edit::value(&config.api_key_name));
            replynow_table.insert("wire_api", toml_edit::value(&config.wire_api));
            replynow_table.insert("requires_openai_auth", toml_edit::value(config.requires_openai_auth));
        }
    }
    
    std::fs::write(path, doc.to_string()).map_err(|e| e.to_string())?;
    Ok(())
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
    let mut clean_config = config.clone();
    clean_config.base_url = config.base_url.trim_end_matches('/').to_string();

    // Update files
    update_config_toml(&config_path, &clean_config)?;
    update_auth_json(&auth_path, &clean_config.api_key_name, &clean_config.api_key)?;

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
    fn test_update_config_toml_new_and_existing() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let mut config = AppConfig {
            base_url: "https://api.replynow.cn:6688/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-5.5".to_string(),
            model_reasoning_effort: "high".to_string(),
            network_access: "enabled".to_string(),
            disable_response_storage: true,
            model_verbosity: "high".to_string(),
            wire_api: "responses".to_string(),
            requires_openai_auth: true,
            api_key_name: "OPENAI_API_KEY".to_string(),
        };

        // Test with new (non-existing) file
        update_config_toml(&config_path, &config).unwrap();
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("model_provider = \"replynow\""));
        assert!(content.contains("base_url = \"https://api.replynow.cn:6688/v1\""));
        assert!(content.contains("model = \"gpt-5.5\""));
        assert!(content.contains("model_reasoning_effort = \"high\""));

        // Add an unrelated comment and field to mock user customizations
        let modified_content = format!(
            "{}\n# Custom User Setting\ncustom_field = \"hello\"\n",
            content
        );
        std::fs::write(&config_path, modified_content).unwrap();

        // Run update again with different URL and model
        config.base_url = "https://example.com/v1".to_string();
        config.model = "gpt-6.0".to_string();
        update_config_toml(&config_path, &config).unwrap();
        let content2 = std::fs::read_to_string(&config_path).unwrap();
        assert!(content2.contains("model_provider = \"replynow\""));
        assert!(content2.contains("base_url = \"https://example.com/v1\""));
        assert!(content2.contains("model = \"gpt-6.0\""));
        // Verify custom comment and custom_field are preserved
        assert!(content2.contains("# Custom User Setting"));
        assert!(content2.contains("custom_field = \"hello\""));
    }

    #[test]
    fn test_update_auth_json() {
        let dir = tempdir().unwrap();
        let auth_path = dir.path().join("auth.json");

        // Test with new file
        update_auth_json(&auth_path, "OPENAI_API_KEY", "sk-test-key").unwrap();
        let content = std::fs::read_to_string(&auth_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["OPENAI_API_KEY"], "sk-test-key");

        // Test preserving existing keys
        let mut map = serde_json::Map::new();
        map.insert("OTHER_KEY".to_string(), serde_json::Value::String("other-val".to_string()));
        map.insert("OPENAI_API_KEY".to_string(), serde_json::Value::String("sk-old".to_string()));
        std::fs::write(&auth_path, serde_json::to_string(&map).unwrap()).unwrap();

        update_auth_json(&auth_path, "OPENAI_API_KEY", "sk-new").unwrap();
        let content2 = std::fs::read_to_string(&auth_path).unwrap();
        let json2: serde_json::Value = serde_json::from_str(&content2).unwrap();
        assert_eq!(json2["OPENAI_API_KEY"], "sk-new");
        assert_eq!(json2["OTHER_KEY"], "other-val");
    }
}
