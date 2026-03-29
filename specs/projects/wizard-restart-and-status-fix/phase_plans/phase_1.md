# Phase 1: Backend shared helper + restart command, frontend store fix + restart flow

## Overview

Single phase delivering all changes needed for gateway restart on wizard completion and stale instance status fix. Changes are tightly coupled — the frontend depends on the backend command, and the store fix depends on the restart flow.

## Ordered Steps

### 1. Extract `wait_for_gateway_ready` helper in `src-tauri/src/commands/instances.rs`

- Add standalone async function before `connect_whatsapp`
- Params: `docker_cli: &DockerCli`, `container_name: &str`, `initial_delay_secs: u64`, `max_attempts: u32`
- Sleeps initial delay, then polls `docker exec echo ready` up to max_attempts times

### 2. Refactor `connect_whatsapp` in `src-tauri/src/commands/instances.rs`

- Replace inline poll loop (lines 1096–1124) with `wait_for_gateway_ready(&state.docker_cli, &container_name, 5, 30).await`
- Map error to `emit_progress` + return

### 3. Add `restart_gateway` Tauri command in `src-tauri/src/commands/instances.rs`

- Load instance config
- compose_stop (non-fatal, log warning)
- sleep 1s
- compose_up (fatal)
- wait_for_gateway_ready
- emit_instance_status

### 4. Register `restart_gateway` in `src-tauri/src/lib.rs`

- Add to invoke_handler generate_handler! array

### 5. Add instance to store after creation in `src/routes/wizard/+page.svelte`

- In `createAndBuild()`, after `wizardStore.createInstance()` succeeds, call `instancesStore.setInstance()` with config + building status

### 6. Replace channel→complete transition in `src/routes/wizard/+page.svelte`

- Add `isRestarting` and `restartError` state variables
- Set `isRestarting = true`, advance step, call `invoke('restart_gateway')`, refresh store
- Add `retryRestart()` function

### 7. Add "Finishing Setup" interstitial UI in `src/routes/wizard/+page.svelte`

- In complete step block: show spinner when `isRestarting`, error when `restartError`, else existing complete screen

### 8. Complete screen reads from store reactively in `src/routes/wizard/+page.svelte`

- Add `$derived` for `createdInstanceFromStore` from `instancesStore.getInstance()`
- Remove `createdInstance` local variable and `fetchCreatedInstance()` function

## Completion Criteria

- `wait_for_gateway_ready` extracted and used by both `connect_whatsapp` and `restart_gateway`
- `restart_gateway` registered and callable from frontend
- Instance added to instancesStore immediately after creation
- Channel→complete transition triggers gateway restart with interstitial UI
- Complete screen derives status reactively from store
- All checks pass: `./checks.sh`
