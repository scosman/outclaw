---
status: complete
---

# Phase 7: Instance Lifecycle + Status Polling

## Overview

Implemented real-time status updates via background polling and instance control (start/stop/restart) from the list view.

## What Was Implemented

### Backend (Rust)

1. **Status Poller** (`src-tauri/src/poller/mod.rs`)
   - Background tokio task that polls Docker status and container states
   - Configurable intervals: 5s (foreground) / 30s (background)
   - Emits `docker-status-changed` events on Docker state changes
   - Emits `instance-status-changed` events on container state changes
   - Thread-safe interval adjustment via `set_poller_interval` command

2. **Lifecycle Commands** (`src-tauri/src/commands/instances.rs`)
   - `start_instance` - Starts a stopped instance via Docker compose
   - `stop_instance` - Stops a running instance via Docker compose
   - `restart_instance` - Restarts an instance via Docker compose

3. **Poller Control** (`src-tauri/src/commands/docker.rs`)
   - `set_poller_interval` command - Adjusts polling frequency based on window focus

### Frontend (Svelte)

1. **Docker Store** (`src/lib/stores/docker.svelte.ts`)
   - Listens to `docker-status-changed` events
   - Reactively updates `isRunning` state

2. **Instances Store** (`src/lib/stores/instances.svelte.ts`)
   - Listens to `instance-status-changed` events
   - Listens to `docker-status-changed` events to update all instances when Docker stops
   - Sets all instances to `docker-not-running` state when Docker unavailable

3. **Instance Card** (`src/lib/components/InstanceCard.svelte`)
   - Action buttons wired based on state:
     - Running: "Open" (opens browser) + "Stop"
     - Stopped: "Start"
     - Error: "Details" (navigates to detail) + "Restart"
     - Docker not running: Disabled "Docker Not Running" button
   - Effective state calculation considering Docker availability
   - Loading states during actions

4. **Focus/Blur Handling** (`src/routes/+layout.svelte`)
   - Listens to `tauri://focus` and `tauri://blur` window events
   - Calls `set_poller_interval` to adjust polling frequency
   - Foreground (5s) when focused, background (30s) when blurred

## Manual Testing Instructions

### Prerequisites
- Docker Desktop must be running
- At least one instance created (via wizard)
- The app should be launched with `npm run tauri dev`

### Test Cases

1. **Status Polling**
   - Launch the app with a running instance
   - Verify instance card shows green "Running" status
   - Stop the container externally via Docker Desktop
   - Verify card updates to "Stopped" within 5 seconds

2. **Docker Status Changes**
   - Stop Docker Desktop
   - Verify all instance cards show "Docker Not Running" within seconds
   - Start Docker Desktop
   - Verify cards recover to their actual states

3. **Start/Stop Actions**
   - Click "Stop" on a running instance
   - Verify status changes to "Stopping" then "Stopped"
   - Click "Start" on a stopped instance
   - Verify status changes to "Starting" then "Running"

4. **Restart Action**
   - For an error state instance, click "Restart"
   - Verify restart attempt is made

5. **Focus/Blur Polling**
   - Open the app, note it polls every 5 seconds
   - Switch to another window (app loses focus)
   - Polling should slow to 30 seconds
   - Switch back to app
   - Polling should resume at 5 seconds

## Files Changed

- `src-tauri/src/poller/mod.rs` - Status poller implementation
- `src-tauri/src/commands/docker.rs` - set_poller_interval command
- `src-tauri/src/commands/instances.rs` - start/stop/restart commands
- `src-tauri/src/lib.rs` - Poller initialization and command registration
- `src-tauri/tauri.conf.json` - Permission for set_poller_interval
- `src/lib/stores/docker.svelte.ts` - Event handling
- `src/lib/stores/instances.svelte.ts` - Event handling and Docker state
- `src/lib/components/InstanceCard.svelte` - Action buttons
- `src/routes/+layout.svelte` - Focus/blur handling
