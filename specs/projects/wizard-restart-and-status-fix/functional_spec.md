---
status: complete
---

# Functional Spec: Wizard Gateway Restart & Instance Status Fix

Two tightly coupled changes to the wizard completion flow: restart the gateway when entering the final step, and fix the bug where newly created instances show stale status.

## 1. Gateway Restart on Wizard Completion

### Behavior

When the user finishes step 5 (channel setup) and advances to step 6 (complete), the app restarts the gateway container and waits for it to come back online before showing the completion screen.

**Trigger:** User clicks "Done" on the channel step (already requires at least one channel connected).

**Flow:**

1. UI transitions to a "restarting" interstitial state (not the complete screen yet)
2. Frontend calls a new backend command `restart_gateway(instance_id)`
3. Backend performs: compose stop → compose up → poll for container readiness
4. On success: frontend transitions to the complete screen with fresh instance status
5. On error: frontend shows error with retry option

### Why restart?

After the provider and channel setup steps, the gateway needs a restart to pick up the new configuration. Without this, the user reaches the complete screen with a gateway that hasn't loaded the provider/channel config yet.

### Polling Logic (shared helper)

The existing WhatsApp connection flow (`connect_whatsapp`) already has a poll-for-gateway-ready pattern:

- Initial 5-second sleep (let the container start)
- Poll up to 30 attempts: `docker exec <container> echo ready`
- 1 second between poll attempts
- Fail if not ready after all attempts

This polling logic must be extracted into a shared helper function and reused for both the WhatsApp flow and the new gateway restart command. The build pipeline's stage 9 (restart gateway) should also use it for consistency, but that's optional/nice-to-have since it already works without polling.

### Error Handling

- If compose stop fails: log warning, continue (non-fatal, same as build stage 9)
- If compose up fails: return error to frontend
- If gateway doesn't become ready within poll timeout: return error to frontend
- Frontend shows the error with a "Retry" button that re-invokes the restart command

## 2. Fix Stale Instance Status After Creation

### Root Cause

The frontend `instancesStore` uses a `SvelteMap<string, InstanceWithStatus>`. The backend poller emits `instance-status-changed` events for all known instances, but the frontend listener silently drops events for IDs not in the map:

```typescript
const instance = instances.get(id);
if (instance) {
	instances.set(id, { ...instance, status });
}
// no else — event is silently dropped
```

During the wizard, `create_instance` returns an `InstanceConfig` but never adds the instance to the store's map. The instance is only added to the store at the very end via `fetchCreatedInstance()` on the channel→complete transition. Between creation and that point, all poller status events for the new instance are lost.

Additionally, the complete screen reads from a local `createdInstance` variable (one-shot fetch), not from the reactive store, so even later poller updates don't reach the UI.

### Fix

**A. Add instance to store immediately after creation:**

After `wizardStore.createInstance()` succeeds, construct an `InstanceWithStatus` with a `building` state and call `instancesStore.setInstance(...)`. This ensures all subsequent poller events for this instance are captured by the store's event listener.

**B. Refresh store after gateway restart completes:**

After the `restart_gateway` command succeeds (at the channel→complete transition), call `instancesStore.refresh()` to get authoritative status for all instances. This guarantees the store has the correct "running" state.

**C. Complete screen reads from the reactive store:**

Instead of binding to a local `createdInstance` variable, the complete screen should derive the instance from `instancesStore.getInstance(wizardStore.createdInstanceId)`. This means status updates from the poller flow through to the UI automatically.

**D. Dashboard also shows correct status:**

Because the instance is in the store from the start, navigating to the dashboard after the wizard shows the correct status immediately — no reload required.

## 3. UI Design

### Restarting Interstitial

When transitioning from channel → complete, show a centered spinner state:

- Heading: "Finishing Setup"
- Subtext: "Restarting gateway to apply your configuration..."
- CrabLoading spinner (reuse existing component)
- If error: show error message + "Retry" button
- No Back button during this state (restart is already in progress)
- No Cancel button (gateway restart is quick and should not be interrupted)

This state is displayed in the wizard's main content area with the same step counter showing the final step number.

### Complete Screen (unchanged layout)

The complete screen layout stays the same. The only change is the data source: instance status is read reactively from the instances store instead of a local variable. The UI displays whatever the current status is (`running`, `stopped`, etc.) — no more hardcoded fallback to "Running".

## Out of Scope

- Updating build pipeline stage 9 to use the shared polling helper (nice-to-have but not required — it works fine without it)
- Adding gateway health check beyond `docker exec echo ready` (HTTP ping, etc.)
- Changing the WhatsApp flow's UX (only extracting the shared polling helper)
