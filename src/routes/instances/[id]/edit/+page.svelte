<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import type { InstanceSettings, InstanceWithStatus } from '$lib/types/instance';
	import ConfigForm from '$lib/components/ConfigForm.svelte';
	import AlertDialog from '$lib/components/ui/AlertDialog.svelte';

	const instanceId = $derived($page.params.id ?? '');

	// Local state
	let instance = $state<InstanceWithStatus | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Confirmation dialog state
	let showRebuildDialog = $state(false);
	let pendingSettings = $state<InstanceSettings | null>(null);

	// Load instance data
	onMount(() => {
		loadInstance();
	});

	async function loadInstance() {
		loading = true;
		error = null;

		try {
			// First check the store
			const storeInstance = instancesStore.getInstance(instanceId);
			if (storeInstance) {
				instance = storeInstance;
			} else {
				// If not in store, fetch directly
				const result = await invoke<InstanceWithStatus>('get_instance', { id: instanceId });
				instance = result;
				instancesStore.setInstance(result);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load instance';
		} finally {
			loading = false;
		}
	}

	// Convert InstanceConfig to InstanceSettings for form
	function getInitialSettings(): Partial<InstanceSettings> | undefined {
		if (!instance) return undefined;

		return {
			name: instance.name,
			openclaw_version: instance.openclaw_version,
			gateway_port: instance.gateway_port,
			bridge_port: instance.bridge_port,
			gateway_bind: instance.gateway_bind,
			timezone: instance.timezone,
			install_browser: instance.install_browser,
			apt_packages: instance.apt_packages,
			extensions: instance.extensions,
			home_volume: instance.home_volume,
			extra_mounts: instance.extra_mounts,
			allow_insecure_ws: instance.allow_insecure_ws
		};
	}

	// Handle save from ConfigForm
	function handleSave(settings: InstanceSettings) {
		pendingSettings = settings;
		showRebuildDialog = true;
	}

	// Handle cancel from ConfigForm
	function handleCancel() {
		goto(`/instances/${instanceId}`);
	}

	// Confirm rebuild
	async function confirmRebuild() {
		if (!pendingSettings) return;

		showRebuildDialog = false;
		saving = true;

		try {
			// Update instance settings
			await invoke('update_instance', { id: instanceId, settings: pendingSettings });

			// Navigate to build screen (reuse wizard build step)
			goto(`/instances/${instanceId}/build?mode=rebuild`);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update instance';
			saving = false;
		}
	}

	// Cancel rebuild dialog
	function cancelRebuild() {
		showRebuildDialog = false;
		pendingSettings = null;
	}
</script>

<svelte:head>
	<title>Edit Settings - {instance?.name ?? 'Instance'} - OutClaw</title>
</svelte:head>

<div class="h-full overflow-y-auto p-6">
	<!-- Back navigation -->
	<a
		href="/instances/{instanceId}"
		class="mb-6 inline-flex items-center gap-1 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
	>
		<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Back to Instance
	</a>

	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="text-zinc-500">Loading instance settings...</div>
		</div>
	{:else if error || !instance}
		<div class="flex h-64 flex-col items-center justify-center gap-4">
			<div class="text-red-400">{error ?? 'Instance not found'}</div>
			<button
				class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
				onclick={() => goto(`/instances/${instanceId}`)}
			>
				Back to Instance
			</button>
		</div>
	{:else}
		<div class="mx-auto max-w-2xl">
			<h1 class="mb-2 text-2xl font-semibold text-zinc-100">Edit Settings</h1>
			<p class="mb-6 text-sm text-zinc-400">
				Modify the settings for <span class="text-zinc-200">{instance.name}</span>. Changes will
				require a rebuild.
			</p>

			{#if saving}
				<div class="mb-4 rounded-lg border border-amber-500/30 bg-amber-500/10 p-4">
					<p class="text-sm text-amber-400">Saving changes and preparing rebuild...</p>
				</div>
			{/if}

			<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-6">
				<ConfigForm
					mode="edit"
					initialSettings={getInitialSettings()}
					onSave={handleSave}
					onCancel={handleCancel}
				/>
			</div>
		</div>
	{/if}
</div>

<!-- Rebuild Confirmation Dialog -->
<AlertDialog
	open={showRebuildDialog}
	title="Rebuild Required"
	description="Saving these changes will trigger a rebuild of the Docker image. The instance will be temporarily unavailable during the rebuild."
	cancelLabel="Cancel"
	actionLabel="Save & Rebuild"
	oncancel={cancelRebuild}
	onaction={confirmRebuild}
/>
