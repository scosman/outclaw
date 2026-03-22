---
status: complete
---

# Architecture: EasyClaw

Tauri desktop application. Rust backend manages Docker and the filesystem; SvelteKit frontend renders the UI. Communication via Tauri's IPC (commands + events).

## 1. Project Structure

```
easyclaw/
  src-tauri/                       ← Rust backend (Tauri)
    src/
      main.rs                      ← entry point, plugin registration
      lib.rs                       ← command registration
      commands/                    ← Tauri command handlers (IPC boundary)
        mod.rs
        docker.rs                  ← check_docker
        instances.rs               ← CRUD, start/stop/restart/rebuild/delete
        releases.rs                ← get_releases
        system.rs                  ← open_browser, get_timezone, generate_name
      docker/                      ← Docker CLI wrapper
        mod.rs
        cli.rs                     ← execute docker/compose commands, parse output
        compose_gen.rs             ← generate docker-compose.yml
        dockerfile_gen.rs          ← fetch + prepare Dockerfile
        env_gen.rs                 ← generate .env file
      instance/                    ← Instance data management
        mod.rs
        manager.rs                 ← CRUD operations, directory management
        models.rs                  ← Instance, InstanceSettings, InstanceStatus
        names.rs                   ← adjective+animal name generator
        ports.rs                   ← port allocation and validation
      container/                   ← Docker container management
        mod.rs
        manager.rs                 ← container lifecycle, directory management
      github/                      ← GitHub API client
        mod.rs
        releases.rs                ← fetch + cache releases
      poller/                      ← Background status polling
        mod.rs
    Cargo.toml
    tauri.conf.json
  src/                             ← SvelteKit frontend
    lib/
      stores/
        instances.ts               ← instance list + status, reactive store
        docker.ts                  ← Docker availability status
        wizard.ts                  ← wizard step state + form data
      components/
        ui/                        ← shadcn-svelte components (auto-generated)
        InstanceCard.svelte
        StatusDot.svelte
        CodeBlock.svelte           ← <pre> with copy button
        CopyButton.svelte
        SectionHeader.svelte
        DockerStatusPill.svelte
        BuildProgress.svelte       ← stage checklist + log output
        ConfigForm.svelte          ← shared between wizard and edit settings
      types/
        instance.ts                ← TypeScript types matching Rust models
    routes/
      +layout.svelte               ← header bar, Docker overlay
      +page.svelte                 ← instance list (main screen)
      instances/[id]/
        +page.svelte               ← instance detail
        edit/+page.svelte          ← edit settings form
      wizard/
        +page.svelte               ← setup wizard (all steps in one route)
  static/
    fonts/                         ← JetBrains Mono font files
  package.json
  svelte.config.js
  vite.config.ts
  tailwind.config.ts
  tsconfig.json
```

### SvelteKit Configuration
- Adapter: `@sveltejs/adapter-static` — builds to static SPA for Tauri
- SSR disabled (`ssr: false` in all layouts)
- Client-side routing only, all navigation happens in the Tauri webview

## 2. Data Model

### Persisted: `instance.json`

```typescript
interface InstanceConfig {
  id: string;                    // "ec_a1b2c3" — generated, filesystem-safe
  name: string;                  // "Cosmic Otter" — user-facing
  openclawVersion: string;       // GitHub release tag, e.g., "v0.42.1"
  containerId: string;           // references a container in ~/.easyclaw/docker-containers/<containerId>/
  gatewayPort: number;
  bridgePort: number;
  gatewayBind: "loopback" | "lan";
  gatewayToken: string;          // 32-byte hex, auto-generated
  timezone: string;              // IANA, e.g., "America/Toronto"
  installBrowser: boolean;
  aptPackages: string;           // space-separated
  extensions: string;            // space-separated
  homeVolume: string;            // empty = default
  extraMounts: string;           // comma-separated source:target[:opts]
  allowInsecureWs: boolean;
  createdAt: string;             // ISO 8601
  updatedAt: string;             // ISO 8601
}
```

Serialized as JSON to `~/.easyclaw/instances/<id>/instance.json`. The Rust backend owns all reads/writes; the frontend never touches the filesystem directly.

### Runtime-Only (Not Persisted)

```typescript
interface InstanceStatus {
  state: "building" | "running" | "stopped" | "error" | "docker-not-running";
  containerId?: string;
  errorMessage?: string;
}

interface DockerStatus {
  state: "running" | "not-running" | "not-installed";
  composeAvailable: boolean;
}

interface Release {
  tag: string;              // "v0.42.1"
  name: string;             // release title
  publishedAt: string;      // ISO 8601
  prerelease: boolean;
  commitSha: string;        // for fetching Dockerfile at this version
}
```

### App State: `app-state.json`

```typescript
interface AppState {
  windowPosition?: { x: number; y: number };
  windowSize?: { width: number; height: number };
  lastActiveInstance?: string;  // instance ID
}
```

Stored at `~/.easyclaw/app-state.json`. Read on launch, written on window close/move.

## 3. IPC Design (Tauri Commands + Events)

### Commands (Frontend → Backend)

All commands are `async` and return `Result<T, String>` on the Rust side. The frontend calls them via `@tauri-apps/api/core`.

**Docker:**
```rust
#[tauri::command]
async fn check_docker() -> Result<DockerStatus, String>
```

**Instances:**
```rust
#[tauri::command]
async fn list_instances() -> Result<Vec<InstanceWithStatus>, String>

#[tauri::command]
async fn get_instance(id: String) -> Result<InstanceWithStatus, String>

#[tauri::command]
async fn create_instance(settings: InstanceSettings) -> Result<InstanceConfig, String>
// Creates ID, dirs, generates Dockerfile/compose/env. Does NOT build.

#[tauri::command]
async fn update_instance(id: String, settings: InstanceSettings) -> Result<InstanceConfig, String>
// Updates config, regenerates Docker files. Does NOT rebuild.

#[tauri::command]
async fn delete_instance(id: String) -> Result<(), String>
// Stops container, removes image, deletes all instance dirs.

#[tauri::command]
async fn rename_instance(id: String, name: String) -> Result<(), String>
```

**Lifecycle:**
```rust
#[tauri::command]
async fn build_instance(id: String, app_handle: tauri::AppHandle) -> Result<(), String>
// Runs docker build + setup steps. Streams progress via events.

#[tauri::command]
async fn start_instance(id: String) -> Result<(), String>
// docker compose up -d

#[tauri::command]
async fn stop_instance(id: String) -> Result<(), String>
// docker compose stop

#[tauri::command]
async fn restart_instance(id: String) -> Result<(), String>
// docker compose restart
```

**Releases:**
```rust
#[tauri::command]
async fn get_releases() -> Result<Vec<Release>, String>
// Returns cached if fresh, fetches if stale/missing.
```

**System:**
```rust
#[tauri::command]
async fn get_system_timezone() -> Result<String, String>

#[tauri::command]
async fn generate_instance_name() -> Result<String, String>

#[tauri::command]
async fn open_in_browser(url: String) -> Result<(), String>
```

### Events (Backend → Frontend)

Emitted via `app_handle.emit(event, payload)`. Frontend listens with `listen()`.

```
"docker-status-changed"    → DockerStatus
"instance-status-changed"  → { id: string, status: InstanceStatus }
"build-progress"           → { id: string, stage: string, log: string, done: boolean, error?: string }
```

The poller emits `docker-status-changed` and `instance-status-changed`. The `build_instance` command emits `build-progress` as each stage completes and as Docker build output streams.

## 4. Backend Modules

### 4.1 Docker CLI Wrapper (`docker/cli.rs`)

All Docker interaction goes through CLI commands — no Docker API libraries. Rationale:
- `docker compose` has complex behavior that's hard to replicate via API
- CLI output matches what users would see in a terminal (helpful for debugging)
- Fewer Rust dependencies, simpler cross-platform (Docker CLI handles platform differences)

```rust
pub struct DockerCli {
    docker_bin: String,  // "docker", resolved on init
}

impl DockerCli {
    pub async fn check_available(&self) -> Result<DockerStatus>;
    pub async fn compose_up(&self, compose_path: &Path, project_name: &str) -> Result<()>;
    pub async fn compose_stop(&self, compose_path: &Path, project_name: &str) -> Result<()>;
    pub async fn compose_down(&self, compose_path: &Path, project_name: &str) -> Result<()>;
    pub async fn compose_run(&self, compose_path: &Path, project_name: &str, service: &str, args: &[&str]) -> Result<String>;
    pub async fn build(&self, context_path: &Path, tag: &str, build_args: &HashMap<String, String>, progress_tx: Sender<String>) -> Result<()>;
    pub async fn inspect_container(&self, container_name: &str) -> Result<ContainerInfo>;
    pub async fn list_containers(&self, label_filter: &str) -> Result<Vec<ContainerInfo>>;
    pub async fn remove_image(&self, tag: &str) -> Result<()>;
}
```

**Key details:**
- Commands run via `tokio::process::Command` for async execution
- Build output streamed line-by-line via a channel, forwarded as Tauri events
- Container listing uses `docker ps --filter label=easyclaw.container --format json` — we label all containers with both container and instance IDs
- Errors: capture stderr, return as structured error messages
- Timeouts: 30s for quick commands (inspect, ps), no timeout for build (can take minutes)

### 4.2 Dockerfile Generator (`docker/dockerfile_gen.rs`)

```rust
pub async fn fetch_dockerfile(release: &Release) -> Result<String>
pub fn prepare_build_context(instance: &InstanceConfig, dockerfile_content: &str, docker_dir: &Path) -> Result<()>
```

**Fetch strategy:**
1. Download `Dockerfile` from GitHub raw content at the release tag: `https://raw.githubusercontent.com/openclaw/openclaw/{tag}/Dockerfile`
2. Cache downloaded Dockerfiles in `~/.easyclaw/docker-containers/<containerId>/`
3. If fetch fails, check cache — a cached Dockerfile from a previous build of the same version is reusable

**Build context:**
The fetched Dockerfile and any supporting files are written to `~/.easyclaw/docker-containers/<containerId>/`. This directory is the Docker build context. If the Dockerfile references local files (e.g., `COPY`), we may need to fetch those too — determine during implementation by inspecting the actual OpenClaw Dockerfile.

### 4.3 Compose Generator (`docker/compose_gen.rs`)

```rust
pub fn generate_compose(instance: &InstanceConfig) -> Result<String>
pub fn generate_extra_compose(instance: &InstanceConfig) -> Result<Option<String>>
```

Generates `docker-compose.yml` as a YAML string. Uses the `serde_yaml` crate (no hand-written YAML templates).

**Service naming:** `easyclaw-{containerId}-gateway`, `easyclaw-{containerId}-cli`. The compose project name is `easyclaw-{containerId}`.

**Labels:** Every container gets `easyclaw.container={containerId}` and `easyclaw.instance={instanceId}` labels for identification by the poller.

**Port mapping:**
```yaml
ports:
  - "127.0.0.1:{gatewayPort}:18789"   # loopback
  # or
  - "{gatewayPort}:18789"              # lan
```

Bind address depends on `gatewayBind` setting.

### 4.4 Env Generator (`docker/env_gen.rs`)

```rust
pub fn generate_env(instance: &InstanceConfig) -> String
```

Writes all instance settings as `KEY=VALUE` lines to `~/.easyclaw/docker/<id>/.env`. Format matches what OpenClaw's docker-compose expects.

### 4.5 Instance Manager (`instance/manager.rs`)

```rust
pub struct InstanceManager {
    base_dir: PathBuf,  // ~/.easyclaw
}

impl InstanceManager {
    pub fn list(&self) -> Result<Vec<InstanceConfig>>;
    pub fn get(&self, id: &str) -> Result<InstanceConfig>;
    pub fn create(&self, settings: InstanceSettings) -> Result<InstanceConfig>;
    pub fn update(&self, id: &str, settings: InstanceSettings) -> Result<InstanceConfig>;
    pub fn delete(&self, id: &str) -> Result<()>;
    pub fn rename(&self, id: &str, name: &str) -> Result<()>;
}
```

**create() flow:**
1. Generate instance ID (`ec_` + 6 random alphanumeric chars)
2. Generate container ID (`ct_` + 6 random alphanumeric chars)
3. Generate name if not provided (via `names::generate()`)
4. Allocate ports (via `ports::allocate()`)
5. Generate gateway token (`rand::thread_rng().gen::<[u8; 32]>` → hex)
6. Create directory structure:
   ```
   instances/<instanceId>/instance.json
   instances/<instanceId>/config/
   instances/<instanceId>/config/identity/
   instances/<instanceId>/config/agents/main/agent/
   instances/<instanceId>/config/agents/main/sessions/
   instances/<instanceId>/workspace/
   docker-containers/<containerId>/
   ```
7. Write `instance.json` (includes `containerId` reference)
8. Generate and write Dockerfile, docker-compose.yml, .env to `docker-containers/<containerId>/`

**delete() flow:**
1. Stop container if running (`docker compose stop`)
2. Remove containers (`docker compose down`)
3. Remove Docker image (`docker rmi easyclaw-<containerId>:latest`)
4. Delete `instances/<instanceId>/` directory tree
5. Delete `docker-containers/<containerId>/` directory tree (only if no other instance references it — future-proof for forking)

### 4.6 Name Generator (`instance/names.rs`)

```rust
pub fn generate(existing_names: &[String]) -> String
```

Two curated word lists embedded in the binary:
- ~100 adjectives: "cosmic", "swift", "midnight", "golden", "electric", "crimson", "phantom", "stellar", "neon", "arctic", ...
- ~100 animals: "otter", "falcon", "raven", "buffalo", "panther", "mantis", "condor", "viper", "lynx", "osprey", ...

Pick one from each at random, join with space. Check against `existing_names`; if collision, regenerate (max 10 attempts, then append a digit).

10,000 possible combinations — more than enough to avoid collisions for local instances.

### 4.7 Port Allocator (`instance/ports.rs`)

```rust
pub fn allocate(existing_instances: &[InstanceConfig]) -> Result<(u16, u16)>
pub fn validate_port(port: u16, instance_id: Option<&str>, existing: &[InstanceConfig]) -> Result<()>
```

**allocate():** Start from (18789, 18790). If taken by an existing instance, try (18791, 18792), etc. Also check OS-level port availability using `TcpListener::bind()` — catches non-EasyClaw conflicts.

**validate_port():** Check range (1024–65535), check against other instances (excluding self if editing), check OS bind. Returns specific error message for each failure case.

### 4.8 Status Poller (`poller/mod.rs`)

```rust
pub struct Poller { /* ... */ }

impl Poller {
    pub fn start(app_handle: AppHandle, instance_manager: Arc<InstanceManager>, docker_cli: Arc<DockerCli>);
    pub fn set_interval(&self, interval: Duration);
}
```

Runs as a `tokio::spawn` background task. On each tick:
1. `docker_cli.check_available()` → emit `docker-status-changed` if state changed
2. If Docker running: `docker_cli.list_containers("easyclaw.container")` → match containers to instances via `containerId` → emit `instance-status-changed` for any changes
3. Sleep for interval

**Intervals:**
- Default: 5 seconds
- When Tauri window loses focus: 30 seconds (listen to Tauri `window-focus-changed` event)
- During build: poller skips the building instance (build command handles its own status)

The poller keeps a `HashMap<String, InstanceStatus>` of last-known statuses to avoid emitting duplicate events.

### 4.9 GitHub Releases Client (`github/releases.rs`)

```rust
pub struct ReleasesClient {
    cache_path: PathBuf,  // ~/.easyclaw/releases-cache.json
    cache_ttl: Duration,  // 1 hour
}

impl ReleasesClient {
    pub async fn get_releases(&self) -> Result<Vec<Release>>;
}
```

**Fetch:** GET `https://api.github.com/repos/openclaw/openclaw/releases` with `Accept: application/vnd.github+json`. Parse JSON response. No auth token needed for public repos (but respect rate limits — 60 req/hr unauthenticated).

**Cache:** Write response to `~/.easyclaw/releases-cache.json` with a timestamp. On next call, if cache is < 1 hour old, return cached. If fetch fails but cache exists, return stale cache with a warning flag.

**HTTP client:** `reqwest` crate with Tauri's HTTP client plugin, or direct `reqwest` with a reasonable timeout (10s).

## 5. Frontend Architecture

### 5.1 Stores

**`instances.ts`** — Central instance store:
```typescript
interface InstanceStore {
  instances: Map<string, InstanceConfig & { status: InstanceStatus }>;
  loading: boolean;
}
```
- Initialized on app load via `list_instances()` command
- Updated reactively from `instance-status-changed` events
- Individual mutations (create, delete, rename) update the store optimistically then confirm via backend response

**`docker.ts`** — Docker status:
```typescript
interface DockerStore {
  status: DockerStatus;
}
```
- Initialized on app load via `check_docker()` command
- Updated from `docker-status-changed` events
- Used by layout to show/hide Docker overlay

**`wizard.ts`** — Wizard step state:
```typescript
interface WizardStore {
  currentStep: number;
  installType: "standard" | "custom";
  settings: Partial<InstanceSettings>;
  buildState: { stage: string; logs: string[]; done: boolean; error?: string } | null;
  createdInstanceId: string | null;
}
```
- Local to the wizard route, reset on wizard entry
- `buildState` updated from `build-progress` events

### 5.2 Routing

| Route | View | Notes |
|-------|------|-------|
| `/` | Instance list | Main screen, empty state if no instances |
| `/instances/[id]` | Instance detail | Back button returns to `/` |
| `/instances/[id]/edit` | Edit settings form | Reuses `ConfigForm.svelte` |
| `/wizard` | Setup wizard | All steps within a single route, managed by `wizard.ts` store |

The wizard is a single SvelteKit route that renders different step components based on the store's `currentStep`. This avoids URL-per-step complexity and makes the linear flow easy to manage.

### 5.3 Shared Components

**`ConfigForm.svelte`** — Used by both wizard Step 3 and the edit settings page. Accepts initial values as props, emits `on:save` with validated settings. Handles validation, advanced options toggle, version dropdown fetching.

**`BuildProgress.svelte`** — Used by wizard Step 4 and the rebuild flow. Listens to `build-progress` events for a given instance ID. Renders stage checklist and log output.

## 6. Build Process (Instance Setup)

The `build_instance` command orchestrates the full setup sequence. Each stage emits progress events.

```
Stage 1: "Fetching Dockerfile"
  → download Dockerfile from GitHub at release tag
  → write to docker-containers/<containerId>/Dockerfile

Stage 2: "Generating configuration"
  → generate docker-compose.yml → docker-containers/<containerId>/docker-compose.yml
  → generate .env → docker-containers/<containerId>/.env
  → generate extra compose if needed

Stage 3: "Building Docker image"
  → docker build -t easyclaw-<containerId>:latest --build-arg ... docker-containers/<containerId>/
  → stream output line by line

Stage 4: "Creating directories"
  → mkdir config subdirs, workspace
  → (already done by create_instance, but ensure they exist)

Stage 5: "Starting container"
  → docker compose -f ... -p easyclaw-<containerId> up -d

Stage 6: "Running initial setup"
  → docker compose run --rm easyclaw-<containerId>-cli onboard --mode local --no-install-daemon

Stage 7: "Fixing permissions"
  → docker compose run --rm --user root --entrypoint sh easyclaw-<containerId>-cli -c 'find /home/node/.openclaw -xdev -exec chown node:node {} +'

Stage 8: "Configuring gateway"
  → docker compose run --rm easyclaw-<containerId>-cli config set gateway.mode local
  → docker compose run --rm easyclaw-<containerId>-cli config set gateway.bind <bind>
  → if lan: configure control UI allowed origins

Stage 9: "Restarting gateway"
  → docker compose -p easyclaw-<containerId> up -d (pick up config changes)
```

If any stage fails, emit error via `build-progress` event with the stage name and error output. The frontend shows the failure point and log. The user can retry (re-runs from stage 1) or go back to edit settings.

**Cancellation:** The build command checks a cancellation flag between stages. The frontend sends a `cancel_build` command that sets this flag. The currently running Docker command is killed via process termination.

## 7. Security Architecture

### Agent Execution Model

**V1: agent-in-gateway mode only (no sandbox).**

The OpenClaw agent runs within the gateway container process. The container itself provides isolation:
- No access to the host filesystem
- No Docker socket mounted
- No host network access (only the gateway/bridge ports are mapped)
- `OPENCLAW_SANDBOX` is always disabled
- `OPENCLAW_DOCKER_SOCKET` is not set
- `OPENCLAW_INSTALL_DOCKER_CLI` is not set

This is the most secure configuration when the gateway itself runs in Docker — the container IS the sandbox. There is no need for Docker-socket-based sandboxing in V1 since it would actually reduce security by exposing the host Docker daemon.

**Future (V2+):** If agent sandboxing is needed, investigate Docker-in-Docker (nested Docker daemon inside the container) so sandbox containers are isolated from the host. This avoids ever exposing the host Docker socket.

### Token Generation

Gateway tokens: `rand` crate with `OsRng` (cryptographically secure). Generate 32 bytes, encode as hex (64 characters).

### File Permissions

Instance directories created with user-default permissions. No sensitive data is world-readable (gateway tokens are in instance.json which inherits the user's home directory permissions).

## 8. Error Handling Strategy

### Rust Backend

All public functions return `Result<T, EasyClawError>`. Custom error type:

```rust
#[derive(Debug, thiserror::Error)]
pub enum EasyClawError {
    #[error("Docker is not running")]
    DockerNotRunning,
    #[error("Docker is not installed")]
    DockerNotInstalled,
    #[error("Docker command failed: {0}")]
    DockerCommand(String),
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Filesystem error: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("Build failed at stage '{stage}': {message}")]
    BuildFailed { stage: String, message: String },
    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for EasyClawError { /* for Tauri IPC */ }
```

Tauri commands convert `EasyClawError` to serializable error strings for the frontend. The frontend displays these in appropriate UI contexts (inline errors, toast notifications, build failure screens).

### Frontend

- Commands that fail: catch the error, display in the relevant UI location
- Network/transient errors: offer retry where applicable
- Fatal errors (corrupted state): surface clearly, offer "Delete Instance" as recovery

### Logging

Use `tracing` crate on the Rust side. Log to:
- stderr (visible in dev)
- `~/.easyclaw/logs/easyclaw.log` (rotating, for debugging production issues)

Log levels: `error` for failures, `warn` for recoverable issues, `info` for lifecycle events (instance created, build started), `debug` for Docker command output.

## 9. Testing Strategy

### Rust Backend

**Unit tests** for pure logic:
- `names.rs`: generates valid names, handles collisions
- `ports.rs`: allocation logic, validation
- `compose_gen.rs`: generated YAML matches expected structure
- `env_gen.rs`: generated .env matches expected format
- `models.rs`: serialization/deserialization roundtrips

**Integration tests** (require Docker):
- `docker/cli.rs`: check_available, build, compose up/down
- Full build flow: create instance → build → verify container running → stop → delete

Integration tests are gated behind a `#[cfg(feature = "integration-tests")]` flag so they don't run in CI without Docker.

### Frontend

**Component tests** (vitest + @testing-library/svelte):
- `ConfigForm.svelte`: renders all fields, validates input, emits correct settings
- `InstanceCard.svelte`: renders different states correctly
- `BuildProgress.svelte`: updates stages from events

**E2E tests** (Playwright or similar, stretch goal for V1):
- Full wizard flow with mocked Tauri commands
- Instance list interactions

### Test commands

```bash
# Rust unit tests
cargo test

# Rust integration tests (requires Docker)
cargo test --features integration-tests

# Frontend unit/component tests
npm run test

# All checks
npm run check     # svelte-check + TypeScript
npm run lint      # ESLint
npm run format:check  # Prettier
```

## 10. Key Dependencies

### Rust (Cargo.toml)

| Crate | Purpose |
|-------|---------|
| `tauri` (v2) | App framework, IPC, window management |
| `tokio` | Async runtime (Tauri's default) |
| `serde` + `serde_json` | Serialization for instance configs, IPC |
| `serde_yaml` | docker-compose.yml generation |
| `reqwest` | HTTP client for GitHub API, Dockerfile fetch |
| `rand` | Instance ID + name generation, token generation |
| `thiserror` | Error type derivation |
| `tracing` + `tracing-subscriber` | Structured logging |
| `dirs` | Cross-platform home directory resolution |

### Frontend (package.json)

| Package | Purpose |
|---------|---------|
| `@sveltejs/kit` | App framework |
| `@sveltejs/adapter-static` | Static SPA build for Tauri |
| `svelte` | UI framework |
| `@tauri-apps/api` | Tauri IPC from frontend |
| `@tauri-apps/plugin-shell` | Open URLs in browser |
| `shadcn-svelte` | UI component library |
| `tailwindcss` | Utility CSS (shadcn dependency) |
| `bits-ui` | Headless UI primitives (shadcn dependency) |
| `typescript` | Type checking |
| `vitest` | Unit/component testing |
| `@testing-library/svelte` | Component test utilities |
| `eslint` + `eslint-plugin-svelte` | Linting |
| `prettier` + `prettier-plugin-svelte` | Formatting |

## 11. CI / CD

### GitHub Actions: CI

Triggered on push and PR:

```yaml
jobs:
  check:
    - npm ci
    - npm run check          # svelte-check + tsc
    - npm run lint
    - npm run format:check
    - npm run test           # vitest
  rust:
    - cargo fmt -- --check
    - cargo clippy --all-targets -- -D warnings
    - cargo test
```

### GitHub Actions: Release

Triggered when a GitHub Release is created (tag push):

```yaml
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest      # Apple Silicon
            target: aarch64-apple-darwin
          - os: macos-13          # Intel
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
    steps:
      - npm ci
      - npm run build
      - cargo tauri build --target ${{ matrix.target }}
      - Upload artifacts to GitHub Release
```

Uses `tauri-apps/tauri-action` GitHub Action which handles building and uploading platform-specific artifacts (.dmg, .msi, .AppImage, .deb).

## 12. Platform Considerations

### Docker Socket Detection

```rust
pub fn docker_socket_path() -> PathBuf {
    if let Ok(host) = std::env::var("DOCKER_HOST") {
        if let Some(path) = host.strip_prefix("unix://") {
            return PathBuf::from(path);
        }
    }
    #[cfg(target_os = "windows")]
    return PathBuf::from(r"\\.\pipe\docker_engine");

    #[cfg(not(target_os = "windows"))]
    return PathBuf::from("/var/run/docker.sock");
}
```

### Home Directory

```rust
pub fn easyclaw_dir() -> PathBuf {
    dirs::home_dir().expect("No home directory").join(".easyclaw")
}
```

On Windows this resolves to `C:\Users\<name>\.easyclaw\`. On macOS/Linux: `~/.easyclaw/`.

### Path Handling

All paths stored in instance.json use platform-native format. Docker volume mount paths in docker-compose.yml use forward slashes on all platforms (Docker Desktop normalizes). The compose generator handles this conversion.
