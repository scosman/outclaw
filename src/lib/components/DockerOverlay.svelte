<script lang="ts">
	import { dockerStore } from '$lib/stores/docker.svelte';
	import StatusDot from './StatusDot.svelte';

	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
</script>

{#if !dockerStore.loading && !dockerStore.isAvailable}
	<div
		class="absolute inset-0 z-50 flex items-center justify-center bg-zinc-900/95 backdrop-blur-sm"
	>
		<div class="mx-4 max-w-md text-center">
			<div class="mb-6 flex justify-center">
				<div class="rounded-full bg-zinc-800 p-4">
					<StatusDot state={dockerStore.status.state} size="lg" />
				</div>
			</div>

			<h2 class="mb-2 text-xl font-semibold text-zinc-100">
				{#if dockerStore.isNotInstalled}
					Docker Not Installed
				{:else if dockerStore.isNotRunning}
					Docker Not Running
				{:else}
					Docker Compose Not Available
				{/if}
			</h2>

			<p class="mb-6 text-sm text-zinc-400">
				{#if dockerStore.isNotInstalled}
					OutClaw requires Docker to manage OpenClaw instances. Please install Docker Desktop to
					continue.
				{:else if dockerStore.isNotRunning}
					Docker Desktop is installed but not running. Please start Docker Desktop to continue.
				{:else}
					Docker Compose is required but not available. Please ensure Docker Compose is installed.
				{/if}
			</p>

			{#if dockerStore.isNotInstalled || true}
				<a
					href="https://www.docker.com/products/docker-desktop"
					target="_blank"
					rel="noopener noreferrer"
					class="inline-block rounded-lg bg-emerald-600 px-5 py-2.5 text-sm font-medium text-white transition-colors hover:bg-emerald-500"
				>
					Download Docker Desktop
				</a>
			{:else if dockerStore.isNotRunning}
				<div class="flex flex-col items-center">
					<div class="animate-pulse text-zinc-500">
						<svg class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
							/>
						</svg>
					</div>
					<p class="mt-3 text-sm text-zinc-500">Waiting for Docker to start...</p>
				</div>
			{/if}
		</div>
	</div>
{:else}
	{@render children?.()}
{/if}
