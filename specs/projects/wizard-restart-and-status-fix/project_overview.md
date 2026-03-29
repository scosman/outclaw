---
status: complete
---

# Wizard Gateway Restart & Instance Status Fix

Two related improvements to the setup wizard's completion flow:

1. **Gateway restart on wizard completion**: When finishing step 5 (channel setup) and entering step 6 (complete), call a backend API that restarts the gateway and waits for it to come back online. The existing polling/wait code in the WhatsApp connection backend should be extracted into a shared helper and reused for both the WhatsApp flow and this new restart flow. The UI should show a "restarting" spinner state during this process.

2. **Fix stale instance status after creation**: After creating an instance via the wizard, it always shows "stopped" even if the instance is running. This also happens when returning to the dashboard — it shows "stopped" but the container is actually running. The state doesn't update until a full webapp reload. The root cause is that the poller's `instance-status-changed` events are silently dropped for instance IDs not yet in the frontend store's `SvelteMap`, and the instance is only added to the store very late in the wizard flow. After the gateway restart completes, the UI state should be refreshed so the status is instantly correct.
