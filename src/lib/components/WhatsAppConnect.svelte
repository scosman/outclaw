<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	interface Props {
		/** Instance ID to connect */
		instanceId: string;
		/** Callback when connection succeeds */
		onSuccess?: () => void;
		/** Callback when connection fails */
		onError?: (error: string) => void;
	}

	let { instanceId, onSuccess, onError }: Props = $props();

	// WhatsApp progress event from backend
	interface WhatsAppProgressEvent {
		id: string;
		log: string;
		done: boolean;
		error?: string;
	}

	let logs = $state<string[]>([]);
	let isComplete = $state(false);
	let hasError = $state(false);
	let errorMessage = $state<string | undefined>();
	let isConnecting = $state(false);
	let isInstalling = $state(false); // True during channel install/wait phase

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
		// Listen for WhatsApp progress events
		unlisten = await listen<WhatsAppProgressEvent>('whatsapp-progress', (event) => {
			const payload = event.payload;

			// Only process events for our instance
			if (payload.id !== instanceId) return;

			// Detect when installation phase ends and login starts
			// Login phase starts when we see "Starting WhatsApp login"
			if (payload.log.includes('Starting WhatsApp login')) {
				isInstalling = false;
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
				isConnecting = false;
				isInstalling = false;
				if (payload.error) {
					hasError = true;
					errorMessage = payload.error;
					onError?.(payload.error);
				} else {
					onSuccess?.();
				}
			}
		});

		// Auto-start connection on mount
		startConnection();
	});

	onDestroy(() => {
		if (unlisten) {
			unlisten();
		}
	});

	async function startConnection() {
		if (!instanceId) {
			console.error('No instance ID provided');
			return;
		}

		isConnecting = true;
		isInstalling = true; // Start in installation phase
		logs = [];
		hasError = false;
		isComplete = false;
		errorMessage = undefined;

		try {
			await invoke('connect_whatsapp', { instanceId });
		} catch (e) {
			// Error will be handled via event
			console.error('WhatsApp connection error:', e);
		}
	}
</script>

<div class="flex flex-col gap-4">
	{#if isInstalling}
		<!-- Installation Phase - Show only spinner -->
		<div class="flex flex-col items-center justify-center gap-4 py-8">
			<div
				class="h-8 w-8 animate-spin rounded-full border-4 border-zinc-700 border-t-emerald-500"
			></div>
			<div class="text-center">
				<p class="text-sm font-medium text-zinc-300">Installing WhatsApp Channel</p>
				<p class="mt-1 text-xs text-zinc-500">This may take a few seconds...</p>
			</div>
		</div>
	{:else}
		<!-- Terminal Output (after installation, during login) -->
		<div class="rounded-lg border border-zinc-800 bg-zinc-900/50">
			<div class="flex items-center justify-between border-b border-zinc-800 px-3 py-2">
				<span class="text-xs font-medium text-zinc-400">Terminal Output</span>
				<span class="text-xs text-zinc-500">{logs.length} lines</span>
			</div>
			<div
				bind:this={logContainer}
				class="h-120 min-h-48 overflow-y-auto overflow-x-auto bg-black/30 p-3 font-mono text-[10px] leading-tight"
			>
				{#if logs.length === 0 && isConnecting}
					<span class="text-zinc-600">Starting WhatsApp login...</span>
				{:else if logs.length === 0}
					<span class="text-zinc-600">Waiting for output...</span>
				{:else}
					{#each logs as log, idx (idx)}
						<pre class="whitespace-pre text-zinc-400">{log}</pre>
					{/each}
				{/if}
			</div>
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
						<p class="text-sm font-medium text-red-400">Connection Failed</p>
						<p class="mt-1 text-xs text-red-400/80">
							{errorMessage || 'An unexpected error occurred'}
						</p>
					</div>
				</div>
			</div>

			<!-- Retry Button (only on error) -->
			<div class="flex justify-end">
				<button
					type="button"
					class="flex items-center gap-1.5 rounded-lg bg-emerald-600 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-50"
					disabled={isConnecting}
					onclick={startConnection}
				>
					{#if isConnecting}
						<div
							class="h-3 w-3 animate-spin rounded-full border-2 border-white/30 border-t-white"
						></div>
						<span>Connecting...</span>
					{:else}
						<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
							/>
						</svg>
						<span>Retry Connection</span>
					{/if}
				</button>
			</div>
		{:else if isComplete}
			<!-- Success State -->
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
					<p class="text-sm font-medium text-emerald-400">WhatsApp Connected!</p>
				</div>
			</div>
		{/if}
	{/if}
</div>
