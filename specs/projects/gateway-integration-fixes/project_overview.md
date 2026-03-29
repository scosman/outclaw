---
status: complete
---

# Gateway Integration Fixes

A batch of fixes for the OpenClaw gateway integration in OutClaw. The gateway UI currently has several usability and correctness issues that make the Docker-hosted gateway hard to use from the desktop app.

## Issues

1. **Instance list shows gateway URL unnecessarily** — The main instances list (InstanceCard) displays the raw gateway URL. This isn't important information at this level and adds clutter. Remove it.

2. **"Open" button opens gateway instead of details** — On the main instances list, the "Open" button launches the gateway URL in a browser. It should navigate to the instance detail page instead. Rename the button to "Details".

3. **Gateway URL missing auth token** — The "Open Gateway" button (on the detail page) opens `http://localhost:{port}` but OpenClaw requires a `?token=<token>` query parameter for authentication. The token is stored in the instance config as `gateway_token`.

4. **gateway_bind=lan produces wrong JSON config** — When the user sets `gateway_bind=lan` in the UI, the `openclaw.json` inside the container still gets `"mode": "local"` and `"bind": "loopback"`. The Stage 8 config-set calls in `create_instance` (`config set gateway.bind`) are wrapped in non-fatal error handling that swallows failures. The setting is never plumbed through correctly — the config-set commands appear to fail silently, leaving the defaults from `onboard --mode local` in place. `gateway.mode` should always be `"local"`; only `gateway.bind` toggles between `"loopback"` and `"lan"`.

5. **Missing controlUi config** — When using the gateway from the host browser (the normal OutClaw use case), OpenClaw needs `gateway.controlUi.allowedOrigins` set to allow the mapped port origin, and `gateway.controlUi.dangerouslyDisableDeviceAuth` set to `true`. Both settings should be configured for all instances (loopback and LAN). The reference `docker-setup.sh` partially handles this but OutClaw's Rust code does not.
