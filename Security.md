# Security Guide

## OpenClaw Gateway Runs in Docker

Our main security principal is to run OpenClaw in Docker. This provides isolation from host system by default.

## No "Agents in Docker Sandbox"

Running each agent in docker (Sandbox mode) sounds more secure... but it isn't.

Since we're running the gateway in docker, to make this work we'd need to give the gateway access to the host docker socket. With this it could create a new container with arbitray mounts on the host, exposing your data. By leaving it disables, the agents run in the Gateway docker container, with no way to access host OS.

Sandboxing Agents is better than nothing when your running OpenClaw on your host machine, but worse if you're running Gateway in Docker (IMO).
