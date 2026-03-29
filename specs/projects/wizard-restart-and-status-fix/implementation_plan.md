---
status: complete
---

# Implementation Plan: Wizard Gateway Restart & Instance Status Fix

## Phases

- [x] Phase 1: Backend shared helper + restart command, frontend store fix + restart flow

---

### Phase 1: Backend shared helper + restart command, frontend store fix + restart flow

Single phase — all changes are tightly coupled and need to ship together for the feature to work.

**Delivers:**

- `wait_for_gateway_ready` shared helper extracted from `connect_whatsapp`
- `connect_whatsapp` refactored to use the shared helper (no behavioral change)
- `restart_gateway` Tauri command (stop → start → wait for ready → emit status)
- `restart_gateway` registered in `lib.rs` invoke handler
- Instance added to `instancesStore` immediately after `createInstance()` in the wizard
- Channel→complete transition calls `restart_gateway` + `instancesStore.refresh()`
- "Finishing Setup" restarting interstitial UI with CrabLoading spinner
- Restart error state with Retry button
- Complete screen reads instance status reactively from the store
- Removal of `createdInstance` local variable and `fetchCreatedInstance()` function

**Manual test:** Run full wizard → after channel setup, "Finishing Setup" spinner appears → transitions to complete screen showing "Running" status → return to dashboard → instance shows "Running".
