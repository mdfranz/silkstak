use std::collections::HashMap;
use std::path::PathBuf;

use compact_str::CompactString;

use crate::config::{Config, EditSystem, QuickModelConfig, ShowToolDetails};
#[cfg(feature = "mcp")]
use crate::extras::mcp::config::McpServerConfig;
use crate::session::storage;

fn resolve_config_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("ZS_CONFIG_DIR") {
        let dir = PathBuf::from(dir);
        let toml = dir.join("config.toml");
        let json = dir.join("config.json");
        if toml.exists() {
            return toml;
        }
        if json.exists() {
            return json;
        }
        return toml;
    }

    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("zerostack");
        let toml = dir.join("config.toml");
        let json = dir.join("config.json");
        if toml.exists() {
            return toml;
        }
        if json.exists() {
            return json;
        }
    }

    let dir = storage::data_dir();
    let toml = dir.join("config.toml");
    let json = dir.join("config.json");
    if toml.exists() {
        toml
    } else if json.exists() {
        json
    } else {
        toml
    }
}

pub fn config_file_path() -> PathBuf {
    resolve_config_path()
}

fn default_quick_models() -> HashMap<String, QuickModelConfig> {
    let mut map = HashMap::new();

    // Anthropic
    map.insert(
        "haiku".to_string(),
        QuickModelConfig {
            provider: CompactString::new("anthropic"),
            model: CompactString::new("claude-haiku-4-5"),
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        },
    );
    map.insert(
        "sonnet".to_string(),
        QuickModelConfig {
            provider: CompactString::new("anthropic"),
            model: CompactString::new("claude-sonnet-4-6"),
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        },
    );

    // OpenAI
    map.insert(
        "gpt-mini".to_string(),
        QuickModelConfig {
            provider: CompactString::new("openai"),
            model: CompactString::new("gpt-5-mini"),
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        },
    );
    map.insert(
        "gpt".to_string(),
        QuickModelConfig {
            provider: CompactString::new("openai"),
            model: CompactString::new("gpt-5"),
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        },
    );

    // Gemini
    map.insert(
        "gemini-flash".to_string(),
        QuickModelConfig {
            provider: CompactString::new("gemini"),
            model: CompactString::new("gemini-3.5-flash"),
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        },
    );
    map.insert(
        "gemini-pro".to_string(),
        QuickModelConfig {
            provider: CompactString::new("gemini"),
            model: CompactString::new("gemini-2.5-pro"),
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        },
    );

    map
}

pub fn quick_models_map(cfg: &Config) -> HashMap<String, QuickModelConfig> {
    let mut map = default_quick_models();
    if let Some(user_models) = &cfg.quick_models {
        for (k, v) in user_models {
            map.insert(k.clone(), v.clone());
        }
    }
    map
}

pub fn save_provider_and_model(provider: &str, model: &str) -> std::io::Result<()> {
    let path = resolve_config_path();
    let mut cfg: Config = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).unwrap_or_default(),
            _ => serde_json::from_str(&content).unwrap_or_default(),
        }
    } else {
        Config::default()
    };

    cfg.provider = Some(CompactString::new(provider));
    cfg.model = Some(CompactString::new(model));

    // If this is a known custom provider, also update its model field so that
    // default_model_for_provider will return the right value on the next start.
    if let Some(ref mut providers) = cfg.custom_providers {
        if let Some(entry) = providers.get_mut(provider) {
            entry.model = Some(CompactString::new(model));
        }
    }

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid config path")
    })?;
    std::fs::create_dir_all(parent)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => {
            let content = toml::to_string(&cfg).map_err(std::io::Error::other)?;
            std::fs::write(&path, content)?;
        }
        _ => std::fs::write(&path, serde_json::to_string_pretty(&cfg)?)?,
    }
    Ok(())
}

fn save_configure_data_to(
    path: &std::path::Path,
    api_keys: &[(String, String)],
    active_provider: &str,
    active_model: &str,
) -> std::io::Result<()> {
    let mut cfg: Config = if path.exists() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).unwrap_or_default(),
            _ => serde_json::from_str(&content).unwrap_or_default(),
        }
    } else {
        Config::default()
    };

    if !api_keys.is_empty() {
        let keys_map = cfg.api_keys.get_or_insert_with(HashMap::new);
        for (provider, key) in api_keys {
            keys_map.insert(provider.clone(), key.clone());
        }
    }

    cfg.provider = Some(CompactString::new(active_provider));
    cfg.model = Some(CompactString::new(active_model));

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid config path")
    })?;
    std::fs::create_dir_all(parent)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => {
            let content = toml::to_string(&cfg).map_err(std::io::Error::other)?;
            std::fs::write(path, content)?;
        }
        _ => std::fs::write(path, serde_json::to_string_pretty(&cfg)?)?,
    }
    Ok(())
}

pub fn save_configure_data(
    api_keys: &[(String, String)],
    active_provider: &str,
    active_model: &str,
) -> std::io::Result<()> {
    save_configure_data_to(
        &resolve_config_path(),
        api_keys,
        active_provider,
        active_model,
    )
}

pub fn save_quick_model(
    name: &str,
    provider: &str,
    model: &str,
    input_token_cost: f64,
    output_token_cost: f64,
) -> std::io::Result<()> {
    let path = resolve_config_path();
    let mut cfg: Config = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).unwrap_or_default(),
            _ => serde_json::from_str(&content).unwrap_or_default(),
        }
    } else {
        Config::default()
    };

    let quick_models = cfg.quick_models.get_or_insert_with(HashMap::new);
    quick_models.insert(
        name.to_string(),
        QuickModelConfig {
            provider: CompactString::new(provider),
            model: CompactString::new(model),
            input_token_cost,
            output_token_cost,
        },
    );

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid config path")
    })?;
    std::fs::create_dir_all(parent)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => {
            let content = toml::to_string(&cfg).map_err(std::io::Error::other)?;
            std::fs::write(&path, content)?;
        }
        _ => std::fs::write(&path, serde_json::to_string_pretty(&cfg)?)?,
    }
    Ok(())
}

fn rich_default_config() -> Config {
    let mut cfg = Config::default();
    cfg.quick_models = Some(default_quick_models());
    cfg.provider = Some(CompactString::new("auto"));
    cfg.max_tokens = Some(16384);
    cfg.context_window = Some(128_000);
    cfg.compact_enabled = Some(true);
    cfg.max_text_file_size = Some(1_048_576);
    cfg.edit_system = Some(EditSystem::Similarity);
    cfg.default_permission_mode = Some("standard".to_string());
    cfg.default_prompt = Some(CompactString::new("code"));
    cfg.show_tool_details = Some(ShowToolDetails::Lines(1));
    cfg.subagent_model = Some(CompactString::new("haiku"));
    #[cfg(feature = "subagents")]
    {
        cfg.subagent_max_read_lines = Some(2000);
        cfg.subagent_max_grep_results = Some(200);
        cfg.subagent_max_find_results = Some(200);
    }
    cfg
}

pub fn load() -> Config {
    let path = resolve_config_path();
    #[allow(unused_mut)]
    let mut cfg: Config = if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let default = rich_default_config();
        if path.extension().and_then(|e| e.to_str()) == Some("toml")
            && let Ok(content) = toml::to_string(&default)
        {
            std::fs::write(&path, content).ok();
        }
        default
    } else {
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!(
                "error: failed to read config file ({}): {}\n\
                 Fix the file or remove it to use defaults.",
                path.display(),
                e,
            );
            std::process::exit(1);
        });
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!(
                    "error: {} is not a valid config: {}\n\
                     Fix the file or remove it to use defaults.",
                    path.display(),
                    e,
                );
                std::process::exit(1);
            }),
            _ => serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!(
                    "error: {} is not a valid config: {}\n\
                     Fix the file or remove it to use defaults.",
                    path.display(),
                    e,
                );
                std::process::exit(1);
            }),
        }
    };

    #[cfg(feature = "mcp")]
    if cfg.mcp_servers.is_none() {
        let mut headers = HashMap::new();
        if let Ok(key) = std::env::var("EXA_API_KEY") {
            headers.insert("x-api-key".to_string(), key);
        }
        let mut defaults = HashMap::new();
        defaults.insert(
            "Exa Web Search".to_string(),
            McpServerConfig::Url {
                url: "https://mcp.exa.ai/mcp".to_string(),
                headers,
            },
        );
        cfg.mcp_servers = Some(defaults);
    }

    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_toml(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zs_configure_test_{tag}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("config.toml")
    }

    #[test]
    fn configure_creates_new_toml_with_provider_and_model() {
        let path = tmp_toml("new");
        let _ = std::fs::remove_file(&path);
        save_configure_data_to(&path, &[], "anthropic", "claude-sonnet-4-6").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let cfg: Config = toml::from_str(&content).unwrap();
        assert_eq!(cfg.provider.as_deref(), Some("anthropic"));
        assert_eq!(cfg.model.as_deref(), Some("claude-sonnet-4-6"));
        assert!(cfg.api_keys.is_none());
    }

    #[test]
    fn configure_saves_api_keys_into_map() {
        let path = tmp_toml("apikeys");
        let _ = std::fs::remove_file(&path);
        let keys = vec![
            ("anthropic".to_string(), "sk-ant-test".to_string()),
            ("openai".to_string(), "sk-openai-test".to_string()),
        ];
        save_configure_data_to(&path, &keys, "anthropic", "claude-sonnet-4-6").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let cfg: Config = toml::from_str(&content).unwrap();
        let api_keys = cfg.api_keys.unwrap();
        assert_eq!(
            api_keys.get("anthropic").map(|s| s.as_str()),
            Some("sk-ant-test")
        );
        assert_eq!(
            api_keys.get("openai").map(|s| s.as_str()),
            Some("sk-openai-test")
        );
    }

    #[test]
    fn configure_merges_into_existing_config() {
        let path = tmp_toml("merge");
        let _ = std::fs::remove_file(&path);
        let existing = Config {
            max_tokens: Some(8192),
            ..Default::default()
        };
        std::fs::write(&path, toml::to_string(&existing).unwrap()).unwrap();
        save_configure_data_to(&path, &[], "openai", "gpt-4o").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let cfg: Config = toml::from_str(&content).unwrap();
        assert_eq!(cfg.max_tokens, Some(8192));
        assert_eq!(cfg.provider.as_deref(), Some("openai"));
        assert_eq!(cfg.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn configure_overwrites_existing_api_key() {
        let path = tmp_toml("overwrite_key");
        let _ = std::fs::remove_file(&path);
        let keys1 = vec![("anthropic".to_string(), "old-key".to_string())];
        save_configure_data_to(&path, &keys1, "anthropic", "claude-haiku-4-5").unwrap();
        let keys2 = vec![("anthropic".to_string(), "new-key".to_string())];
        save_configure_data_to(&path, &keys2, "anthropic", "claude-haiku-4-5").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let cfg: Config = toml::from_str(&content).unwrap();
        let api_keys = cfg.api_keys.unwrap();
        assert_eq!(
            api_keys.get("anthropic").map(|s| s.as_str()),
            Some("new-key")
        );
    }
}
