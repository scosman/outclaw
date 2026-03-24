<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import type { InstanceWithStatus } from '$lib/types/instance';
	import { formatInstanceState, getGatewayUrl } from '$lib/types/instance';
	import { dockerStore } from '$lib/stores/docker.svelte';
	import StatusDot from './StatusDot.svelte';

	interface Props {
		instance: InstanceWithStatus;
	}

	let { instance }: Props = $props();

	// Track loading states for actions
	let isLoading = $state(false);
	let actionInProgress = $state<string | null>(null);

	// Check if Docker is running
	const dockerAvailable = $derived(dockerStore.isRunning);

	// Determine effective state based on Docker availability
	const effectiveState = $derived(dockerAvailable ? instance.status.state : 'docker-not-running');

	// Computed state flags
	const isRunning = $derived(effectiveState === 'running');
	const isStopped = $derived(effectiveState === 'stopped');
	const hasError = $derived(effectiveState === 'error');
	const isDockerNotRunning = $derived(effectiveState === 'docker-not-running');

	// Navigate to instance detail page when card is clicked
	function handleCardClick(e: MouseEvent) {
		// Don't navigate if clicking on a button or link
		if ((e.target as HTMLElement).closest('button, a')) return;
		goto(`/instances/${instance.id}`);
	}

	// Open gateway in browser
	async function handleOpen(e: MouseEvent) {
		e.stopPropagation(); // Prevent card click
		try {
			await invoke('open_in_browser', { url: getGatewayUrl(instance) });
		} catch (error) {
			console.error('Failed to open browser:', error);
		}
	}

	// Start instance
	async function handleStart(e: MouseEvent) {
		e.stopPropagation(); // Prevent card click
		if (actionInProgress) return;
		actionInProgress = 'start';
		isLoading = true;
		try {
			await invoke('start_instance', { id: instance.id });
		} catch (error) {
			console.error('Failed to start instance:', error);
		} finally {
			isLoading = false;
			actionInProgress = null;
		}
	}

	// Stop instance
	async function handleStop(e: MouseEvent) {
		e.stopPropagation(); // Prevent card click
		if (actionInProgress) return;
		actionInProgress = 'stop';
		isLoading = true;
		try {
			await invoke('stop_instance', { id: instance.id });
		} catch (error) {
			console.error('Failed to stop instance:', error);
		} finally {
			isLoading = false;
			actionInProgress = null;
		}
	}

	// Restart instance
	async function handleRestart(e: MouseEvent) {
		e.stopPropagation(); // Prevent card click
		if (actionInProgress) return;
		actionInProgress = 'restart';
		isLoading = true;
		try {
			await invoke('restart_instance', { id: instance.id });
		} catch (error) {
			console.error('Failed to restart instance:', error);
		} finally {
			isLoading = false;
			actionInProgress = null;
		}
	}

	// Navigate to instance detail page
	function handleDetails(e: MouseEvent) {
		e.stopPropagation(); // Prevent card click
		goto(`/instances/${instance.id}`);
	}
</script>

<div
	class="cursor-pointer rounded-lg border border-zinc-800 bg-zinc-900/50 p-4 transition-colors hover:border-zinc-700 hover:bg-zinc-900"
	onclick={handleCardClick}
	onkeydown={(e) => e.key === 'Enter' && handleCardClick(e as unknown as MouseEvent)}
	role="button"
	tabindex="0"
>
	<div class="mb-3 flex items-start justify-between">
		<div class="flex items-center gap-2">
			<StatusDot state={effectiveState} />
			<h3 class="font-medium text-zinc-100">{instance.name}</h3>
		</div>
		<span class="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
			{instance.openclaw_version}
		</span>
	</div>

	<div class="mb-4 flex items-center gap-4 text-sm text-zinc-500">
		<span>{formatInstanceState(effectiveState)}</span>
		{#if isRunning}
			<span class="text-zinc-600">|</span>
			<a
				href={getGatewayUrl(instance)}
				class="text-emerald-500 hover:text-emerald-400"
				target="_blank"
				rel="noopener noreferrer"
			>
				{getGatewayUrl(instance)}
			</a>
		{/if}
	</div>

	{#if hasError && instance.status.error_message}
		<p class="mb-4 text-sm text-red-400">{instance.status.error_message}</p>
	{/if}

	<!-- Action buttons - wired in Phase 7 -->
	<div class="flex gap-2">
		{#if isDockerNotRunning}
			<!-- No actions available when Docker is not running -->
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-500"
				disabled
				title="Docker is not running"
			>
				Docker Not Running
			</button>
		{:else if isRunning}
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
				onclick={handleOpen}
				disabled={isLoading}
				title="Open gateway in browser"
			>
				{#if actionInProgress === 'open'}
					<span class="opacity-50">Opening...</span>
				{:else}
					Open
				{/if}
			</button>
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
				onclick={handleStop}
				disabled={isLoading}
				title="Stop instance"
			>
				{#if actionInProgress === 'stop'}
					<span class="opacity-50">Stopping...</span>
				{:else}
					Stop
				{/if}
			</button>
		{:else if isStopped}
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
				onclick={handleStart}
				disabled={isLoading}
				title="Start instance"
			>
				{#if actionInProgress === 'start'}
					<span class="opacity-50">Starting...</span>
				{:else}
					Start
				{/if}
			</button>
		{:else if hasError}
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
				onclick={handleDetails}
				disabled={isLoading}
				title="View instance details"
			>
				Details
			</button>
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
				onclick={handleRestart}
				disabled={isLoading}
				title="Restart instance"
			>
				{#if actionInProgress === 'restart'}
					<span class="opacity-50">Restarting...</span>
				{:else}
					Restart
				{/if}
			</button>
		{/if}
	</div>
</div>
