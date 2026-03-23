---
status: complete
---

# Phase 5: Docker Build Pipeline

## Overview

Implemented the core build system with 9-stage pipeline, cancellation support, and real-time progress UI.

## What Was Implemented

### Backend (Rust)

1. **Build Pipeline** (`src-tauri/src/commands/instances.rs`)
   - 9-stage build process with progress events
   - Stage 1: Fetch Dockerfile from GitHub (with fallback)
   - Stage 2: Generate docker-compose.yml and .env files
   - Stage 3: Build Docker image with streaming output
   - Stage 4: Verify/create directories
   - Stage 5: Start container via compose_up
   - Stage 6: Run onboarding command
   - Stage 7: Fix file permissions (as root)
   - Stage 8: Configure gateway mode and bind
   - Stage 9: Restart gateway to apply changes
   - Cancellation checks between stages

2. **Build Cancellation** (`src-tauri/src/commands/instances.rs`)
   - `BuildTracker` struct for managing active builds
   - `cancel_build` Tauri command
   - Cancellation token checked between stages

3. **Docker CLI Enhancement** (`src-tauri/src/docker/cli.rs`)
   - `compose_run_with_entrypoint` method for permission fix command
   - Entry point override support

4. **Fallback Dockerfile** (`src-tauri/src/docker/fallback_dockerfile.txt`)
   - Minimal Dockerfile for error/offline scenarios

### Frontend (Svelte)

1. **BuildProgress Component** (`src/lib/components/BuildProgress.svelte`)
   - Stage checklist with status icons (✓/◌/○/✗)
   - Scrolling log output panel
   - Cancel button with confirmation
   - Error state with Retry and Back to Settings buttons
   - Success state indicator
   - Listens to `build-progress` events from backend

2. **Wizard Integration** (`src/routes/wizard/+page.svelte`)
   - Integrated BuildProgress component
   - Tracks build completion/error state
   - Footer shows appropriate buttons based on build state

## Manual Testing Instructions

### Prerequisites
- Docker Desktop must be running
- The app should be launched with `npm run tauri dev`

### Test Cases

1. **Standard Install Flow**
   - Launch the app
   - Click "Setup OpenClaw" button
   - Select "Standard Install"
   - Click Next
   - Verify the build progress screen shows:
     - Stage checklist with progressing states
     - Log output scrolling
     - Cancel button available
   - Wait for completion
   - Verify "Go to Dashboard" button appears
   - Click it and verify the instance appears in the list

2. **Custom Install Flow**
   - Start new instance creation
   - Select "Custom Install"
   - Configure options (change ports, timezone, etc.)
   - Click Next
   - Verify build progresses
   - Verify instance is created with custom settings

3. **Build Cancellation**
   - Start a build
   - Click "Cancel Build" button during build
   - Verify cancellation is requested
   - Note: Cancellation happens between stages, not mid-stage

4. **Error Handling**
   - To test error handling, you could:
     - Stop Docker Desktop during build
     - Or modify the Dockerfile to have an error
   - Verify error state shows with:
     - Error message displayed
     - "Retry Build" button
     - "Back to Settings" button

5. **Retry Flow**
   - After an error, click "Retry Build"
   - Verify build restarts from beginning

6. **Back to Settings**
   - After an error, click "Back to Settings"
   - Verify you return to the configuration step
   - Verify settings are preserved

## Known Limitations

1. The OpenClaw Dockerfile fetch from GitHub will fail in dev until there's a real OpenClaw repo - fallback Dockerfile is used
2. Onboarding, permission fix, and gateway config commands may fail if the image doesn't have those CLI commands - errors are logged but don't fail the build

## Files Changed

- `src-tauri/src/commands/instances.rs` - Build pipeline and cancellation
- `src-tauri/src/docker/cli.rs` - compose_run_with_entrypoint method
- `src-tauri/src/docker/mod.rs` - Export dockerfile functions
- `src-tauri/src/lib.rs` - Register cancel_build command
- `src-tauri/src/docker/fallback_dockerfile.txt` - New file
- `src/lib/components/BuildProgress.svelte` - New file
- `src/routes/wizard/+page.svelte` - Integrated BuildProgress
- `src/lib/components/EmptyState.svelte` - Minor text change
