<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import InstanceCard from '$lib/components/InstanceCard.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';

	onMount(() => {
		instancesStore.initialize();
		return instancesStore.cleanup;
	});

	function handleAddInstance() {
		goto('/wizard');
	}
</script>

<div class="h-full p-6">
	{#if instancesStore.loading}
		<div class="flex h-full items-center justify-center">
			<div class="text-zinc-500">Loading instances...</div>
		</div>
	{:else if instancesStore.hasInstances}
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-lg font-medium text-zinc-200">Instances</h2>
			<div class="flex items-center gap-4">
				<span class="text-sm text-zinc-500">
					{instancesStore.runningCount} of {instancesStore.instanceCount} running
				</span>
				<button
					type="button"
					class="flex items-center gap-1.5 rounded-lg bg-emerald-600 px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
					onclick={handleAddInstance}
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 4v16m8-8H4"
						/>
					</svg>
					Add Instance
				</button>
			</div>
		</div>
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each instancesStore.instanceList as instance (instance.id)}
				<InstanceCard {instance} />
			{/each}
		</div>
	{:else}
		<EmptyState />
	{/if}
</div>
