use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::cli::Cli;
use crate::config::{self, Config, QuickModelConfig};
use crate::provider::{AnyClient, ModelEntry, list_models_manual};
use crate::ui::slash::{SlashCtx, write_error, write_ok, write_result};

pub async fn handle(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    match parts[0] {
        "/provider" => handle_provider(parts, ctx).await,
        "/model" | "/models" => handle_models(parts, ctx).await,
        "/models-add" => handle_models_add(parts, ctx).await,
        #[cfg(feature = "subagents")]
        "/model-subagent" => handle_model_subagent(parts, ctx).await,
        #[cfg(feature = "subagents")]
        "/models-subagent" => handle_models_subagent(parts, ctx).await,
        _ => Ok(()),
    }
}

static MODEL_CACHE: LazyLock<Mutex<HashMap<String, Arc<[ModelEntry]>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Returns the provider's models.
///
/// Network is only touched on `refresh`, for custom gateways, or for built-in
/// providers that aren't baked (e.g. ollama). Baked built-ins are served from
/// the embedded catalog with no network call — this is what keeps startup instant.
pub(crate) async fn fetch_models_cached(
    provider: &str,
    is_custom: bool,
    client: &AnyClient,
    cli: &Cli,
    cfg: &Config,
    refresh: bool,
) -> anyhow::Result<Arc<[ModelEntry]>> {
    if !refresh {
        if let Some(hit) = MODEL_CACHE.lock().unwrap().get(provider) {
            return Ok(Arc::clone(hit)); // guard dropped here, NOT across any await
        }
        // No cache yet: serve the baked catalog for built-in providers — no network.
        if !is_custom && let Some(entries) = crate::models_catalog::catalog_entries(provider) {
            let models: Vec<ModelEntry> = entries
                .iter()
                .filter(|m| crate::provider::is_agent_model(m))
                .cloned()
                .collect();
            let arc: Arc<[ModelEntry]> = Arc::from(models.into_boxed_slice());
            MODEL_CACHE
                .lock()
                .unwrap()
                .insert(provider.to_string(), Arc::clone(&arc));
            return Ok(arc);
        }
    }
    let mut models = if is_custom {
        list_models_manual(
            provider,
            cli.api_key.as_deref(),
            &cfg.custom_providers_map(),
            cfg.api_keys.as_ref(),
        )
        .await?
    } else {
        client.list_models().await?
    };
    models.retain(crate::provider::is_agent_model);
    let arc: Arc<[ModelEntry]> = Arc::from(models.into_boxed_slice());
    MODEL_CACHE
        .lock()
        .unwrap()
        .insert(provider.to_string(), Arc::clone(&arc));
    Ok(arc)
}

/// sync read for the picker (no await)
pub(crate) fn cached_model_ids(provider: &str) -> Vec<String> {
    MODEL_CACHE
        .lock()
        .unwrap()
        .get(provider)
        .map(|v| v.iter().map(|m| m.id.clone()).collect())
        .unwrap_or_default()
}

/// best-effort warm; returns id list (empty on failure, never errors)
pub(crate) async fn warm_model_cache(
    provider: &str,
    is_custom: bool,
    client: &AnyClient,
    cli: &Cli,
    cfg: &Config,
) -> Vec<String> {
    let _ = fetch_models_cached(provider, is_custom, client, cli, cfg, false).await;
    cached_model_ids(provider)
}

async fn apply_model(ctx: &mut SlashCtx<'_>, model_id: &str) {
    let new_model = compact_str::CompactString::new(model_id);
    let model = ctx.client.completion_model(new_model.to_string());
    let is_reasoning = crate::provider::is_reasoning_model(model_id);
    *ctx.agent = Some(
        crate::provider::build_agent(
            model,
            ctx.cli,
            ctx.cfg,
            ctx.context,
            ctx.permission.clone(),
            ctx.ask_tx.clone(),
            ctx.sandbox.clone(),
            *ctx.reasoning_enabled,
            is_reasoning,
            #[cfg(feature = "mcp")]
            ctx.mcp_manager,
        )
        .await,
    );
    ctx.session.model = new_model.clone();
    let _ = config::save_provider_and_model(&ctx.session.provider, model_id);
    write_ok(ctx.renderer, format!("switched to model: {}", new_model));
}

async fn handle_provider(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    if parts.len() < 2 {
        write_ok(
            ctx.renderer,
            format!("current provider: {}", ctx.session.provider),
        );
        return Ok(());
    }
    let new_provider = parts[1].trim();
    if crate::provider::parse_provider(new_provider).is_none()
        && !ctx.cfg.custom_providers_map().contains_key(new_provider)
    {
        write_error(
            ctx.renderer,
            format!("unknown provider: '{}'", new_provider),
        );
        return Ok(());
    }
    // Default the model to something valid for the new provider BEFORE rebuilding,
    // since rebuild_agent_with_client reads session.model. Otherwise the old id
    // (e.g. an OpenRouter id) is carried onto a provider where it is invalid.
    if let Some(model) = crate::provider::default_model_for_provider(new_provider, ctx.cfg) {
        ctx.session.model = compact_str::CompactString::new(&model);
    }
    ctx.rebuild_agent_with_client(new_provider, *ctx.reasoning_enabled)
        .await?;
    ctx.session.provider = compact_str::CompactString::new(new_provider);
    let _ = config::save_provider_and_model(new_provider, &ctx.session.model);
    write_ok(
        ctx.renderer,
        format!(
            "switched to provider: {} (model: {})",
            new_provider, ctx.session.model
        ),
    );
    Ok(())
}

async fn handle_models(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    let qm = config::quick_models_map(ctx.cfg);
    let provider = ctx.session.provider.to_string();
    let is_custom = ctx.cfg.custom_providers_map().contains_key(&provider);

    let arg1 = parts.get(1).map(|s| s.trim()).unwrap_or("");
    let show_all = arg1 == "all";
    let show_list = arg1 == "list";
    let refresh = arg1 == "refresh";

    // /models without arguments — launch interactive model picker directly
    if parts.len() < 2 {
        if fetch_models_cached(&provider, is_custom, ctx.client, ctx.cli, ctx.cfg, false)
            .await
            .is_ok()
        {
            ctx.input.set_live_model_names(cached_model_ids(&provider));
        }
        ctx.input.buffer = "/models ".into();
        ctx.input.cursor = 8;
        ctx.input.start_models_picker();
        return Ok(());
    }

    // /models <name-or-id> — quick-model alias first, then raw model id
    if parts.len() >= 2 && !show_all && !show_list && !refresh {
        if let Some(q) = qm.get(arg1) {
            ctx.rebuild_agent_with_client(&q.provider, *ctx.reasoning_enabled)
                .await?;
            apply_model(ctx, &q.model).await;
            ctx.session.provider = compact_str::CompactString::new(&q.provider);
            write_result(
                ctx.renderer,
                format!("  quick model {} — {} / {}", arg1, q.provider, q.model),
            );
            return Ok(());
        }
        apply_model(ctx, arg1).await;
        return Ok(());
    }

    // Warm the cache so the picker has live names; get the count for the hint.
    let available_count = match fetch_models_cached(
        &provider, is_custom, ctx.client, ctx.cli, ctx.cfg, refresh,
    )
    .await
    {
        Ok(models) => {
            ctx.input.set_live_model_names(cached_model_ids(&provider));
            if refresh {
                write_result(
                    ctx.renderer,
                    format!(
                        "model list refreshed — {} models from {}",
                        models.len(),
                        provider
                    ),
                );
                return Ok(());
            }
            if show_list {
                // Full dump requested explicitly.
                write_ok(
                    ctx.renderer,
                    format!("available from {} ({}):", provider, models.len()),
                );
                for m in models.iter() {
                    let ctx_win = m
                        .context_length
                        .map(|c| format!("  [{}k ctx]", c / 1000))
                        .unwrap_or_default();
                    let label = if m.display == m.id {
                        m.id.clone()
                    } else {
                        format!("{} ({})", m.display, m.id)
                    };
                    write_result(ctx.renderer, format!("  {}{}", label, ctx_win));
                }
                return Ok(());
            }
            models.len()
        }
        Err(e) => {
            tracing::debug!("model listing failed for {}: {}", provider, e);
            if refresh {
                write_error(ctx.renderer, format!("model list refresh failed: {}", e));
                return Ok(());
            }
            0
        }
    };

    // Default listing: current model, then quick models for this provider.
    write_ok(
        ctx.renderer,
        format!("current model: {}  [{}]", ctx.session.model, provider),
    );

    let mut sorted: Vec<&String> = qm.keys().collect();
    sorted.sort();
    let provider_qm: Vec<(&String, &QuickModelConfig)> = if show_all {
        write_result(ctx.renderer, "all quick aliases:".to_string());
        sorted.iter().map(|n| (*n, &qm[*n])).collect()
    } else {
        let list: Vec<(&String, &QuickModelConfig)> = sorted
            .iter()
            .filter(|n| qm[**n].provider.as_str() == provider)
            .map(|n| (*n, &qm[*n]))
            .collect();
        if !list.is_empty() {
            write_result(ctx.renderer, "quick aliases:".to_string());
        }
        list
    };

    if provider_qm.is_empty() && !show_all {
        write_result(
            ctx.renderer,
            format!(
                "  no quick aliases for {} — /models all or /models-add <name> {} <model>",
                provider, provider
            ),
        );
    } else {
        let name_w = provider_qm.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
        for (name, q) in &provider_qm {
            let active = if q.model.as_str() == ctx.session.model.as_str() {
                " ←"
            } else {
                ""
            };
            if show_all {
                write_result(
                    ctx.renderer,
                    format!("  {name:<name_w$}  {}/{}{}", q.provider, q.model, active),
                );
            } else {
                write_result(
                    ctx.renderer,
                    format!("  {name:<name_w$}  {}{}", q.model, active),
                );
            }
        }
    }

    if available_count > 0 {
        write_result(
            ctx.renderer,
            format!(
                "  {} total models — type '/models ' to open the picker (Left/Right to switch tabs)",
                available_count
            ),
        );
    }

    Ok(())
}

async fn handle_models_add(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    if parts.len() < 3 {
        write_ok(ctx.renderer, "usage: /models-add <name> <provider> <model>");
        return Ok(());
    }
    let name = parts[1].trim().to_string();
    let rest = parts[2].trim();
    let (provider, model) = match rest.split_once(' ') {
        Some((p, m)) => (p.trim().to_string(), m.trim().to_string()),
        None => {
            write_ok(ctx.renderer, "usage: /models-add <name> <provider> <model>");
            return Ok(());
        }
    };
    if name.is_empty() || provider.is_empty() || model.is_empty() {
        write_ok(ctx.renderer, "usage: /models-add <name> <provider> <model>");
        return Ok(());
    }
    match config::save_quick_model(&name, &provider, &model) {
        Ok(()) => {
            write_ok(
                ctx.renderer,
                format!("saved quick model: {} ({} / {})", name, provider, model),
            );
        }
        Err(e) => {
            write_error(ctx.renderer, format!("failed to save quick model: {}", e));
        }
    }
    Ok(())
}

#[cfg(feature = "subagents")]
async fn handle_model_subagent(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    use crate::extras::subagents;

    if parts.len() < 2 {
        let (provider_name, model_name) =
            subagents::with_config(|cfg| (cfg.client.provider_name(), cfg.model_name.clone()));
        write_ok(
            ctx.renderer,
            format!("current subagent model: {} / {}", provider_name, model_name),
        );
        return Ok(());
    }

    let new_model = parts[1].trim().to_string();
    let model = ctx.client.completion_model(new_model.clone());
    model_for_subagent(ctx, model).await?;
    subagents::set_model_name(new_model.clone());
    write_ok(
        ctx.renderer,
        format!("switched subagent to model: {}", new_model),
    );
    Ok(())
}

#[cfg(feature = "subagents")]
async fn handle_models_subagent(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    use crate::extras::subagents;

    let qm = config::quick_models_map(ctx.cfg);
    let mut sorted: Vec<&String> = qm.keys().collect();
    sorted.sort();

    if parts.len() < 2 {
        let (provider_name, model_name) =
            subagents::with_config(|cfg| (cfg.client.provider_name(), cfg.model_name.clone()));
        if sorted.is_empty() {
            write_ok(
                ctx.renderer,
                format!(
                    "current subagent: {} / {} (no quick models defined)",
                    provider_name, model_name
                ),
            );
        } else {
            write_ok(
                ctx.renderer,
                format!(
                    "quick models (current subagent: {} | {}):",
                    provider_name, model_name
                ),
            );
            for name in &sorted {
                let q = &qm[name.as_str()];
                write_result(
                    ctx.renderer,
                    format!("  {}  ({} / {})", name, q.provider, q.model),
                );
            }
        }
        return Ok(());
    }

    let name = parts[1].trim();
    if let Some(q) = qm.get(name) {
        if q.provider.as_str() != ctx.client.provider_name() {
            let new_client = crate::provider::create_client(
                &q.provider,
                ctx.cli.api_key.as_deref(),
                &ctx.cfg.custom_providers_map(),
                ctx.cfg.api_keys.as_ref(),
            )?;
            let model = new_client.completion_model(q.model.to_string());
            model_for_subagent(ctx, model).await?;
            subagents::set_client_and_model(new_client, q.model.to_string());
        } else {
            let model = ctx.client.completion_model(q.model.to_string());
            model_for_subagent(ctx, model).await?;
            subagents::set_model_name(q.model.to_string());
        }
        write_ok(
            ctx.renderer,
            format!(
                "switched subagent to quick model: {} ({} / {})",
                name, q.provider, q.model
            ),
        );
    } else {
        write_error(ctx.renderer, format!("unknown quick model: '{}'", name));
        if !sorted.is_empty() {
            write_ok(ctx.renderer, "available quick models:");
            for n in &sorted {
                write_result(ctx.renderer, format!("  {}", n));
            }
        }
    }
    Ok(())
}

/// Validate a model handle by trying to build a subagent with it.
/// If it fails, the error is shown but does not abort the command.
#[cfg(feature = "subagents")]
async fn model_for_subagent(
    ctx: &mut SlashCtx<'_>,
    model: crate::provider::AnyModel,
) -> anyhow::Result<()> {
    let max_turns = ctx.cfg.task_max_turns.unwrap_or(20);
    let _agent =
        crate::extras::subagents::builder::build_explore_agent(model, max_turns, ctx.cfg).await;
    Ok(())
}
