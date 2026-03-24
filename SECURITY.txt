# Security

## OpenClaw Gateway Runs in Docker

Our main security principle is to run OpenClaw in Docker. This provides isolation from the host system by default.

## No Protection for API Keys

OpenClaw doesn't provide any strong security around API keys. The agents that use them need them and have internet access. OutClaw can't and doesn't solve this.

Best practice: use keys you don't mind losing, like an OpenRouter key with a spend limit or a key with a fixed billing plan. Don't use keys with unlimited billing.

## No "Agent Sandboxing in Docker"

OpenClaw offers an option to [run agents in Docker sandboxes](https://docs.openclaw.ai/gateway/sandboxing). However, we think this is less secure!

Since we run the gateway in Docker, enabling this would require giving the gateway access to the host Docker socket. With that access, it could create a new container with arbitrary mounts on the host, exposing your host system's data. By leaving sandboxing disabled, agents run inside the gateway Docker container with no additional access to the host OS.

Agent sandboxing is better than nothing when you're running OpenClaw directly on your host machine, but it's actually worse when running the gateway in Docker.

If we enable sandboxing later, we'll do it with [Docker-in-Docker](https://www.docker.com/resources/docker-in-docker-containerized-ci-workflows-dockercon-2023/).
