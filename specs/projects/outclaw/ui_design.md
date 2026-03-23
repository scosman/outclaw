---
status: complete
---

# UI Design: OutClaw

Desktop application UI built with SvelteKit + shadcn-svelte. Single-window app with simple view-based navigation. Dark theme, monospace typography, clean developer-tool aesthetic.

## 1. Global Layout

```
┌──────────────────────────────────────────────────┐
│  Header Bar                                      │
│  [Logo/Name]              [Docker Status] [⚙]    │
├──────────────────────────────────────────────────┤
│                                                  │
│                                                  │
│                 Content Area                     │
│                                                  │
│                                                  │
│                                                  │
└──────────────────────────────────────────────────┘
```

### Header Bar

- Fixed at top, always visible (except during wizard flow, which is full-screen)
- **Left**: App logo — "OutClaw" in styled monospace, small ASCII art flourish (e.g., a claw mark `>///<` or similar). Keep it compact, not a full splash graphic.
- **Right**: Docker status pill (green dot + "Running" / red dot + "Not Running" / yellow dot + "Not Installed"), settings gear icon
- Height: ~48px. Subtle bottom border separating from content.
- Background: slightly lighter than content area (shadcn `card` background or similar)

### Content Area

- Fills remaining window space
- Scrollable when content overflows
- Switches between views: Instance List, Instance Detail, Setup Wizard

### Window

- Default size: 900×640px
- Minimum size: 720×500px
- Resizable. Content reflows within min/max but this is a desktop app — no mobile breakpoints needed.
- Tauri window decorations: use native title bar on macOS, custom on Windows (standard Tauri pattern)

## 2. Docker Status Overlay

When Docker is not running or not installed, an overlay covers the content area (header remains visible). The overlay is not dismissible — it blocks interaction until Docker is available.

### Docker Not Installed

```
┌──────────────────────────────────────┐
│                                      │
│        🐳  Docker Required           │
│                                      │
│   OutClaw needs Docker Desktop      │
│   to run OpenClaw instances.         │
│                                      │
│   [ Download Docker Desktop ]        │
│                                      │
│   Platform-specific install steps:   │
│   1. Download from docker.com        │
│   2. Run the installer               │
│   3. Launch Docker Desktop           │
│                                      │
│   Waiting for Docker...  ◌           │
│                                      │
└──────────────────────────────────────┘
```

- "Download Docker Desktop" is a button that opens the browser to the Docker download page
- Spinner at bottom with polling indicator
- Auto-advances when Docker is detected

### Docker Not Running

```
┌──────────────────────────────────────┐
│                                      │
│   Docker Desktop is not running.     │
│                                      │
│   Please start Docker Desktop.       │
│                                      │
│   Waiting for Docker...  ◌           │
│                                      │
└──────────────────────────────────────┘
```

- Simpler variant — just a message and spinner
- Auto-advances when Docker responds

## 3. Instance List View

The default view when the app has at least one instance and Docker is running.

### With Instances

```
┌──────────────────────────────────────────────────┐
│  Header Bar                                      │
├──────────────────────────────────────────────────┤
│                                                  │
│  Instances                        [ + New ]      │
│                                                  │
│  ┌────────────────────────────────────────────┐  │
│  │  ● Cosmic Otter              v0.42.1       │  │
│  │    Running · localhost:18789               │  │
│  │                    [Open]  [Stop]          │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌────────────────────────────────────────────┐  │
│  │  ○ Swift Falcon              v0.41.0       │  │
│  │    Stopped                                │  │
│  │                   [Open]  [Start]          │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
│  ┌────────────────────────────────────────────┐  │
│  │  ⚠ Midnight Raven            v0.42.1       │  │
│  │    Error · Container exited unexpectedly   │  │
│  │                  [Details]  [Restart]       │  │
│  └────────────────────────────────────────────┘  │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Instance cards:**

- Full-width cards in a vertical stack
- Left: status indicator dot (green=running, gray=stopped, red=error, blue=building, yellow=docker-not-running)
- Instance name: prominent, left-aligned
- Version tag: right-aligned, muted text
- Second line: status text + gateway URL if running
- Action buttons: right-aligned on the card. Contextual:
  - Running: "Open" (opens gateway in browser) + "Stop"
  - Stopped: "Open" (disabled/hidden) + "Start"
  - Error: "Details" + "Restart"
  - Building: progress indicator, no actions
- Entire card is clickable → navigates to Instance Detail
- Action buttons stop click propagation (don't navigate)

**"+ New" button:**

- Top right, prominent. shadcn `Button` variant `default`.
- Launches the setup wizard.

### Empty State

```
┌──────────────────────────────────────────────────┐
│  Header Bar                                      │
├──────────────────────────────────────────────────┤
│                                                  │
│                                                  │
│               >///<                              │
│            OutClaw                              │
│                                                  │
│     Manage OpenClaw instances with ease.         │
│                                                  │
│        [ Create Your First Instance ]            │
│                                                  │
│                                                  │
└──────────────────────────────────────────────────┘
```

- Centered vertically and horizontally
- ASCII art logo (larger version than header)
- Tagline
- Single prominent CTA button
- Clean, welcoming, not overwhelming

## 4. Setup Wizard

Full-screen flow that replaces the content area. Header bar is hidden during the wizard to give it a focused, immersive feel. A minimal wizard-specific header shows:

```
┌──────────────────────────────────────────────────┐
│  ← Back          New Instance          Step 2/5  │
├──────────────────────────────────────────────────┤
│                                                  │
│                 [Step Content]                    │
│                                                  │
├──────────────────────────────────────────────────┤
│                              [Cancel]  [Next →]  │
└──────────────────────────────────────────────────┘
```

- **Wizard header**: Back arrow (disabled on step 1), title, step indicator (e.g., "Step 2 of 5")
- **Footer**: Cancel button (left-ish, muted), Next/action button (right, primary)
- **Step indicator**: simple text "Step N of M", not a progress bar or stepper dots — keeps it clean

### Step 1: Docker Check (auto-skipped if running)

Same as the Docker Status Overlay (Section 2). If Docker is running, user never sees this — wizard starts at Step 2. If Docker is detected mid-step, auto-advance with a brief "Docker detected!" flash.

### Step 2: Install Type

```
┌──────────────────────────────────────────────────┐
│  ← Back          New Instance          Step 1/5  │
├──────────────────────────────────────────────────┤
│                                                  │
│              Choose Install Type                 │
│                                                  │
│  ┌─────────────────────┐ ┌─────────────────────┐│
│  │                     │ │                     ││
│  │   ⚡ Standard       │ │   ⚙ Custom          ││
│  │                     │ │                     ││
│  │  Recommended.       │ │  Configure ports,   ││
│  │  One click, done.   │ │  networking, and    ││
│  │                     │ │  more.              ││
│  │  Uses sensible      │ │                     ││
│  │  defaults.          │ │  For advanced       ││
│  │                     │ │  users.             ││
│  └─────────────────────┘ └─────────────────────┘│
│                                                  │
├──────────────────────────────────────────────────┤
│                              [Cancel]  [Next →]  │
└──────────────────────────────────────────────────┘
```

- Two large cards, side by side (50/50 width)
- Click to select (highlighted border on selection)
- Standard is pre-selected by default
- Selecting Standard and clicking Next skips to Build step
- Selecting Custom and clicking Next goes to Configuration step

### Step 3: Custom Configuration

Scrollable form. Sections with headers.

```
┌──────────────────────────────────────────────────┐
│  ← Back          New Instance          Step 2/5  │
├──────────────────────────────────────────────────┤
│                                                  │
│  Instance Name                                   │
│  ┌──────────────────────────────────────────┐    │
│  │ Cosmic Otter                             │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  OpenClaw Version                                │
│  ┌──────────────────────────────────────────┐    │
│  │ v0.42.1 (latest)                      ▾ │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  ── Networking ──────────────────────────────    │
│                                                  │
│  Gateway Port          Bridge Port               │
│  ┌──────────┐          ┌──────────┐              │
│  │ 18789    │          │ 18790    │              │
│  └──────────┘          └──────────┘              │
│                                                  │
│  Network Access                                  │
│  ◉ Local only    ○ LAN access                    │
│  Accessible only from this machine.              │
│                                                  │
│  ── Environment ─────────────────────────────    │
│                                                  │
│  Timezone                                        │
│  ┌──────────────────────────────────────────┐    │
│  │ America/Toronto                       ▾ │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  Install Browser             [ toggle OFF ]      │
│  Adds a browser for web browsing tasks.          │
│                                                  │
│  ▸ Advanced Options                              │
│                                                  │
├──────────────────────────────────────────────────┤
│                              [Cancel]  [Next →]  │
└──────────────────────────────────────────────────┘
```

**Form patterns:**

- Labels above inputs (shadcn standard)
- Short helper text below inputs where needed, muted color
- Inline validation: red border + error text below field on invalid input
- Ports side by side on one row (they're related)
- Network Access as radio group with description text that updates based on selection
- Toggle switches for boolean options, with label left and switch right
- **"Advanced Options"**: collapsible section (shadcn `Collapsible`). Chevron rotates on expand. Collapsed by default.

**Advanced section (expanded):**

```
│  ▾ Advanced Options                              │
│  ┌───────────────────────────────────────────┐   │
│  │                                           │   │
│  │  Additional System Packages               │   │
│  │  ┌─────────────────────────────────────┐  │   │
│  │  │                                     │  │   │
│  │  └─────────────────────────────────────┘  │   │
│  │  Space-separated apt package names.       │   │
│  │                                           │   │
│  │  Extensions                               │   │
│  │  ┌─────────────────────────────────────┐  │   │
│  │  │                                     │  │   │
│  │  └─────────────────────────────────────┘  │   │
│  │                                           │   │
│  │  Home Volume                              │   │
│  │  ┌─────────────────────────────────────┐  │   │
│  │  │                                     │  │   │
│  │  └─────────────────────────────────────┘  │   │
│  │  Named Docker volume or host path.        │   │
│  │                                           │   │
│  │  Extra Volume Mounts                      │   │
│  │  ┌─────────────────────────────────────┐  │   │
│  │  │                                     │  │   │
│  │  │                                     │  │   │
│  │  └─────────────────────────────────────┘  │   │
│  │  source:target[:options], one per line.   │   │
│  │                                           │   │
│  │  ☐ Allow Insecure WebSocket               │   │
│  │  ⚠ Allows unencrypted WebSocket on        │   │
│  │    private networks.                      │   │
│  │                                           │   │
│  └───────────────────────────────────────────┘   │
```

- Slightly indented or within a subtle bordered container to visually group
- Warning icon and text for risky options

### Step 4: Building

```
┌──────────────────────────────────────────────────┐
│              New Instance                        │
├──────────────────────────────────────────────────┤
│                                                  │
│                  >///<                            │
│               Building...                        │
│                                                  │
│  ✓ Generating Dockerfile                         │
│  ✓ Building Docker image                         │
│  ◌ Starting container...                         │
│  ○ Running initial setup                         │
│  ○ Configuring gateway                           │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │ Step 3/10: Installing dependencies...     │   │
│  │ npm install                               │   │
│  │ added 847 packages in 12s                 │   │
│  │ ...                                       │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
├──────────────────────────────────────────────────┤
│                                        [Cancel]  │
└──────────────────────────────────────────────────┘
```

- No Back button during build (destructive to go back)
- Cancel button available (confirms with "Are you sure? This will stop the build.")
- **Stage checklist**: vertical list with status icons
  - `✓` completed (green)
  - `◌` in progress (animated spinner)
  - `○` pending (muted)
- **Log output panel**: scrollable, monospace (fits naturally since whole app is monospace), dark inset background (`muted` bg from shadcn). Auto-scrolls to bottom. Shows real-time Docker build output.
- ASCII art logo above the stage list adds personality to what could be a boring wait screen

**Error state:**

```
│                  >///<                            │
│             Build Failed                         │
│                                                  │
│  ✓ Generating Dockerfile                         │
│  ✓ Building Docker image                         │
│  ✗ Starting container                            │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │ ERROR: port 18789 is already in use       │   │
│  │ ...                                       │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│              [← Back to Settings]  [Retry]       │
```

- Failed step marked with `✗` (red)
- Error output visible in log panel
- Two recovery options: go back to edit settings, or retry with same settings

### Step 5: Provider Setup (V1 Early — CLI Instructions)

```
┌──────────────────────────────────────────────────┐
│              New Instance               Step 4/5 │
├──────────────────────────────────────────────────┤
│                                                  │
│           Set Up AI Provider                     │
│                                                  │
│  OpenClaw needs an AI provider to work.          │
│  Run this command in a terminal to configure:    │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │ docker compose -f ~/.outclaw/docker/     │ 📋│
│  │ ec_a1b2c3/docker-compose.yml run --rm     │   │
│  │ outclaw-ec_a1b2c3-cli configure          │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  This will walk you through selecting a          │
│  provider and entering your API key.             │
│                                                  │
├──────────────────────────────────────────────────┤
│                         [Skip for now]  [Done →] │
└──────────────────────────────────────────────────┘
```

- Command displayed in a code block with copy button (📋 icon)
- Brief explanation above and below the command
- "Skip for now" (muted/secondary) and "Done" (primary) buttons
- No validation that they actually did it — honor system in V1

### Step 6: Chat Channel Setup (V1 Early — CLI Instructions)

```
┌──────────────────────────────────────────────────┐
│              New Instance               Step 5/5 │
├──────────────────────────────────────────────────┤
│                                                  │
│          Set Up Chat Channels                    │
│                                                  │
│  Connect OpenClaw to your chat apps.             │
│                                                  │
│  ── Telegram ────────────────────────────────    │
│  Create a bot via @BotFather on Telegram,        │
│  then run:                                       │
│  ┌───────────────────────────────────────────┐   │
│  │ docker compose -f ... run --rm            │ 📋│
│  │ outclaw-...-cli channels add             │   │
│  │ --channel telegram --token <YOUR_TOKEN>   │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  ── WhatsApp ────────────────────────────────    │
│  Run this command and scan the QR code:          │
│  ┌───────────────────────────────────────────┐   │
│  │ docker compose -f ... run --rm            │ 📋│
│  │ outclaw-...-cli channels login           │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
├──────────────────────────────────────────────────┤
│                         [Skip for now]  [Done →] │
└──────────────────────────────────────────────────┘
```

- Each channel in its own section with separator
- Brief setup instructions + copy-able command
- Same Skip/Done pattern as provider step

### Step 7: Complete

```
┌──────────────────────────────────────────────────┐
│              New Instance                        │
├──────────────────────────────────────────────────┤
│                                                  │
│                  >///<                            │
│              You're all set!                     │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │  Instance    Cosmic Otter                  │   │
│  │  Version     v0.42.1                      │   │
│  │  Status      ● Running                    │   │
│  │  Gateway     http://localhost:18789       │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│     [ Open Gateway ]   [ Go to Dashboard ]       │
│                                                  │
└──────────────────────────────────────────────────┘
```

- Celebratory but restrained — ASCII art + "You're all set!"
- Summary card with key instance info
- Two CTAs: Open Gateway (opens browser), Go to Dashboard (returns to instance list)
- No wizard header/footer on this step — clean finish

## 5. Instance Detail View

Navigated to from the instance list. Replaces the content area (header bar remains).

```
┌──────────────────────────────────────────────────┐
│  Header Bar                                      │
├──────────────────────────────────────────────────┤
│                                                  │
│  ← Instances                                     │
│                                                  │
│  Cosmic Otter                         ● Running  │
│  v0.42.1                                         │
│                                                  │
│  [ Open Gateway ]  [ Stop ]  [ Restart ]         │
│                                                  │
│  ── Details ─────────────────────────────────    │
│                                                  │
│  Gateway URL     http://localhost:18789      📋  │
│  Gateway Token   ••••••••••••••••••••   👁 📋   │
│  Bridge Port     18790                           │
│  Network Access  Local only (localhost)          │
│  Timezone        America/Toronto                 │
│  Config Path     ~/.outclaw/instances/ec_a1/... 📋│
│  Workspace Path  ~/.outclaw/instances/ec_a1/... 📋│
│  Container ID    a1b2c3d4e5f6                 📋 │
│                                                  │
│  ── Actions ─────────────────────────────────    │
│                                                  │
│  [ Edit Settings ]  [ Rebuild ]                  │
│  [ Provider Setup ] [ Channel Setup ]            │
│                                                  │
│  ── Danger Zone ─────────────────────────────    │
│                                                  │
│  [ Delete Instance ]                             │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Navigation:**

- "← Instances" breadcrumb/back link at top left — returns to instance list
- No sidebar; this is a simple drill-down

**Instance header:**

- Name: large, prominent. Clicking it makes it editable (inline edit with save/cancel on blur/enter/escape).
- Status badge: right-aligned, same color coding as instance list
- Version: below the name, muted

**Action buttons:**

- Primary row: "Open Gateway" (opens browser), "Stop"/"Start" (contextual), "Restart"
- Use shadcn button variants: "Open Gateway" = `default`, "Stop" = `outline`, "Restart" = `outline`
- When stopped: "Start" replaces "Stop", "Restart" is hidden

**Details section:**

- Key-value layout: label left (muted), value right, copy button on applicable fields
- Gateway Token: masked with dots by default. Eye icon toggles visibility. Copy button copies actual value regardless of mask state.
- Monospace values (URLs, paths, IDs) stand out naturally since the whole app is monospace

**Actions section:**

- "Edit Settings": opens the configuration form (same layout as wizard Step 3, pre-filled). This could be a modal/dialog or a new view. Recommend: full content area replacement (same as wizard step) with a "← Back to Instance" breadcrumb.
- "Rebuild": confirms ("This will rebuild the Docker image and restart the container. Continue?") then shows the build screen (same as wizard Step 4)
- "Provider Setup" / "Channel Setup": same as wizard Steps 5/6

**Danger Zone:**

- Visually separated section at the bottom
- Red-tinted or bordered (shadcn `destructive` variant)
- "Delete Instance" button: `destructive` variant
- Confirmation dialog: "Delete 'Cosmic Otter'? This will stop the container, remove the Docker image, and delete all instance data including workspace files. This cannot be undone." with "Cancel" and "Delete" buttons.

## 6. Settings (Gear Icon)

Minimal app-level settings. Opens as a modal dialog from the gear icon in the header.

Contents (V1):

- **Data Directory**: shows `~/.outclaw/` path (read-only, informational)
- **About**: OutClaw version, link to GitHub repo

This is intentionally minimal. Per-instance settings live on the instance detail screen, not here.

## 7. Component Inventory

Reusable shadcn-svelte components used across the app:

| Component           | shadcn Component                          | Usage                                  |
| ------------------- | ----------------------------------------- | -------------------------------------- |
| Status Dot          | Custom (colored `<span>`)                 | Instance list cards, detail header     |
| Instance Card       | `Card`                                    | Instance list items                    |
| Code Block          | Custom (`<pre>` with copy button)         | CLI commands in wizard                 |
| Copy Button         | `Button` variant `ghost` + clipboard icon | Detail fields, code blocks             |
| Form Field          | `Label` + `Input`/`Select`/`Switch`       | Wizard configuration, edit settings    |
| Confirm Dialog      | `AlertDialog`                             | Delete instance, cancel build, rebuild |
| Collapsible Section | `Collapsible`                             | Advanced options in config form        |
| Searchable Dropdown | `Combobox` (command + popover)            | Timezone selector, version selector    |
| Step Indicator      | Custom text                               | Wizard header "Step N of M"            |
| Section Header      | Custom (label + horizontal rule)          | Instance detail, wizard config form    |
| Docker Status Pill  | `Badge` variant                           | Header bar                             |

## 8. Navigation Map

```
App Launch
  │
  ├─ Docker not available → Docker Overlay (blocks until resolved)
  │
  ├─ No instances → Empty State
  │   └─ "Create First Instance" → Setup Wizard
  │
  └─ Has instances → Instance List
      ├─ "+ New" → Setup Wizard
      └─ Click instance → Instance Detail
          ├─ "← Instances" → Instance List
          ├─ "Edit Settings" → Config Form → (save) → Build Screen → Instance Detail
          ├─ "Rebuild" → Build Screen → Instance Detail
          ├─ "Provider Setup" → Provider CLI Screen → Instance Detail
          ├─ "Channel Setup" → Channel CLI Screen → Instance Detail
          └─ "Delete" → Confirm → Instance List

Setup Wizard (linear):
  Docker Check → Install Type → [Config] → Build → Provider → Channels → Complete
                                                                          ├─ "Open Gateway" → browser
                                                                          └─ "Dashboard" → Instance List
```

All navigation is stack-based (push/pop). No deep nesting. Maximum depth is 2 (List → Detail → Edit Settings). Wizard is a separate flow that exits back to the list.

## 9. Visual Design Notes

### Typography

- **Primary font**: JetBrains Mono (fallback: Fira Code, Cascadia Code, monospace)
- All text in monospace — headers, body, labels, buttons, everything
- Size scale follows shadcn defaults but in monospace:
  - Page titles: 24px, semibold
  - Section headers: 16px, semibold
  - Body/labels: 14px, regular
  - Helper text: 12px, muted color
  - Code blocks: 13px (slightly smaller for density)

### Color Palette

Follow shadcn dark theme defaults:

- Background: `hsl(240 10% 3.9%)` (near-black)
- Card/elevated: `hsl(240 10% 6%)` (slightly lighter)
- Primary: shadcn default blue/white
- Muted text: `hsl(240 5% 64.9%)`
- Status colors:
  - Running: green (`hsl(142 76% 36%)`)
  - Stopped: gray (`hsl(240 5% 50%)`)
  - Error: red (`hsl(0 84% 60%)`)
  - Building: blue (`hsl(217 91% 60%)`)
  - Docker not running: yellow/amber (`hsl(48 96% 53%)`)
- Danger zone: red-tinted border or background

### ASCII Art / Branding

- Logo mark: small claw-inspired ASCII art (explore options like `>///<`, `{🦀}`, or a custom design during implementation)
- Used in: header bar (compact), empty state (larger), build/complete screens (medium)
- Keep it subtle — 2-3 lines max, not a full-screen splash
- The logo is a creative element to be finalized during implementation, not a blocker

### Spacing & Layout

- Content padding: 24px on sides, 16px top
- Card padding: 16px
- Gap between cards: 12px
- Section spacing: 24px between sections
- Max content width: 800px, centered (prevents overly wide layouts on large screens)
