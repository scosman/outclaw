---
status: complete
---

# Project: EasyClaw

I want build a visual app for setting up and running OpenClaw inside a docker container. Context: OpenClaw is a personal agent system, which can be run in docker.

They publish a setup script that walks users through `specs/references/setup.sh`. We’re essentially automating this setup, and future management, for non-technical users.

### High Level UI Flow

- Download a app (MacOS, windows)
- Launch it
- Walk through a wizard for initial setup: 
  - ask them to install docker desktop, wait until it’s running (check socket)
  - ask them for config options. “Standard Install” vs “Custom Install (Advanced)”
  - Custom Install: UI with options mapping to the docker setup script, explaining each. User friendly. Good defaults.
  - Build Dockerfile (like the setup file) and docker build
  - Start container
  - Ask user which AI provider to use, set them up with keys, confirming they work before moving on.
  - Ask user which chat channels they want to setup. Support Telegram and WhatsApp in UI in V1.
- Management
  - Main screen
    - List instances
    - New instance button
    - Status: docker not running, etc.
  - Instance screen
  - Fresh Install
  - Backup settings files / docker images

### Tech Stack

- App framework: Tauri. 
- Language: typescript with type checking. No untyped JS.
- Web frameworks: SvelteKit, shadcn-svelte
- Docker Desktop: required, user must install separately and have running. Detect if running
- Hosting: Github for repo, Github actions for CI, Github actions for release builds, Github releases for release hosting. 
- CI: usual excellent stack for tauri dev: (you decide, this is rough): svelte-check, eslint, type checking, etc.
- Releases: Github action triggered by release creation. Builds and attaches tauri app builds for Mac (intel and arm), Windows, and linux.

### Principals:

- User friendly: 
  - Anyone can run it and successfully install openclaw
- Don’t ask technical questions to non technical users.
  - Example: don’t ask them to generate gateway key, use a secure random.
  - Don’t ask them confusing questions a typical user wouldn’t understand. And there are tiers: Default Install == anyone can do it, Custom Install == more technical user, Custom Install + Advanced expansion section == most advanced options.
- Secure by default
  - The gateway running in docker, and is secure by default (no host system access). Enabling host system access has warnings.
- Minimize coupling
  - OpenClaw changes often. Really often. If we couple too deeply to it, we’ll be breaking non-stop. 
  - I’m okay automating/replacing the setup script for docker, but preferably that’s all.
- Isolation and Robustness:
  - have the concept of “instances”. I might want to install 3 open claws locally, each with their own config dir, own docker container, and own workspace. I can upgrade/manage them separately. Future: I can “fork” an instance.
  - Instances have versions. Anytime you edit the config, or re-build the docker image, you  get a new version. We save these older versions (openclaw config directory, Dockerfile), and can restore/roll back any time.
  - The workspace directory (its data DIR) is not versioned/saved. One workspace per instance.

### Settings

- Config directory: we manage completely
  - `~/.easyclaw` is our root directory, all of our app data is saved here (including openclaw data like config and workspace). 
  - Don’t mount this root ever, use sub-folders. We want to be able to save non-mounted data (config backups, docker images, etc). See instances/versions above.

### Resolved: Provider & Chat Setup Approach

- How do we setup chat providers and AI providers? `openclaw configure` has decent setup TUI already for WhatsApp, Telegram and AI providers. But it’s a TUI, and also has more advanced options.
  - Option 1:
    - Is there a good way to wrap this TUI in tauri (docker exec the config, web-terminal)?
    - If we wrap their TUI, can we deep link to the right thing (connect WhatsApp, connect provider) without showing all the config UI.
  - Option 2:
    - Is the config for these simple enough to just write our own webUI + script to manage? I’m worried about providers (about 20 options, changing often). But maybe it’s all “OpenAI compatible API” and we can just have AI coding agents update occasionally? I imagine they maintain backwards compatibility because people have existing configs so maybe my compatibility concerns aren’t too bad if all these do is modify the config file.
**Resolved:** Early V1 shows CLI instructions. Later V1 phase builds native UI (simple forms writing to OpenClaw config files). Detailed design deferred to implementation phase.

### Docker Setup Config Summary

Here’s an agent’s summary of the config options. Part of the functional spec should be a list of each build option
1) do we support setting it or only allow a fixed value/default
2) which values do we support
3) What’s it’s UI: title, description and extended tooltip text.

```
Docker Image
	•	OPENCLAW_IMAGE — Image to use. Defaults to openclaw:local (triggers local build); any other value triggers docker pull
Directory Paths
	•	OPENCLAW_CONFIG_DIR — Config directory. Defaults to ~/.openclaw. Gets bind-mounted as /home/node/.openclaw
	•	OPENCLAW_WORKSPACE_DIR — Workspace directory. Defaults to ~/.openclaw/workspace. Bind-mounted inside the config dir mount
Networking
	•	OPENCLAW_GATEWAY_PORT — Gateway HTTP port. Defaults to 18789
	•	OPENCLAW_BRIDGE_PORT — Bridge port. Defaults to 18790
	•	OPENCLAW_GATEWAY_BIND — Network bind mode: loopback (localhost only) or lan (default). Controls CORS allowlist behavior and is persisted into OpenClaw config
Authentication
	•	OPENCLAW_GATEWAY_TOKEN — Auth token for the gateway. Resolution order: env var → openclaw.json config → .env file → auto-generated via openssl rand -hex 32 or Python secrets
Sandbox / Docker-in-Docker
	•	OPENCLAW_SANDBOX — Truthy value (1/true/yes/on) enables sandbox mode. Mounts Docker socket, builds sandbox image, configures mode=non-main, scope=agent, workspaceAccess=none
	•	OPENCLAW_DOCKER_SOCKET — Path to Docker socket. Falls back to DOCKER_HOST (stripping unix:// prefix), then /var/run/docker.sock
Container Customization
	•	OPENCLAW_DOCKER_APT_PACKAGES — Space-separated apt packages to install at build time (passed as build arg)
	•	OPENCLAW_EXTENSIONS — Extensions to install at build time (passed as build arg)
	•	OPENCLAW_INSTALL_DOCKER_CLI — Auto-set to 1 when sandbox is enabled; installs Docker CLI in the image
Volume & Mount Configuration
	•	OPENCLAW_HOME_VOLUME — Named Docker volume or host path for /home/node. If it contains / it's treated as a host path; otherwise as a named volume (validated against [A-Za-z0-9][A-Za-z0-9_.-]*)
	•	OPENCLAW_EXTRA_MOUNTS — Comma-separated list of additional bind mounts in source:target[:options] format. Applied to both openclaw-gateway and openclaw-cli services
Timezone
	•	OPENCLAW_TZ — IANA timezone string (e.g. America/Toronto). Validated against /usr/share/zoneinfo
Misc
	•	OPENCLAW_ALLOW_INSECURE_PRIVATE_WS — Passed through to the container; presumably allows unencrypted WebSocket connections on private networks

```

Install browser into docker: is this an option? It should be. 

### Non Docker Config

- openclaw version: show a dropdown of releases pulled from openclaw GitHub Releases to determine which to download. Should download the source from the GitHub release, not git sync.
- Chat setup: Telegram, WhatsApp
- Provider setup: select provider, connect keys, test
- What else?

### Style/Design

We want a bit of style/design sense.

- ShadCN dark style followed by default, it’s quite good.
- hip “nerd” feel (think CLAUDE CODE ascii art logo) for loading screens, headers/etc