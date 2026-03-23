<script lang="ts">
	import type { InstanceWithStatus } from '$lib/types/instance';
	import { formatInstanceState, getGatewayUrl } from '$lib/types/instance';
	import StatusDot from './StatusDot.svelte';

	interface Props {
		instance: InstanceWithStatus;
	}

	let { instance }: Props = $props();

	// Action buttons are placeholder/disabled - wired in Phase 7
	const isRunning = $derived(instance.status.state === 'running');
	const isStopped = $derived(instance.status.state === 'stopped');
	const hasError = $derived(instance.status.state === 'error');
</script>

<div
	class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4 transition-colors hover:border-zinc-700 hover:bg-zinc-900"
>
	<div class="mb-3 flex items-start justify-between">
		<div class="flex items-center gap-2">
			<StatusDot state={instance.status.state} />
			<h3 class="font-medium text-zinc-100">{instance.name}</h3>
		</div>
		<span class="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
			{instance.openclaw_version}
		</span>
	</div>

	<div class="mb-4 flex items-center gap-4 text-sm text-zinc-500">
		<span>{formatInstanceState(instance.status.state)}</span>
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

	<!-- Action buttons - placeholder/disabled for Phase 3, wired in Phase 7 -->
	<div class="flex gap-2">
		{#if isRunning}
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200"
				disabled
				title="Open gateway in browser (Phase 7)"
			>
				Open
			</button>
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200"
				disabled
				title="Stop instance (Phase 7)"
			>
				Stop
			</button>
		{:else if isStopped}
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200"
				disabled
				title="Start instance (Phase 7)"
			>
				Start
			</button>
		{:else if hasError}
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200"
				disabled
				title="View details (Phase 7)"
			>
				Details
			</button>
			<button
				class="rounded bg-zinc-800 px-3 py-1.5 text-sm text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200"
				disabled
				title="Restart instance (Phase 7)"
			>
				Restart
			</button>
		{/if}
	</div>
</div>
