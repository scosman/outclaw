# Phase 2: Backend Gateway Config Fix (Stage 8 Race/Overwrite + controlUi)

## Overview

Fix the Stage 8 config-set race condition by replacing CLI-based config commands with direct `openclaw.json` filesystem writes. This bypasses the CLI entirely, eliminating the race where the gateway startup in Stage 9 overwrites CLI-set values. Also adds `controlUi` configuration and proper error handling with retry logic.

## Steps

### 1. Add `apply_gateway_config` helper in `src-tauri/src/commands/instances.rs`

New function that directly writes gateway settings to `openclaw.json` on the host filesystem:

- Takes config dir path, bind mode string, and gateway port
- Reads existing `openclaw.json` (or creates empty JSON object)
- Ensures intermediate objects (`gateway`, `controlUi`) exist
- Sets all four config values:
  - `gateway.mode` → `"local"`
  - `gateway.bind` → `bind_mode`
  - `gateway.controlUi.allowedOrigins` → `["http://localhost:{port}"]`
  - `gateway.controlUi.dangerouslyDisableDeviceAuth` → `true`
- Writes back with `serde_json::to_string_pretty`
- Returns `Result<(), String>`

### 2. Add `write_gateway_config_with_retry` wrapper

- Calls `apply_gateway_config` up to 3 times with 1-second delays
- Returns the last error on exhaustion
- Treats failures as real errors (not warnings)

### 3. Modify Stage 8 in `build_instance`

- Remove the two `compose_run` CLI-based config-set calls (`config set gateway.mode`, `config set gateway.bind`)
- Keep the progress emission ("Configuring gateway settings...")
- Progress now indicates preparation only (actual write happens in Stage 9)

### 4. Modify Stage 9 in `build_instance`

- After `compose_stop` and sleep, call `write_gateway_config_with_retry`
- If it fails, emit error and return early (fatal)
- Then proceed with `compose_up`
- This timing ensures no running gateway can overwrite our changes

### 5. Add tests for `apply_gateway_config`

In `instances.rs`, add a `#[cfg(test)]` module with:

- Test: writes config to fresh directory (no existing `openclaw.json`)
- Test: preserves existing fields in `openclaw.json` while setting gateway fields
- Test: overwrites existing gateway fields with new values
- Test: creates intermediate objects (`gateway`, `controlUi`) when missing

## Test Plan

- `./checks.sh` passes (lint, format, typecheck, clippy, cargo test)
- Unit tests verify JSON structure and field preservation
- Manual: create instance with `gateway_bind=lan` → verify `openclaw.json` has `"bind": "lan"`
- Manual: verify `controlUi.allowedOrigins` and `dangerouslyDisableDeviceAuth` are set

## Completion Criteria

- Stage 8 no longer uses CLI `config set` commands
- `openclaw.json` is written directly from Rust after gateway stop, before restart
- All four gateway config values are set correctly
- Failures propagate as build errors (not swallowed as warnings)
- Retry logic (3 attempts, 1s delay) wraps the config write
- All checks pass
