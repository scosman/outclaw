# Phase 1: Frontend Fixes (InstanceCard Cleanup + Token in URL)

## Overview

Simplify the InstanceCard component by removing the gateway URL display and converting the "Open" button to a "Details" navigation button. Add auth token to gateway URLs so all consumers (detail page, copy button, browser open) include it automatically.

## Steps

### 1. Update `getGatewayUrl` in `src/lib/types/instance.ts`

- Change return value from `http://localhost:${config.gateway_port}` to `http://localhost:${config.gateway_port}?token=${config.gateway_token}`
- All consumers (detail page Open Gateway button, URL display, CopyButton) pick this up automatically

### 2. Update `gateway_url` in `src-tauri/src/instance/models.rs`

- Change `format!("http://localhost:{}", self.gateway_port)` to `format!("http://localhost:{}?token={}", self.gateway_port, self.gateway_token)`
- Used in build-complete log message

### 3. Update `InstanceCard.svelte` in `src/lib/components/InstanceCard.svelte`

- Remove `getGatewayUrl` from imports
- Remove `handleOpen` function
- Remove the gateway URL display block (the `{#if isRunning}` section with the link)
- Replace the "Open" button in the running state with a "Details" button that navigates to `/instances/{id}` (using existing `handleDetails`)

### 4. Update Rust tests

- Add a test for `gateway_url()` verifying it includes the token

## Test Plan

- `./checks.sh` passes (lint, format, typecheck, clippy, cargo test)
- Manual: instance card shows "Details" button, not "Open" or gateway URL
- Manual: detail page gateway URL display includes `?token=...`
- Manual: "Open Gateway" button opens URL with token
- Manual: CopyButton copies URL with token

## Completion Criteria

- InstanceCard has no gateway URL display and no `handleOpen`/`open_in_browser`
- InstanceCard "Open" button replaced with "Details" navigating to `/instances/{id}`
- `getGatewayUrl()` appends `?token=` to URL
- `gateway_url()` in Rust appends `?token=` to URL
- All checks pass
