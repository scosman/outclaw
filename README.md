# OutClaw

**Get your own AI agent running in minutes — no terminal needed.**

OutClaw is a desktop app that sets up and manages [OpenClaw](https://github.com/openclaw) for you. Instead of wrestling with Docker commands, config files, and setup scripts, you get a guided wizard that does it all.

<!-- TODO: ![OutClaw screenshot](screenshot.png) -->

## Why OutClaw?

Setting up OpenClaw normally means cloning repos, editing config files, and running shell scripts. OutClaw removes all of that:

- **Just click through a wizard.** OutClaw walks you through everything step by step — Docker setup, configuration, building, and connecting your AI provider and chat channels. No command line. No guesswork.

- **Secure by default.** The agent runs sandboxed inside Docker with no access to your system. If you want to open things up, you can — but you'll see clear warnings first.

- **Run multiple agents side by side.** Each OpenClaw instance is fully isolated — its own config, its own container, its own workspace. Run one or five, update them independently.

- **Roll back anytime.** Every config change and image rebuild is versioned. Made a mistake? Restore a previous version in one click.

## Getting Started

1. **Install [Docker Desktop](https://www.docker.com/products/docker-desktop/)** and make sure it's running.
2. **Download OutClaw** for your platform from [Releases](https://github.com/scosman/outclaw/releases).
3. **Launch OutClaw** and follow the setup wizard.

That's it. The wizard handles the rest — building the Docker image, configuring OpenClaw, and connecting your AI provider.

## Requirements

- macOS, Windows, or Linux
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed and running

