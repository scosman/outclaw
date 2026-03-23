<script lang="ts">
	import { onMount } from 'svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import InstanceCard from '$lib/components/InstanceCard.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';

	onMount(() => {
		instancesStore.initialize();
		return instancesStore.cleanup;
	});
</script>

<div class="h-full p-6">
	{#if instancesStore.loading}
		<div class="flex h-full items-center justify-center">
			<div class="text-zinc-500">Loading instances...</div>
		</div>
	{:else if instancesStore.hasInstances}
		<div class="mb-4 flex items-center justify-between">
			<h2 class="text-lg font-medium text-zinc-200">Instances</h2>
			<span class="text-sm text-zinc-500">
				{instancesStore.runningCount} of {instancesStore.instanceCount} running
			</span>
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
