use std::io::{self, Write};

use crate::config;

struct ProviderInfo {
    name: &'static str,
    env_var: &'static str,
    default_model: &'static str,
}

const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        name: "anthropic",
        env_var: "ANTHROPIC_API_KEY",
        default_model: "claude-sonnet-4-6",
    },
    ProviderInfo {
        name: "openai",
        env_var: "OPENAI_API_KEY",
        default_model: "gpt-4o",
    },
    ProviderInfo {
        name: "gemini",
        env_var: "GEMINI_API_KEY",
        default_model: "gemini-2.5-pro",
    },
    ProviderInfo {
        name: "ollama",
        env_var: "",
        default_model: "llama3.2",
    },
];

fn prompt(msg: &str) -> io::Result<String> {
    print!("{msg}");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn key_status(provider_name: &str, env_var: &str, cfg: &config::Config) -> &'static str {
    let in_env = std::env::var(env_var)
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    let in_cfg = cfg
        .api_keys
        .as_ref()
        .and_then(|m| m.get(provider_name))
        .map(|k| !k.is_empty())
        .unwrap_or(false);
    if in_env {
        "set via env"
    } else if in_cfg {
        "set in config"
    } else {
        "not set"
    }
}

struct ProviderSetup {
    name: String,
    api_key: Option<String>,
    model: String,
}

pub fn run_configure(cfg: &config::Config) -> anyhow::Result<()> {
    let config_path = config::config_file_path();
    println!("zerostack setup");
    println!("Config file: {}", config_path.display());
    println!();

    let mut setups: Vec<ProviderSetup> = Vec::new();

    for p in PROVIDERS {
        println!("--- {} ---", p.name);

        if p.env_var.is_empty() {
            // Ollama: no API key, just ask for the model
            let current_model = if cfg.provider.as_deref() == Some(p.name) {
                cfg.model.as_deref().unwrap_or(p.default_model)
            } else {
                p.default_model
            };
            let model_input = prompt(&format!("  Default model [{}]: ", current_model))?;
            let model = if model_input.is_empty() {
                current_model.to_string()
            } else {
                model_input
            };
            println!("  No API key required (local Ollama server).");
            println!();
            setups.push(ProviderSetup {
                name: p.name.to_string(),
                api_key: None,
                model,
            });
            continue;
        }

        let status = key_status(p.name, p.env_var, cfg);
        println!("  {} ({})", p.env_var, status);

        let hint = match status {
            "set via env" => {
                "env var takes priority; enter key to also save in config, or press Enter to skip"
            }
            "set in config" => {
                "currently saved in config; enter new key to replace, or press Enter to keep"
            }
            _ => "enter key to save, or press Enter to skip",
        };
        println!("  {hint}");
        let key = prompt("  API key: ")?;
        let api_key = if key.is_empty() { None } else { Some(key) };

        // Model default: preserve the existing config model only if this is the
        // currently-configured provider, otherwise show the built-in default.
        let current_model = if cfg.provider.as_deref() == Some(p.name) {
            cfg.model.as_deref().unwrap_or(p.default_model)
        } else {
            p.default_model
        };
        let model_input = prompt(&format!("  Default model [{}]: ", current_model))?;
        let model = if model_input.is_empty() {
            current_model.to_string()
        } else {
            model_input
        };

        println!();
        setups.push(ProviderSetup {
            name: p.name.to_string(),
            api_key,
            model,
        });
    }

    // Choose active provider
    let current_provider = cfg.provider.as_deref().unwrap_or("anthropic");
    let provider_input = prompt(&format!("Active provider [{}]: ", current_provider))?;
    let active_provider = if provider_input.is_empty() {
        current_provider.to_string()
    } else {
        provider_input
    };

    // Pre-fill model from what the user entered for this provider above
    let configured_model = setups
        .iter()
        .find(|s| s.name == active_provider)
        .map(|s| s.model.as_str());
    let active_model_default = configured_model.unwrap_or("claude-sonnet-4-6");
    let model_input = prompt(&format!("Active model [{}]: ", active_model_default))?;
    let active_model = if model_input.is_empty() {
        active_model_default.to_string()
    } else {
        model_input
    };

    // Collect API keys to save
    let collected_keys: Vec<(String, String)> = setups
        .iter()
        .filter_map(|s| s.api_key.as_ref().map(|k| (s.name.clone(), k.clone())))
        .collect();

    config::save_configure_data(&collected_keys, &active_provider, &active_model)?;

    println!();
    println!("Saved to: {}", config_path.display());
    println!("Active: {} / {}", active_provider, active_model);

    Ok(())
}
