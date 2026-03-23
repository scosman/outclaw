<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	interface Props {
		/** Instance ID being built */
		instanceId: string;
		/** Callback when build completes successfully */
		onComplete?: () => void;
		/** Callback when build fails */
		onError?: (error: string) => void;
		/** Callback when user wants to go back to settings */
		onBackToSettings?: () => void;
		/** Callback when user wants to retry */
		onRetry?: () => void;
	}

	let { instanceId, onComplete, onError, onBackToSettings, onRetry }: Props = $props();

	// Build stage definitions
	interface Stage {
		id: string;
		label: string;
		status: 'pending' | 'in-progress' | 'complete' | 'error';
	}

	// Build progress event from backend
	interface BuildProgressEvent {
		id: string;
		stage: string;
		log: string;
		done: boolean;
		error?: string;
	}

	let stages = $state<Stage[]>([
		{ id: 'fetching-source', label: 'Fetching OpenClaw release', status: 'pending' },
		{ id: 'generating-config', label: 'Generating configuration', status: 'pending' },
		{ id: 'building-image', label: 'Building Docker image', status: 'pending' },
		{ id: 'verifying-directories', label: 'Verifying directories', status: 'pending' },
		{ id: 'starting-container', label: 'Starting container', status: 'pending' },
		{ id: 'running-onboarding', label: 'Running onboarding', status: 'pending' },
		{ id: 'fixing-permissions', label: 'Fixing permissions', status: 'pending' },
		{ id: 'configuring-gateway', label: 'Configuring gateway', status: 'pending' },
		{ id: 'restarting-gateway', label: 'Restarting gateway', status: 'pending' }
	]);

	let logs = $state<string[]>([]);
	let showLogs = $state(false);
	let isComplete = $state(false);
	let hasError = $state(false);
	let errorMessage = $state<string | undefined>();
	let isCancelling = $state(false);

	let unlisten: UnlistenFn | null = null;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let logContainer: any = $state();

	// Maximum number of log lines to keep
	const MAX_LOG_LINES = 500;

	// Auto-scroll logs to bottom
	$effect(() => {
		if (logContainer && logs.length > 0) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	});

	onMount(async () => {
		// Listen for build progress events
		unlisten = await listen<BuildProgressEvent>('build-progress', (event) => {
			const payload = event.payload;

			// Only process events for our instance
			if (payload.id !== instanceId) return;

			// Update stage status
			// Handle "complete" stage specially - it marks all stages done
			if (payload.stage === 'complete' && payload.done) {
				stages = stages.map((s) => ({ ...s, status: 'complete' as const }));
			} else {
				const stageIndex = stages.findIndex((s) => s.id === payload.stage);
				if (stageIndex >= 0) {
					if (payload.error) {
						stages[stageIndex].status = 'error';
					} else if (payload.done) {
						stages[stageIndex].status = 'complete';
					} else {
						// Mark current as in-progress, all previous as complete
						for (let i = 0; i < stageIndex; i++) {
							if (stages[i].status !== 'complete') {
								stages[i].status = 'complete';
							}
						}
						stages[stageIndex].status = 'in-progress';
					}
				}
			}

			// Add log line with max limit
			if (payload.log) {
				const newLogs = [...logs, payload.log];
				// Keep only the last MAX_LOG_LINES
				logs = newLogs.length > MAX_LOG_LINES ? newLogs.slice(-MAX_LOG_LINES) : newLogs;
			}

			// Handle completion
			if (payload.done) {
				isComplete = true;
				if (payload.error) {
					hasError = true;
					errorMessage = payload.error;
					onError?.(payload.error);
				} else {
					onComplete?.();
				}
			}
		});
	});

	onDestroy(() => {
		if (unlisten) {
			unlisten();
		}
	});

	async function handleCancel() {
		if (isCancelling) return;
		isCancelling = true;

		try {
			await invoke('cancel_build', { id: instanceId });
			logs = [...logs, 'Build cancellation requested...'];
		} catch (e) {
			console.error('Failed to cancel build:', e);
		} finally {
			isCancelling = false;
		}
	}

	function getStageIcon(status: Stage['status']): string {
		switch (status) {
			case 'complete':
				return '✓';
			case 'in-progress':
				return '◌';
			case 'error':
				return '✗';
			default:
				return '○';
		}
	}

	function getStageColor(status: Stage['status']): string {
		switch (status) {
			case 'complete':
				return 'text-emerald-500';
			case 'in-progress':
				return 'text-amber-500 animate-pulse';
			case 'error':
				return 'text-red-500';
			default:
				return 'text-zinc-600';
		}
	}
</script>

<div class="flex flex-col gap-4">
	<!-- Stage Checklist -->
	<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
		<h3 class="mb-3 text-sm font-medium text-zinc-300">Build Progress</h3>
		<div class="space-y-2">
			{#each stages as stage (stage.id)}
				<div
					class="flex items-center gap-2 text-sm transition-colors {stage.status === 'in-progress'
						? 'text-zinc-100'
						: 'text-zinc-500'}"
				>
					<span class="w-4 text-center font-mono {getStageColor(stage.status)}">
						{getStageIcon(stage.status)}
					</span>
					<span>{stage.label}</span>
				</div>
			{/each}
		</div>
	</div>

	<!-- Log Output (collapsible) -->
	<div class="rounded-lg border border-zinc-800 bg-zinc-900/50">
		<button
			type="button"
			class="flex w-full items-center justify-between px-4 py-2 text-left hover:bg-zinc-800/50"
			onclick={() => (showLogs = !showLogs)}
		>
			<span class="text-xs font-medium text-zinc-400">Build Log</span>
			<div class="flex items-center gap-2">
				<span class="text-xs text-zinc-500">{logs.length} lines</span>
				<svg
					class="h-4 w-4 text-zinc-500 transition-transform {showLogs ? 'rotate-180' : ''}"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M19 9l-7 7-7-7"
					/>
				</svg>
			</div>
		</button>
		{#if showLogs}
			<div
				bind:this={logContainer}
				class="max-h-64 min-h-32 overflow-y-auto border-t border-zinc-800 bg-black/30 p-3 font-mono text-xs"
			>
				{#if logs.length === 0}
					<span class="text-zinc-600">Waiting for build output...</span>
				{:else}
					{#each logs as log, idx (idx)}
						<pre class="whitespace-pre-wrap text-zinc-400">{log}</pre>
					{/each}
				{/if}
			</div>
		{/if}
	</div>

	<!-- Error State -->
	{#if hasError}
		<div class="rounded-lg border border-red-500/30 bg-red-500/10 p-4">
			<div class="flex items-start gap-3">
				<svg
					class="mt-0.5 h-5 w-5 flex-shrink-0 text-red-400"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
					/>
				</svg>
				<div class="flex-1">
					<p class="text-sm font-medium text-red-400">Build Failed</p>
					<p class="mt-1 text-xs text-red-400/80">
						{errorMessage || 'An unexpected error occurred'}
					</p>
				</div>
			</div>
			<div class="mt-4 flex gap-2">
				{#if onRetry}
					<button
						type="button"
						class="rounded-lg bg-emerald-600 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-emerald-700"
						onclick={onRetry}
					>
						Retry Build
					</button>
				{/if}
				{#if onBackToSettings}
					<button
						type="button"
						class="rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-1.5 text-xs font-medium text-zinc-300 transition-colors hover:bg-zinc-700"
						onclick={onBackToSettings}
					>
						Back to Settings
					</button>
				{/if}
			</div>
		</div>
	{/if}

	<!-- Action Buttons -->
	{#if !isComplete && !hasError}
		<div class="flex justify-end">
			<button
				type="button"
				class="flex items-center gap-1.5 rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-1.5 text-xs font-medium text-zinc-300 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-50"
				disabled={isCancelling}
				onclick={handleCancel}
			>
				{#if isCancelling}
					<div
						class="h-3 w-3 animate-spin rounded-full border-2 border-zinc-600 border-t-zinc-300"
					></div>
					<span>Cancelling...</span>
				{:else}
					<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
					<span>Cancel Build</span>
				{/if}
			</button>
		</div>
	{/if}

	<!-- Success State -->
	{#if isComplete && !hasError}
		<div class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-4">
			<div class="flex items-center gap-3">
				<svg class="h-5 w-5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
					/>
				</svg>
				<p class="text-sm font-medium text-emerald-400">Build Complete!</p>
			</div>
		</div>
	{/if}
</div>
