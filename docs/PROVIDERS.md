# Providers

zerostack supports four built-in providers and allows custom provider
definitions for OpenAI-compatible endpoints.

## Built-in Providers

| Provider  | Config name         | Default env var for API key |
| --------- | ------------------- | --------------------------- |
| Anthropic | `anthropic`         | `ANTHROPIC_API_KEY`         |
| OpenAI    | `openai`            | `OPENAI_API_KEY`            |
| Gemini    | `gemini` / `google` | `GEMINI_API_KEY`            |
| Ollama    | `ollama`            | (no key required)           |

Select a provider via the config file, the `--provider` CLI flag, or the
`ZS_PROVIDER` environment variable:

```
zerostack --provider anthropic
```

The model is set with `--model` or `ZS_MODEL`:

```
zerostack --provider openai --model gpt-4o
```

## Custom Providers

Custom providers let you point zerostack at any OpenAI-compatible API (vLLM,
LiteLLM, Ollama, local models, enterprise gateways, etc.). Define them under
the `custom_providers` key in the config file:

```json
{
  "custom_providers": {
    "local-vllm": {
      "provider_type": "openai",
      "base_url": "http://localhost:8000/v1",
      "api_key_env": "VLLM_API_KEY",
      "model": "gemma4"
    },
    "company-gateway": {
      "provider_type": "openai",
      "base_url": "https://gateway.example.com/v1",
      "model": "glm"
    }
  }
}
```

| Field                         | Type    | Description                                                                                                                                                                   |
| ----------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `provider_type`               | string  | Must be one of the built-in provider types (`openai`, `anthropic`, `gemini`, `ollama`).                                                                                       |
| `base_url`                    | string  | The API base URL.                                                                                                                                                             |
| `api_key_env`                 | string  | Optional. Name of an environment variable holding the API key. Falls back to the provider-kind default if not set.                                                            |
| `api_style`                   | string  | Optional. For OpenAI-based providers: `"responses"` (Responses API, default when no `base_url` is set) or `"completions"` (Chat Completions, default when `base_url` is set). |
| `headers`                     | object  | Optional. HTTP headers to include in every request. Values support `${ENV_VAR}` expansion.                                                                                    |
| `danger_accept_invalid_certs` | boolean | Optional. Disables TLS certificate verification (MITM risk — use with care).                                                                                                  |
| `timeout_secs`                | integer | Optional. Overrides the default HTTP timeout.                                                                                                                                 |
| `model`                       | string  | Optional. Default model name for this provider. Used when no model is specified via `--model` or `ZS_MODEL`.                                                                  |

### Header variable expansion

Header values can reference environment variables with `${VAR}` syntax:

```json
{
  "custom_providers": {
    "company-gateway": {
      "provider_type": "openai",
      "base_url": "https://gateway.example.com/v1",
      "headers": {
        "cf-access-client-id": "${CF_ACCESS_CLIENT_ID}",
        "cf-access-client-secret": "${CF_ACCESS_CLIENT_SECRET}"
      }
    }
  }
}
```

## API Key Resolution

The API key is resolved in this priority order:

1. **CLI flag** `--api-key` (visible in process listings — use with care)
2. **Environment variable** — either the custom one from `api_key_env`, or the
   default env var for the provider kind
3. **Config file** `api_keys` map — keyed by provider slug or custom provider name
4. **Ollama** — returns an empty string (no key required)

### Config-level API keys

```json
{
  "api_keys": {
    "openai": "sk-...",
    "anthropic": "sk-ant-..."
  }
}
```

## OpenAI API Styles

The OpenAI provider supports two API transports:

- **Responses API** (`/responses`) — the default for OpenAI's own API. Required
  for GPT-5-series models that reject `max_tokens` on Chat Completions.
- **Chat Completions API** (`/chat/completions`) — the default when a custom
  `base_url` is set, since most OpenAI-compatible gateways implement only this
  endpoint.

Override with `api_style: "responses"` or `api_style: "completions"` on a
custom provider, or set `api_style` on the built-in OpenAI provider to force a
specific transport.

## Prompt caching

zerostack enables prompt caching automatically where the underlying rig provider supports it. The behavior depends on which provider backs the model you choose.

### Automatic — no zerostack action

These providers cache server-side without any markers in the request:

- **OpenAI** — automatic above ~1024 tokens, 50% discount on cached input.
- **Google Gemini 2.5+** — implicit caching, 75% discount.

For these providers, zerostack passes through to rig without additional configuration.

### Explicitly enabled by zerostack

Anthropic requires `cache_control` markers; zerostack adds them via rig's `.with_prompt_caching()`:

- **Anthropic** — marks system prompt, the final tool definition, and the last message. All three breakpoints contribute to cumulative savings as the conversation grows.

### Empirical impact

Measured on Sonnet 4.6, second turn of a tool-heavy session (grep + read across the zerostack repo, ~6k-token system prompt including AGENTS.md and ARCHITECTURE.md):

| Configuration         | turn 2 cost | reduction |
| --------------------- | ----------: | --------: |
| Baseline (no caching) |      $0.186 |         — |
| Anthropic + caching   |      $0.024 |      -87% |

Projected monthly cost at 50 such turns per working day: $204 (baseline) → $26 (Anthropic direct).

### DeepSeek via custom provider

DeepSeek provides automatic caching (persistent across days, ~90% discount) and is accessible via a custom provider pointing at the DeepSeek API:

```toml
[custom_providers.deepseek]
provider_type = "openai"
base_url = "https://api.deepseek.com"
api_key_env = "DEEPSEEK_API_KEY"
```

Then use it with `--provider deepseek --model deepseek-chat`.

## CLI Flags

| Flag                | Env var       | Description                         |
| ------------------- | ------------- | ----------------------------------- |
| `--provider`        | `ZS_PROVIDER` | Provider name                       |
| `--model`           | `ZS_MODEL`    | Model name                          |
| `--quick-model`     | —             | Use a named quick model from config |
| `--api-key`         | —             | API key (visible in `ps`)           |
| `--max-tokens`      | —             | Maximum response tokens             |
| `--temperature`     | —             | Model temperature (0.0–2.0)         |
| `--max-agent-turns` | —             | Maximum agent turns per response    |
