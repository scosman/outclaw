---
status: complete
---

# Functional Spec: OutClaw

A cross-platform desktop application for setting up, running, and managing OpenClaw instances inside Docker containers. Targets non-technical users who want to run a personal AI agent without touching the command line.

## 1. Core Concepts

### Instance

An **instance** is a self-contained OpenClaw installation: its own Docker container, config directory, workspace directory, and port assignments. Users can create and manage multiple instances independently.

Each instance tracks:

- **Name**: user-chosen, unique, displayed in the UI
- **OpenClaw version**: which GitHub release was used to build the image
- **Docker container ID/name**: the running (or stopped) container
- **Status**: building, running, stopped, error, docker-not-running
- **Config directory**: under `~/.outclaw/instances/<id>/config/`
- **Workspace directory**: under `~/.outclaw/instances/<id>/workspace/`
- **Docker build settings**: the options chosen during setup (ports, bind mode, packages, extensions, etc.)
- **Created/last modified timestamps**

### Versions (Future — V2+)

The directory structure and data model are designed to support instance versioning. In a future release, every config or Docker image change will create an automatic version snapshot, and users will be able to roll back. V1 does not implement versioning or rollback — it just writes to a structure that won't conflict with it.

Planned structure (for reference, not implemented in V1):

```
~/.outclaw/instances/<id>/versions/
  v1/
    config/          ← snapshot of config at this version
    Dockerfile       ← Dockerfile used for this build
    metadata.json    ← timestamp, openclaw version, build settings
  v2/
    ...
```

### Directory Layout

```
~/.outclaw/
  app-state.json              ← OutClaw app state (window position, last active instance, etc.)
  docker-containers/          ← Generated Dockerfiles, docker-compose files, build context
    <container-id>/           ← each container has its own ID (separate from instance ID)
      Dockerfile
      docker-compose.yml
      docker-compose.extra.yml (if needed)
      .env
  instances/
    <instance-id>/
      instance.json            ← instance metadata (name, version, ports, container ref, settings)
      config/                  ← mounted as OpenClaw config dir
        identity/
        agents/main/agent/
        agents/main/sessions/
      workspace/               ← mounted as OpenClaw workspace dir
```

Containers have their own IDs separate from instance IDs. An instance references a container via `containerId` in `instance.json`. This separation allows future features like forking (two instances sharing a container's image/config baseline).

Instance IDs are generated (e.g., short random alphanumeric like `ec_a1b2c3`). Names are user-facing and can contain spaces/special characters; IDs are filesystem-safe.

## 2. Prerequisites & Docker Detection

### Docker Desktop Requirement

OutClaw requires Docker Desktop to be installed and running. It does not install Docker itself.

**Detection logic (polled):**

1. Check if the Docker socket exists and is responsive (platform-specific path)
2. Run `docker info` or equivalent API call to confirm Docker is running
3. Run `docker compose version` to confirm Compose is available

**States:**

- **Docker not installed**: Show instructions to download Docker Desktop with a link. Platform-specific instructions (macOS: `.dmg` download, Windows: installer download).
- **Docker installed but not running**: Show "Docker Desktop is not running. Please start it." with a retry/poll button. Auto-detect when it starts (poll every 3–5 seconds).
- **Docker running**: Proceed normally.

The app shows Docker status globally in the UI (e.g., status bar or header indicator). If Docker stops while the app is open, surface this clearly — instance statuses should reflect "docker not running" rather than "stopped."

## 3. Screens & Navigation

### 3.1 Main Screen (Instance List)

The landing screen after setup wizard completion. Shows:

- **Header**: App name/logo, global Docker status indicator
- **Instance list**: Cards or rows, each showing:
  - Instance name
  - Status badge (running / stopped / error / building)
  - OpenClaw version
  - Quick actions: Start / Stop / Open (gateway URL in browser)
- **"New Instance" button**: Launches the setup wizard
- **Empty state**: When no instances exist, show a welcome message and prominent "Create your first instance" CTA

Clicking an instance navigates to the Instance Detail screen.

### 3.2 Setup Wizard (New Instance)

A multi-step wizard for creating a new instance. Steps flow linearly with back/next navigation.

#### Step 1: Docker Check

- If Docker is running: auto-advance (user may never see this step)
- If not: show install/start instructions, poll for Docker availability, advance when detected

#### Step 2: Install Type

Two options presented as large selectable cards:

- **Standard Install**: "Recommended for most users. Uses sensible defaults."
- **Custom Install**: "For advanced users. Configure ports, networking, extensions, and more."

Selecting Standard Install skips to Step 4 (build) with all defaults.

#### Step 3: Custom Configuration (Custom Install only)

A form with OpenClaw Docker build options. Organized into sections. Each option has:

- A user-friendly title
- A short description
- An extended tooltip or help text (expandable)
- A sensible default pre-filled

**Basic Options** (always visible in Custom Install):

| Setting                 | UI Title           | Default                           | Control                                     | Notes                                                                                                                                                                                                            |
| ----------------------- | ------------------ | --------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Instance name           | "Instance Name"    | "My OpenClaw"                     | Text input                                  | User-chosen, must be unique                                                                                                                                                                                      |
| `OPENCLAW_IMAGE`        | "OpenClaw Version" | Latest release                    | Dropdown (fetched from GitHub Releases API) | Each option is a release tag. Dropdown fetches and caches releases.                                                                                                                                              |
| `OPENCLAW_GATEWAY_PORT` | "Gateway Port"     | 18789 (auto-incremented if taken) | Number input                                | Validated: 1024–65535, not already used by another instance                                                                                                                                                      |
| `OPENCLAW_BRIDGE_PORT`  | "Bridge Port"      | 18790 (auto-incremented if taken) | Number input                                | Same validation                                                                                                                                                                                                  |
| `OPENCLAW_GATEWAY_BIND` | "Network Access"   | "Local only (localhost)"          | Toggle/select: "Local only" / "LAN access"  | Controls whether the gateway is reachable from other devices. "Local only" = `loopback`, "LAN access" = `lan`. Does not affect the container's own internet access — OpenClaw always has full outbound internet. |
| `OPENCLAW_TZ`           | "Timezone"         | Detected from system              | Searchable dropdown                         | Pre-populated with system timezone, full IANA list available                                                                                                                                                     |
| Browser install         | "Install Browser"  | Off                               | Toggle                                      | Installs a browser in the Docker image for web browsing capabilities                                                                                                                                             |

**Advanced Options** (collapsed section, expandable):

| Setting                              | UI Title                     | Default              | Control    | Notes                                     |
| ------------------------------------ | ---------------------------- | -------------------- | ---------- | ----------------------------------------- |
| `OPENCLAW_DOCKER_APT_PACKAGES`       | "Additional System Packages" | Empty                | Text input | Space-separated apt package names         |
| `OPENCLAW_EXTENSIONS`                | "Extensions"                 | Empty                | Text input | Space-separated extension identifiers     |
| `OPENCLAW_HOME_VOLUME`               | "Home Volume"                | (empty, use default) | Text input | Named Docker volume or host path          |
| `OPENCLAW_EXTRA_MOUNTS`              | "Extra Volume Mounts"        | Empty                | Text area  | Comma-separated `source:target[:options]` |
| `OPENCLAW_ALLOW_INSECURE_PRIVATE_WS` | "Allow Insecure WebSocket"   | Off                  | Toggle     | Shows warning when enabled                |

**Not exposed in UI (fixed values managed by OutClaw):**

| Setting                       | Behavior                                                                                                                                                 |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `OPENCLAW_CONFIG_DIR`         | Managed by OutClaw, set to instance config path                                                                                                         |
| `OPENCLAW_WORKSPACE_DIR`      | Managed by OutClaw, set to instance workspace path                                                                                                      |
| `OPENCLAW_GATEWAY_TOKEN`      | Auto-generated (secure random hex, 32 bytes). Stored in instance metadata. User can view/copy from Instance Detail screen but never asked to create one. |
| `OPENCLAW_DOCKER_SOCKET`      | Not set. V1 does not mount the Docker socket (agent-in-gateway mode).                                                                                    |
| `OPENCLAW_SANDBOX`            | Always disabled. No sandbox in V1 — the container itself is the isolation boundary.                                                                      |
| `OPENCLAW_INSTALL_DOCKER_CLI` | Not set. No Docker CLI needed inside the container in V1.                                                                                                |

**Instance name generation:**
Auto-generated using a two-word random name generator (adjective + animal), e.g., "Magical Buffalo", "Swift Falcon", "Cosmic Otter". Ship with curated word lists — adjectives should feel fun/positive, nouns should be memorable animals/creatures. User can always edit the name (pre-filled in Custom Install, editable on Instance Detail screen). If a collision occurs, regenerate.

**Standard Install defaults:**
All basic options at their defaults. Instance name auto-generated. Latest OpenClaw version. Ports auto-assigned (first available starting from 18789/18790). System timezone.

#### Step 4: Building

- Show a progress screen with build output streaming
- Stages: "Generating Dockerfile..." → "Building Docker image..." → "Starting container..." → "Running initial setup..."
- ASCII art / styled loading animation during build
- Build errors: show error message with "Retry" and "Back to Settings" options
- The build process:
  1. Generate Dockerfile from selected options (equivalent to what the setup script does)
  2. Generate docker-compose.yml
  3. Write `.env` file with all settings
  4. Run `docker build`
  5. Create config/workspace directory structure (with correct subdirectories)
  6. Run `docker compose up -d` to start the container
  7. Run onboarding: `docker compose run --rm openclaw-cli onboard --mode local --no-install-daemon`
  8. Fix data directory permissions (chown to node user)
  9. Sync gateway mode and bind settings
  10. Configure control UI allowed origins if LAN mode

#### Step 5: AI Provider Setup

After the container is running:

**V1 (early implementation):** Display instructions to run provider setup via CLI:

```
To set up your AI provider, run this command in a terminal:
docker compose -f <path-to-compose> run --rm openclaw-cli configure
```

Provide a "Copy Command" button. Show a "Skip for now" and "Done" button.

**V1 (later implementation):** Native UI for provider setup. Likely a form where user selects a provider, enters API base URL and key, and we test the connection by writing directly to the OpenClaw config file and hitting the provider's API. Detailed design will be done when this phase is planned.

#### Step 6: Chat Channel Setup

**V1 (early implementation):** Display instructions for each supported channel:

- **Telegram**: Show command to add Telegram bot token:
  `docker compose -f <path> run --rm openclaw-cli channels add --channel telegram --token <token>`
  Brief instructions on how to get a bot token from BotFather.

- **WhatsApp**: Show command for WhatsApp QR login:
  `docker compose -f <path> run --rm openclaw-cli channels login`
  Note that this opens an interactive QR code flow.

Provide "Copy Command" buttons, "Skip for now" and "Done" buttons.

**V1 (later implementation):** Native UI for channel setup. Detailed design will be done when this phase is planned.

#### Step 7: Complete

- Success screen with instance summary (name, version, gateway URL, status)
- "Open Gateway" button (opens gateway URL in browser)
- "Go to Dashboard" button (returns to main screen)

### 3.3 Instance Detail Screen

Accessed by clicking an instance from the main screen.

**Header area:**

- Instance name (editable inline)
- Status badge
- OpenClaw version
- Start / Stop / Restart buttons (contextual based on status)
- "Open Gateway" button (opens `http://localhost:<port>` in default browser)

**Info section:**

- Gateway URL (copyable)
- Gateway token (copyable, masked by default with reveal toggle)
- Gateway port / Bridge port
- Network bind mode
- Config directory path (copyable)
- Workspace directory path (copyable)
- Container ID (copyable)

**Actions section:**

- **Rebuild**: Re-run Docker build with current settings (e.g., after changing version)
- **Edit Settings**: Open the configuration form (same as wizard Step 3) pre-filled with current values. Saving triggers a rebuild.
- **Provider Setup**: Opens provider setup (CLI instructions in early V1, native UI later)
- **Channel Setup**: Opens channel setup (CLI instructions in early V1, native UI later)
- **Delete Instance**: Confirmation dialog → stops container, removes Docker image, deletes instance directory. Warns that workspace data will be lost.

**Logs section (stretch):**

- Tail of container logs (stdout/stderr)
- Not required for V1 launch, but valuable

## 4. Error Handling

### Docker Errors

| Scenario                       | Behavior                                                                                         |
| ------------------------------ | ------------------------------------------------------------------------------------------------ |
| Docker not running             | Global banner: "Docker Desktop is not running." All instance statuses show "Docker not running." |
| Docker stops while app is open | Detect via polling (every 10s). Update status globally.                                          |
| Build fails                    | Show error output on build screen. "Retry" and "Back to Settings" buttons.                       |
| Container fails to start       | Instance status = "error". Instance detail shows last error from container logs.                 |
| Container crashes              | Detected via polling. Status updates to "error".                                                 |
| Port already in use            | During wizard: validate before build, show inline error. Suggest next available port.            |

### Network Errors

| Scenario                                  | Behavior                                                                                                                                                                                                               |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Can't fetch GitHub releases               | Show cached releases if available with a warning that the list may be outdated. If no cache exists, show an error: "Internet connection required to fetch available OpenClaw versions." Block creation until resolved. |
| Docker Hub unreachable (if pulling image) | Show error during build step with retry option.                                                                                                                                                                        |

### Data Errors

| Scenario                             | Behavior                                                       |
| ------------------------------------ | -------------------------------------------------------------- |
| `~/.outclaw` doesn't exist          | Create it on first launch.                                     |
| Instance directory corrupted/missing | Show instance with "error" status. Offer "Delete" to clean up. |
| `instance.json` can't be parsed      | Same as corrupted — surface error, offer delete.               |
| Port conflict between instances      | Detect during creation. Prevent saving duplicate ports.        |

## 5. Docker Build Configuration Details

OutClaw generates a Dockerfile and docker-compose.yml per instance. The generated files are equivalent to what the OpenClaw setup script produces, but generated programmatically rather than by running the bash script.

### Dockerfile Strategy

The OpenClaw Dockerfile is a multi-stage build that `COPY`s the full source tree into the image — it cannot be built in isolation. OutClaw downloads the complete release source:

1. **Fetch the release source**: Download the release tarball from `https://github.com/openclaw/openclaw/archive/refs/tags/{tag}.tar.gz` and extract to `~/.outclaw/source-cache/{tag}/`. Cached per tag so multiple instances on the same version share the download.
2. **Replace setup.sh logic**: The setup script's job (setting environment variables, generating docker-compose, writing .env) is replaced entirely by OutClaw's programmatic generation. We do not run or ship the setup script.
3. **Apply build-time arguments**: Pass user-selected options as build args (`OPENCLAW_DOCKER_APT_PACKAGES`, `OPENCLAW_EXTENSIONS`, `OPENCLAW_INSTALL_DOCKER_CLI`, browser install) to the Dockerfile in the cached source.
4. **Build context**: The `source-cache/{tag}/` directory (containing the Dockerfile and all referenced source files) is the Docker build context. The per-instance `docker-containers/<containerId>/` directory holds only the generated docker-compose.yml and .env.

### docker-compose.yml Generation

Generated per-instance with:

- Service name namespaced to instance (e.g., `outclaw-<id>-gateway`, `outclaw-<id>-cli`)
- Port mappings from instance settings
- Volume mounts for config and workspace directories
- Environment variables from instance settings
- Gateway token
- Timezone if set

### .env File

All instance settings written as environment variables, matching the format the OpenClaw setup script expects.

## 6. Instance Lifecycle

```
[New Instance Wizard] → Building → Running ⇄ Stopped
                                      ↓
                                    Error
```

**State transitions:**

- **Building**: Dockerfile generated, `docker build` in progress. No user interaction except cancel.
- **Running**: Container is up. Gateway accessible.
- **Stopped**: Container exists but is stopped. Can be started.
- **Error**: Container crashed or failed to start. Show diagnostics.
- **Docker not running**: Docker Desktop is not running. Distinct from stopped (not the instance's fault).

**Polling**: The app polls Docker for container status. Frequency:

- When app is in foreground: every 5 seconds
- When app is in background: every 30 seconds (or pause entirely — Tauri can detect focus)

## 7. Port Management

Multiple instances require unique port assignments. OutClaw manages this:

- **Default ports**: Gateway 18789, Bridge 18790
- **Auto-increment**: When creating a new instance, if default ports are taken by another instance, increment (18791/18792, etc.)
- **Validation**: Check against all existing instances. Also attempt to bind-check against the OS to catch non-OutClaw port conflicts.
- **Display**: Show assigned ports clearly in instance detail and main screen tooltips.

## 8. Security

### Gateway Token

- Auto-generated on instance creation using cryptographically secure random (32-byte hex)
- Stored in `instance.json` and written to the container's config
- Displayed in Instance Detail screen, masked by default
- Never asked of the user; never transmitted outside the local machine

### Network Bind

- Default: `loopback` (localhost only) — the safe default for non-technical users
- This controls whether the gateway HTTP server listens on localhost or all interfaces. It does **not** restrict the container's own outbound internet access — OpenClaw always has full internet access (required for AI providers, web browsing, etc.)
- LAN mode available in Custom Install with clear description of what it enables: "Allows other devices on your local network to access this OpenClaw instance"
- LAN mode configures CORS allowlist automatically

### Agent Execution Security

V1 uses agent-in-gateway mode exclusively. The agent runs within the gateway container process. The container itself is the security boundary:

- No Docker socket mounted — no host Docker access
- No host filesystem access beyond explicitly mounted config/workspace dirs
- No sandbox mode — unnecessary when the gateway is already containerized, and would actually reduce security by exposing the host Docker daemon
- Future: Docker-in-Docker may be explored in V2+ if per-agent sandboxing is needed

## 9. OpenClaw Version Management

### Fetching Available Versions

- Query the OpenClaw GitHub Releases API: `https://api.github.com/repos/openclaw/openclaw/releases`
- Cache the response locally (in `~/.outclaw/`) with a TTL (e.g., 1 hour)
- Show release tag name, publication date, and whether it's the latest
- Pre-select the latest stable (non-prerelease) release

### Upgrading an Instance

- User selects a new version from Instance Detail → Edit Settings
- Triggers a rebuild (new Dockerfile with updated version, `docker build`, restart container)
- Old container is stopped and removed; new one takes its place
- V2+: this will create a version snapshot before upgrading

## 10. Platform-Specific Behavior

### macOS

- Docker socket: `/var/run/docker.sock` or as reported by `DOCKER_HOST`
- File permissions: Docker Desktop for Mac handles uid mapping transparently in most cases, but we still run the chown step for safety
- Distribution: `.dmg` containing the `.app` bundle (Universal binary: Intel + Apple Silicon)

### Windows

- Docker socket: named pipe `//./pipe/docker_engine` or TCP
- File paths: use platform-appropriate paths (backslash handling, `%USERPROFILE%` instead of `~`)
- Config directory: `%USERPROFILE%\.outclaw\`
- Distribution: `.msi` or `.exe` installer
- Docker Desktop must be running with WSL2 backend

### Linux

- Docker socket: `/var/run/docker.sock`
- May require user to be in the `docker` group
- Distribution: `.AppImage` and `.deb`

## 11. Style & Design

- **Theme**: shadcn-svelte dark mode as the default and primary theme
- **Typography**: Monospace font throughout (e.g., JetBrains Mono, Fira Code, or similar) for a "hip technical" feel
- **Aesthetic**: Clean, modern, dark. Not terminal-emulator-retro — more like a well-designed developer tool
- **Loading/splash**: Styled ASCII art for the app name/logo on loading screens and headers
- **Components**: Use shadcn-svelte component library as the design system
- **Interactions**: Minimal animations. Functional, not flashy. Clear status indicators.

## 12. Out of Scope (V1)

- **Instance versioning / rollback**: Directory structure supports it; implementation is V2+
- **Instance forking**: Future feature
- **Native provider setup UI**: V1 shows CLI instructions; native UI is a later implementation phase
- **Native channel setup UI**: Same as above
- **Light theme**: Dark mode only in V1
- **Auto-updates for OutClaw itself**: Use GitHub Releases; users download new versions manually
- **Remote instances**: All instances are local Docker containers
- **Docker installation**: Users must install Docker Desktop themselves
- **Container log viewer**: Stretch goal, not required for V1 launch
