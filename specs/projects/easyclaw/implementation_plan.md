---
status: complete
---

# Implementation Plan: EasyClaw

10 phases. Each produces a reviewable, manually testable increment. Dependency order: each phase builds on the previous.

## Phases

- [ ] Phase 1: Project scaffolding, theming, and CI
- [ ] Phase 2: Rust backend core (data model, instance manager, Docker CLI, GitHub client)
- [ ] Phase 3: Docker detection + instance list UI
- [ ] Phase 4: Setup wizard — install type + configuration form
- [ ] Phase 5: Docker build pipeline
- [ ] Phase 6: Wizard completion + post-setup flow
- [ ] Phase 7: Instance lifecycle + status polling
- [ ] Phase 8: Instance detail screen
- [ ] Phase 9: Instance management (edit settings, rebuild, delete)
- [ ] Phase 10: Polish, settings, and release pipeline

---

### Phase 1: Project Scaffolding, Theming, and CI

Set up the project from scratch with all tooling configured.

**Delivers:**
- Tauri v2 project initialized with SvelteKit frontend
- `@sveltejs/adapter-static` configured, SSR disabled
- shadcn-svelte installed with dark theme as default
- Tailwind CSS configured
- JetBrains Mono font installed and set as global monospace font
- ESLint + Prettier + svelte-check configured
- Cargo fmt + Cargo clippy configured
- GitHub Actions CI workflow (lint, format, type-check, tests for both frontend and Rust)
- Basic app window: header bar with "EasyClaw" text, placeholder content area
- Window: 900x640 default, 720x500 minimum

**Manual test:** `npm run tauri dev` — app launches, dark themed window appears with monospace font and header bar.

---

### Phase 2: Rust Backend Core

All Rust backend modules, data models, and Tauri command stubs. No frontend integration yet — this phase is backend-only.

**Delivers:**
- Data models: `InstanceConfig`, `InstanceSettings`, `InstanceStatus`, `DockerStatus`, `Release`, `AppState`, `EasyClawError`
- Instance manager: CRUD operations, directory structure creation/deletion
- Name generator: curated adjective + animal word lists, collision handling
- Port allocator: auto-increment from 18789/18790, OS-level port check, validation
- Docker CLI wrapper: `check_available`, `compose_up`, `compose_stop`, `compose_down`, `compose_run`, `build` (with streaming), `list_containers`, `inspect_container`, `remove_image`
- GitHub releases client: fetch releases from API, cache to disk with TTL
- Dockerfile generator: fetch from GitHub raw content by tag
- Compose generator: produce docker-compose.yml via serde_yaml
- Env generator: produce .env file
- All Tauri commands registered (implemented where possible, stubbed where they need frontend integration)
- Error types with thiserror
- Logging with tracing
- Unit tests: names, ports, compose generation, env generation, model serialization

**Manual test:** `cargo test` — all unit tests pass. `cargo clippy` clean.

---

### Phase 3: Docker Detection + Instance List UI

First frontend screens connected to the backend. The app becomes interactive.

**Delivers:**
- Frontend TypeScript types matching Rust models (`src/lib/types/instance.ts`)
- Docker status store: calls `check_docker` on load, subscribes to `docker-status-changed` events
- `DockerStatusPill.svelte` component in header bar (green/red/yellow dot + text)
- Docker overlay: covers content area when Docker is unavailable, with install instructions or "start Docker" message, auto-advances when detected
- Instance store: calls `list_instances` on load, subscribes to `instance-status-changed` events
- Instance list view (`/` route): renders instance cards or empty state
- `InstanceCard.svelte`: name, status dot, version, status text (action buttons are placeholder/disabled — wired in Phase 7)
- `StatusDot.svelte`: colored dot per status
- Empty state: ASCII art logo placeholder + "Create Your First Instance" button (navigates to `/wizard` but wizard is not built yet — just navigates)
- SvelteKit routing configured for all routes (placeholder content for unbuilt routes)

**Manual test:** App launches. Docker status pill shows correct state. Stopping Docker Desktop shows the overlay. Empty state displays. (No instances to show yet — that comes with the wizard.)

---

### Phase 4: Setup Wizard — Install Type + Configuration Form

The wizard UI up through the configuration step. Does not build yet.

**Delivers:**
- Wizard route (`/wizard`) with wizard store managing step progression
- Wizard layout: custom header (back, title, step counter), footer (cancel, next), full-screen (hides main header)
- Step 1: Docker check — auto-skips if Docker running, otherwise shows overlay with polling
- Step 2: Install type selection — two cards (Standard / Custom), Standard pre-selected, Next advances
- Step 3: Custom configuration form:
  - Instance name (text input, pre-filled with generated name via `generate_instance_name` command)
  - OpenClaw version dropdown (fetched via `get_releases` command, cached)
  - Gateway port + Bridge port (side by side, validated)
  - Network access (radio: Local only / LAN)
  - Timezone (searchable dropdown, pre-filled via `get_system_timezone` command)
  - Install browser toggle
  - Advanced options collapsible section (apt packages, extensions, home volume, extra mounts, insecure WS)
  - Inline validation with error messages
- `ConfigForm.svelte` as shared component (accepts mode prop for create vs. edit)
- `create_instance` command wired: creates instance on disk when user clicks Next from config form (or from install type with Standard selected)
- Standard Install: calls create_instance with all defaults, skips to build step

**Manual test:** Open wizard, step through Docker check → Install Type → Custom Config. Version dropdown loads. Form validates. Back/Cancel work. Selecting Standard and clicking Next creates an instance (visible in `~/.easyclaw/instances/`). Wizard advances to build step (which shows placeholder).

---

### Phase 5: Docker Build Pipeline

The core build system. After this phase, you can create a fully running OpenClaw instance.

**Delivers:**
- `build_instance` Tauri command: full 9-stage pipeline
  - Stage 1: Fetch Dockerfile from GitHub
  - Stage 2: Generate compose + env files
  - Stage 3: Docker build (stream output via events)
  - Stage 4: Create/verify directories
  - Stage 5: Start container
  - Stage 6: Run onboarding
  - Stage 7: Fix permissions
  - Stage 8: Configure gateway
  - Stage 9: Restart gateway
- `build-progress` events streamed to frontend
- `BuildProgress.svelte` component: stage checklist (✓/◌/○/✗) + scrolling log output panel
- Wizard Step 4 (build screen): progress display, cancel button with confirmation, error state with Retry and Back to Settings
- Build cancellation: `cancel_build` command, kills running Docker process

**Manual test:** Run full wizard with Standard Install. Build progresses through all stages. Docker image builds. Container starts. Gateway is accessible at `http://localhost:18789`. If Docker is slow or fails, error is displayed with retry option.

---

### Phase 6: Wizard Completion + Post-Setup Flow

Remaining wizard steps. After this, the full setup wizard works end-to-end.

**Delivers:**
- `CodeBlock.svelte`: styled `<pre>` with copy-to-clipboard button
- `CopyButton.svelte`: clipboard icon button, copies text, brief "Copied!" feedback
- Wizard Step 5: Provider setup — CLI instructions with formatted command, Copy button, Skip/Done buttons
- Wizard Step 6: Channel setup — Telegram and WhatsApp sections, each with instructions and copyable command
- Wizard Step 7: Completion screen — ASCII art, instance summary card (name, version, status, gateway URL), "Open Gateway" and "Go to Dashboard" buttons
- `open_in_browser` command: opens URL in default browser
- On "Go to Dashboard": navigate to `/`, instance list now shows the new instance

**Manual test:** Complete full wizard end-to-end. Provider and channel steps show correct `docker compose` commands with the right paths. Copy buttons work. Completion screen shows correct info. "Open Gateway" opens browser to the gateway URL. "Go to Dashboard" shows instance list with the new instance card.

---

### Phase 7: Instance Lifecycle + Status Polling

Real-time status updates and instance control from the list view.

**Delivers:**
- Status poller: background tokio task, polls Docker every 5s (foreground) / 30s (background)
- `docker-status-changed` events emitted on Docker state changes
- `instance-status-changed` events emitted on container state changes
- Frontend stores reactively updated from events
- `start_instance`, `stop_instance`, `restart_instance` Tauri commands
- Instance card action buttons wired:
  - Running: "Open" (opens browser) + "Stop"
  - Stopped: "Start"
  - Error: "Details" (navigates to detail, placeholder) + "Restart"
- Docker status changes reflected globally (all instance cards show "Docker not running" when Docker stops)
- Poller interval adjusts on window focus/blur

**Manual test:** Start app with a running instance — card shows green "Running." Stop Docker Desktop — all cards switch to "Docker not running" within seconds. Start Docker — cards recover. Click "Stop" on a running instance — status changes to "Stopped." Click "Start" — comes back to "Running."

---

### Phase 8: Instance Detail Screen

Full detail view for a single instance.

**Delivers:**
- Instance detail route (`/instances/[id]`)
- "← Instances" back navigation
- Instance header: name (large, clickable for inline edit), status badge, version
- `rename_instance` command wired to inline edit
- Action buttons: Open Gateway, Start/Stop (contextual), Restart
- Details section: key-value layout with `SectionHeader.svelte`
  - Gateway URL (copyable)
  - Gateway token (masked, reveal toggle, copyable)
  - Bridge port, Network access, Timezone
  - Config path, Workspace path (copyable)
  - Container ID (copyable)
- Actions section: Edit Settings, Rebuild, Provider Setup, Channel Setup (Edit Settings and Rebuild are placeholder buttons — wired in Phase 9; Provider/Channel reuse wizard CLI instruction screens)
- Danger zone: Delete Instance button (placeholder — wired in Phase 9)
- Instance card click in list now navigates to detail

**Manual test:** Click an instance in the list → detail screen loads. All info fields display correctly. Copy buttons work. Token is masked, reveal toggle works. Rename works (inline edit, saves on enter). Start/Stop/Restart work. Back button returns to list. Provider/Channel setup shows CLI instructions.

---

### Phase 9: Instance Management (Edit, Rebuild, Delete)

Full instance lifecycle management from the detail screen.

**Delivers:**
- Edit settings route (`/instances/[id]/edit`): `ConfigForm.svelte` in edit mode, pre-filled with current instance settings
- `update_instance` command: updates instance.json, regenerates Docker files
- Save from edit triggers rebuild confirmation → build screen → return to instance detail
- Rebuild button: confirmation dialog → `build_instance` → build screen → return to detail
- Delete button: confirmation dialog with instance name and warning text
- `delete_instance` command: stops container, removes image, deletes directories
- After delete: navigate to instance list
- `SectionHeader.svelte` component
- `AlertDialog` (shadcn) for all confirmations

**Manual test:** Edit settings → change a port → save → rebuild runs → instance restarts with new port. Rebuild without changes → confirmation → build runs. Delete instance → confirmation → instance disappears from list, container removed, files cleaned up.

---

### Phase 10: Polish, Settings, and Release Pipeline

Final polish pass, app-level settings, and release infrastructure.

**Delivers:**
- Settings modal (gear icon in header): data directory path, app version, GitHub repo link
- `app-state.json` persistence: save/restore window position and size
- ASCII art logo finalized: compact version for header, medium for build/complete screens, large for empty state
- Loading states: skeleton/spinner for instance list initial load, version dropdown loading
- Smooth transitions between views (if appropriate — keep minimal)
- Focus-dependent polling wired to Tauri window events
- GitHub Actions release workflow: triggered on release creation, builds for macOS (Intel + ARM), Windows, Linux using `tauri-apps/tauri-action`
- Update `AGENTS.md` with actual working check/lint/test commands
- README with project description, development setup, and build instructions

**Manual test:** Full end-to-end flow: launch app → create instance (Standard + Custom) → view in list → open detail → edit settings → rebuild → stop/start → delete. Window position remembered across restarts. Settings modal shows correct info. Release build produces artifacts.
