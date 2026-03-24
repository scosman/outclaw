<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import type { InstanceWithStatus } from '$lib/types/instance';
	import BuildProgress from '$lib/components/BuildProgress.svelte';

	const instanceId = $derived($page.params.id ?? '');
	const mode = $derived($page.url.searchParams.get('mode') ?? 'rebuild');

	// Local state
	let instance = $state<InstanceWithStatus | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let buildStarted = $state(false);
	let buildComplete = $state(false);
	let buildError = $state<string | null>(null);

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

			// Start build after loading
			startBuild();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load instance';
		} finally {
			loading = false;
		}
	}

	async function startBuild() {
		if (buildStarted) return;
		buildStarted = true;

		try {
			await invoke('build_instance', { id: instanceId });
		} catch (e) {
			buildError = e instanceof Error ? e.message : 'Build failed';
		}
	}

	function handleBuildComplete() {
		buildComplete = true;
		// Refresh instance data
		instancesStore.refresh();
	}

	function handleBuildError(err: string) {
		buildError = err;
	}

	function handleBackToSettings() {
		goto(`/instances/${instanceId}/edit`);
	}

	function handleRetry() {
		buildStarted = false;
		buildComplete = false;
		buildError = null;
		startBuild();
	}

	function handleGoToInstance() {
		goto(`/instances/${instanceId}`);
	}
</script>

<svelte:head>
	<title>{mode === 'rebuild' ? 'Rebuilding' : 'Building'} - {instance?.name ?? 'Instance'} - OutClaw</title>
</svelte:head>

<div class="h-full overflow-y-auto p-6">
	<!-- Back navigation (only show before build starts or after error) -->
	{#if !buildStarted || buildError}
		<a
			href="/instances/{instanceId}"
			class="mb-6 inline-flex items-center gap-1 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
		>
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Instance
		</a>
	{/if}

	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="text-zinc-500">Loading instance...</div>
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
			<!-- Header -->
			<div class="mb-6">
				<h1 class="text-2xl font-semibold text-zinc-100">
					{mode === 'rebuild' ? 'Rebuilding' : 'Building'} {instance.name}
				</h1>
				<p class="mt-1 text-sm text-zinc-400">
					{mode === 'rebuild'
						? 'Rebuilding the Docker image with your updated settings.'
						: 'Building the Docker image for your new instance.'}
				</p>
			</div>

			<!-- Build Progress -->
			{#if buildStarted}
				<BuildProgress
					{instanceId}
					onComplete={handleBuildComplete}
					onError={handleBuildError}
					onBackToSettings={handleBackToSettings}
					onRetry={handleRetry}
				/>
			{/if}

			<!-- Completion actions -->
			{#if buildComplete && !buildError}
				<div class="mt-6 flex gap-3">
					<button
						class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
						onclick={handleGoToInstance}
					>
						Go to Instance
					</button>
				</div>
			{/if}
		</div>
	{/if}
</div>
