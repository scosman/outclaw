---
status: complete
---

# Architecture: Gateway Integration Fixes

This project is small enough for a single architecture doc — no component designs needed.

## Files Changed

### Frontend (Svelte)

| File                                     | Change                                                                         |
| ---------------------------------------- | ------------------------------------------------------------------------------ |
| `src/lib/components/InstanceCard.svelte` | Remove gateway URL display, rename "Open" → "Details", navigate to detail page |
| `src/lib/types/instance.ts`              | Update `getGatewayUrl()` to append `?token=`                                   |

### Backend (Rust)

| File                                  | Change                                                      |
| ------------------------------------- | ----------------------------------------------------------- |
| `src-tauri/src/instance/models.rs`    | Update `gateway_url()` to append `?token=`                  |
| `src-tauri/src/commands/instances.rs` | Fix Stage 8 config-set race, add controlUi config-set calls |

## Detailed Changes

### 1. InstanceCard.svelte

Remove the gateway URL display block (the `{#if isRunning}` section showing `getGatewayUrl`). Change `handleOpen` to navigate to the detail page (same as `handleDetails`). Rename the button label from "Open" to "Details". Remove the now-unused `getGatewayUrl` import and the `open_in_browser` invoke import if no longer needed.

### 2. getGatewayUrl / gateway_url

**TypeScript** (`src/lib/types/instance.ts`):

```typescript
export function getGatewayUrl(config: InstanceConfig): string {
	return `http://localhost:${config.gateway_port}?token=${config.gateway_token}`;
}
```

**Rust** (`src-tauri/src/instance/models.rs`):

```rust
pub fn gateway_url(&self) -> String {
    format!("http://localhost:{}?token={}", self.gateway_port, self.gateway_token)
}
```

### 3. Stage 8: Config-Set Fix + controlUi

#### Investigation

The config-set CLI commands execute correctly and the bind value is `"lan"` when expected (confirmed via logging). But `openclaw.json` ends up with `"loopback"` after the build completes. Something overwrites the config between Stage 8 and the final state.

Investigation steps during implementation:

1. Read `openclaw.json` from host after Stage 8 completes (before Stage 9 restart) — does it have the correct `bind: "lan"`?
2. Read `openclaw.json` after Stage 9 restart — did the value revert?
3. If the restart overwrites: restructure so config-set runs after the gateway is stopped but before it restarts, or after the final restart with the gateway running.

#### Config-set calls (current + new)

After identifying the correct ordering, Stage 8 should execute these config-set calls:

```
config set gateway.mode local
config set gateway.bind {loopback|lan}
config set gateway.controlUi.allowedOrigins '["http://localhost:{gateway_port}"]' --strict-json
config set gateway.controlUi.dangerouslyDisableDeviceAuth true
```

All four use the same `compose_run` mechanism via the CLI service.

#### Fallback: direct JSON write

If the race/overwrite can't be resolved by reordering, write `openclaw.json` directly from Rust:

```rust
// Read existing config
let config_json_path = config.config_path().join("openclaw.json");
let mut doc: serde_json::Value = if config_json_path.exists() {
    serde_json::from_str(&std::fs::read_to_string(&config_json_path)?)?
} else {
    serde_json::json!({})
};

// Set gateway fields
doc["gateway"]["mode"] = serde_json::json!("local");
doc["gateway"]["bind"] = serde_json::json!(bind_mode);
doc["gateway"]["controlUi"]["allowedOrigins"] = serde_json::json!([format!("http://localhost:{}", config.gateway_port)]);
doc["gateway"]["controlUi"]["dangerouslyDisableDeviceAuth"] = serde_json::json!(true);

// Write back
std::fs::write(&config_json_path, serde_json::to_string_pretty(&doc)?)?;
```

This runs on the host filesystem (config dir is bind-mounted). Must ensure intermediate JSON objects exist (gateway, controlUi) before setting nested keys.

#### Error handling

- Replace `warn!` with real error propagation — if config-set fails after retries, fail the build
- Retry up to 3 times with 1-second delays
- Clear error message: "Failed to configure gateway settings"

## Testing Strategy

### Manual testing

1. Create a new instance with `gateway_bind=lan` → verify `openclaw.json` contains `"bind": "lan"`
2. Create a new instance with `gateway_bind=loopback` → verify `"bind": "loopback"`
3. Verify `controlUi.allowedOrigins` and `dangerouslyDisableDeviceAuth` are set in both cases
4. Open gateway from detail page → URL includes `?token=...` and gateway loads without auth errors
5. Instance card shows "Details" button → navigates to detail page (not browser)
6. Instance card does not show gateway URL

### Unit tests

- Update existing `getGatewayUrl` / `gateway_url` tests to expect `?token=` in output
- Add test for `getGatewayUrl` with token containing special characters (should be fine — tokens are hex strings)
