# OutClaw

**Install OpenClaw in 2 minutes. Secure Docker container.** [![CI](https://github.com/scosman/outclaw/actions/workflows/ci.yml/badge.svg)](https://github.com/scosman/outclaw/actions/workflows/ci.yml)

OutClaw is a desktop app that sets up and manages [OpenClaw](https://github.com/openclaw) for you. Instead of wrestling with terminal commands, config files, and setup scripts, you get a guided wizard that sets up a secure-by-default installation.

[**Download the Latest Release**](https://github.com/scosman/outclaw/releases)

https://github.com/user-attachments/assets/f6b7177e-8692-41e0-b210-a43c64037de1

## Why OutClaw?

Setting up OpenClaw normally means cloning repos, editing config files, and running shell scripts. OutClaw removes all of that:

- **Easy to use.** OutClaw walks you through everything step by step — Docker setup, configuration, building, and connecting your AI provider and chat channels. No command line. No guesswork.
- **Secure by default.** The agent runs inside Docker with no access to your system. If you want to open things up, you can — but you'll see clear warnings first.
- **Run multiple instances of OpenClaw side by side.** Each OpenClaw instance is fully isolated — its own config, its own container, its own workspace. Run one or five, update them independently. Host it for friends and family.

## Getting Started

1. **Install [Docker Desktop](https://www.docker.com/products/docker-desktop/)** and make sure it's running.
2. **Download OutClaw** for your platform from [Releases](https://github.com/scosman/outclaw/releases).
3. **Launch OutClaw** and follow the setup wizard.

That's it. The wizard handles the rest — building the Docker image, configuring OpenClaw, and connecting your AI provider.

## Requirements

- macOS, Windows, or Linux
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) installed and running
