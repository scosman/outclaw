---
status: complete
---

# Architecture: Wizard Gateway Restart & Instance Status Fix

Small, focused change. Everything fits in this document — no component designs needed.

## 1. Backend: Shared Gateway-Ready Helper

### New function in `src-tauri/src/commands/instances.rs`

Extract the poll logic from `connect_whatsapp` into a standalone async function at module level:

```rust
/// Wait for a gateway container to become ready after a restart.
/// Sleeps for an initial delay, then polls `docker exec echo ready`
/// up to `max_attempts` times with 1-second intervals.
async fn wait_for_gateway_ready(
    docker_cli: &DockerCli,
    container_name: &str,
    initial_delay_secs: u64,
    max_attempts: u32,
) -> Result<(), String> {
    tokio::time::sleep(std::time::Duration::from_secs(initial_delay_secs)).await;

    for i in 0..max_attempts {
        match docker_cli
            .docker_exec(container_name, &["echo", "ready"])
            .await
        {
            Ok(_) => return Ok(()),
            Err(_) => {
                if i < max_attempts - 1 {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    Err(format!(
        "Gateway did not become ready after {} attempts",
        max_attempts
    ))
}
```

**Callers:**

- `connect_whatsapp`: replace the inline poll loop with `wait_for_gateway_ready(&state.docker_cli, &container_name, 5, 30).await?`
- `restart_gateway` (new command, see below)

### New Tauri Command: `restart_gateway`

```rust
#[tauri::command]
pub async fn restart_gateway(
    instance_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. Load instance config
    // 2. compose_stop (non-fatal on error, log warning)
    // 3. sleep 1s
    // 4. compose_up (fatal on error)
    // 5. wait_for_gateway_ready(docker_cli, container_name, 5, 30)
    // 6. emit_instance_status to push fresh status to frontend
}
```

The flow mirrors build stage 9 (stop → sleep → up) plus the WhatsApp-style readiness poll. After the container is ready, it calls the existing `emit_instance_status` helper to push the latest Docker status to the frontend.

**Registration:** Add `restart_gateway` to the `invoke_handler` in `lib.rs`.

## 2. Backend: Refactor `connect_whatsapp`

Replace lines 1096–1124 of `connect_whatsapp` (the inline sleep + poll loop) with a call to `wait_for_gateway_ready`. The `emit_progress` call before the wait stays where it is. On error from the helper, map it to an `emit_progress` error + return.

No behavioral change — this is a pure refactor.

## 3. Frontend: Register Instance in Store After Creation

### In `src/routes/wizard/+page.svelte` — `createAndBuild()`

After `wizardStore.createInstance()` succeeds, add the instance to the global store immediately:

```typescript
async function createAndBuild() {
	isCreating = true;
	error = null;

	try {
		await wizardStore.createInstance();
		// Add to instances store immediately so poller events are captured
		if (wizardStore.createdInstanceId && wizardStore.createdInstanceConfig) {
			instancesStore.setInstance({
				...wizardStore.createdInstanceConfig,
				id: wizardStore.createdInstanceId,
				status: { state: 'building', error_message: undefined }
			});
		}
		wizardStore.goToStep('build');
	} catch (e) {
		error = `Failed to create instance: ${e}`;
	} finally {
		isCreating = false;
	}
}
```

`createdInstanceConfig` is an `InstanceConfig`. We spread it and add a `status` field with `state: 'building'` to satisfy the `InstanceWithStatus` shape. From this point forward, poller events for this instance ID are captured by the store's `instance-status-changed` listener.

## 4. Frontend: Gateway Restart + Store Refresh on Step Transition

### In `src/routes/wizard/+page.svelte` — `handleNext()` channel case

Replace the current channel→complete transition:

```typescript
} else if (wizardStore.currentStep === 'channel') {
    wizardStore.nextStep();
    await fetchCreatedInstance();
}
```

With a new flow that shows a "restarting" state, calls the restart command, refreshes the store, then shows complete:

```typescript
} else if (wizardStore.currentStep === 'channel') {
    isRestarting = true;
    restartError = null;
    wizardStore.nextStep(); // advances to 'complete' — but UI shows restarting state

    try {
        await invoke('restart_gateway', {
            instanceId: wizardStore.createdInstanceId
        });
        await instancesStore.refresh();
    } catch (e) {
        restartError = `Failed to restart gateway: ${e}`;
    } finally {
        isRestarting = false;
    }
}
```

**New state variables:**

```typescript
let isRestarting = $state(false);
let restartError = $state<string | null>(null);
```

**Retry handler:**

```typescript
async function retryRestart() {
	isRestarting = true;
	restartError = null;
	try {
		await invoke('restart_gateway', {
			instanceId: wizardStore.createdInstanceId
		});
		await instancesStore.refresh();
	} catch (e) {
		restartError = `Failed to restart gateway: ${e}`;
	} finally {
		isRestarting = false;
	}
}
```

### UI rendering in the `complete` step

The `complete` step's `{:else if wizardStore.currentStep === 'complete'}` block gets a conditional wrapper:

```svelte
{:else if wizardStore.currentStep === 'complete'}
    {#if isRestarting}
        <!-- Restarting interstitial -->
        <div class="mx-auto flex w-full max-w-2xl flex-1 items-center px-6 py-8">
            <div class="w-full text-center space-y-4">
                <h2 class="text-xl font-semibold text-zinc-100">Finishing Setup</h2>
                <p class="text-sm text-zinc-400">Restarting gateway to apply your configuration...</p>
                <CrabLoading loading={true} />
            </div>
        </div>
    {:else if restartError}
        <!-- Restart error -->
        <div class="mx-auto flex w-full max-w-2xl flex-1 items-center px-6 py-8">
            <div class="w-full text-center space-y-4">
                <h2 class="text-xl font-semibold text-zinc-100">Restart Failed</h2>
                <div class="rounded-lg border border-red-500/30 bg-red-500/10 p-4">
                    <p class="text-sm text-red-400">{restartError}</p>
                </div>
                <button ... onclick={retryRestart}>Retry</button>
            </div>
        </div>
    {:else}
        <!-- Existing complete screen (updated to read from store) -->
    {/if}
{/if}
```

### Complete screen: derive status from store

Replace the local `createdInstance` variable usage with a store-derived value:

```typescript
const createdInstanceFromStore = $derived(
	wizardStore.createdInstanceId ? instancesStore.getInstance(wizardStore.createdInstanceId) : null
);
```

The complete screen template changes from `createdInstance?.status?.state` to `createdInstanceFromStore?.status?.state`. The `createdInstance` local variable and `fetchCreatedInstance()` function can be removed entirely.

## 5. Frontend: Footer Visibility During Restart

The wizard footer should be hidden during the restarting interstitial (no Back/Done buttons make sense during an in-flight restart). The footer's visibility condition already excludes the `complete` step, and the restart happens after advancing to the `complete` step, so this works automatically. If a restart error occurs, the Retry button is rendered inline (not in the footer).

## 6. Testing

### Rust

**Unit test for `wait_for_gateway_ready`:** Not practical to unit test (depends on Docker exec). Covered by manual testing and existing integration patterns.

**Verify `connect_whatsapp` refactor:** Run WhatsApp connection flow end-to-end to confirm no behavioral change.

### Manual Testing

1. Run full wizard end-to-end → verify "Finishing Setup" spinner appears after channel step → transitions to complete screen with "Running" status
2. Return to dashboard → verify instance shows "Running" (not "Stopped")
3. Kill Docker during restart → verify error message + Retry button works
4. WhatsApp connection flow still works (refactored polling logic)
