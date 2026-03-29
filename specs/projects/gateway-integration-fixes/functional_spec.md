---
status: complete
---

# Functional Spec: Gateway Integration Fixes

## 1. Instance Card Cleanup

### Remove gateway URL display

The `InstanceCard` component currently shows the gateway URL (e.g., `http://localhost:18789`) when an instance is running. Remove this display entirely — the URL is available on the detail page.

### "Open" → "Details" button

The "Open" button on running instance cards currently opens the gateway URL in an external browser. Change it to:

- Rename the button label from "Open" to "Details"
- Navigate to the instance detail page (`/instances/{id}`) instead of opening the browser
- Remove the `handleOpen` function and the `open_in_browser` invocation from `InstanceCard`
- Remove the `getGatewayUrl` import (no longer needed in this component)

## 2. Gateway URL Auth Token

`getGatewayUrl` currently returns `http://localhost:{port}`. Append the auth token:

```
http://localhost:{port}?token={gateway_token}
```

The token comes from `InstanceConfig.gateway_token`, already available on every config object. This is a single change in `getGatewayUrl()` in `src/lib/types/instance.ts`. All consumers (detail page "Open Gateway" button, URL display, copy button) pick it up automatically.

Also update `gateway_url()` in `src-tauri/src/instance/models.rs` to include the token, so the Rust-side URL matches (used in the build-complete log message).

Token is always present (generated at instance creation) — no empty-token guard needed.

## 3. Fix gateway.bind Config Not Persisting

### Problem

During `build_instance` Stage 8, the `config set gateway.bind` command IS called with the correct value (confirmed via logging — `bind_mode` is `"lan"` when expected). The same CLI command (`openclaw config set gateway.bind lan`) also works when run manually after instance creation. However, the resulting `openclaw.json` still contains `"bind": "loopback"` after the build completes.

This points to a **race condition or overwrite** — something else writes to `openclaw.json` after Stage 8 sets the value, or the gateway process reinitializes its config on startup in Stage 9 and overwrites the changes.

### Likely causes to investigate

1. **Gateway startup overwrite** — Stage 9 restarts the gateway (`compose_stop` then `compose_up`). The gateway process may regenerate default config values on startup, overwriting what Stage 8 set.
2. **Onboarding race** — Stage 6 (`onboard`) may trigger async config initialization that completes after Stage 8's config-set.
3. **File write race** — The config-set CLI container and the running gateway container both have the config directory mounted. Concurrent writes could cause one to overwrite the other.

### Fix approach

1. **Identify the overwrite source** — Compare `openclaw.json` contents after Stage 8 vs. after Stage 9 restart to pinpoint when the value reverts.
2. **Fix the ordering/race** — Possible fixes depending on root cause:
   - Move config-set calls to after the final gateway restart
   - Stop the gateway before config-set, then start it after
   - Add a delay or readiness check between config-set and restart
3. **Fallback** — If the race can't be reliably resolved, write `openclaw.json` directly from Rust on the host filesystem (config dir is bind-mounted) after the gateway is stopped but before final start. This bypasses the CLI and any in-process overwrites.

### Expected config values

- `gateway.mode` → always `"local"`
- `gateway.bind` → `"loopback"` or `"lan"` (matching `config.gateway_bind`)

### Error handling

Config-set failures should be treated as real errors, not swallowed. Use retry (2-3 attempts with short delay) then fail the build with a clear error message.

## 4. Set controlUi Configuration

### New config-set calls

After setting `gateway.mode` and `gateway.bind`, also configure:

- `gateway.controlUi.allowedOrigins` → `["http://localhost:{gateway_port}"]`
- `gateway.controlUi.dangerouslyDisableDeviceAuth` → `true`

Set for **all** instances (both loopback and LAN). Without these, the gateway control UI rejects connections from the host browser.

### Implementation

Use the same mechanism as the mode/bind config-set (whatever approach fixes issue #3). For `allowedOrigins`, pass as JSON with `--strict-json` flag if using the CLI approach.

### Error handling

Same as #3 — retry then fail on error.

## Out of Scope

- Changing the gateway URL for LAN mode (stays `http://localhost:{port}`)
- `update_instance` re-applying gateway config on setting changes
- Changes to the wizard flow beyond what's listed here
