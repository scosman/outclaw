<script lang="ts">
	import { page } from '$app/stores';

	const instanceId = $derived($page.params.id);
</script>

<svelte:head>
	<title>Edit Instance {instanceId} - EasyClaw</title>
</svelte:head>

<div class="h-full p-6">
	<a
		href={`/instances/${instanceId}`}
		class="mb-4 inline-block text-sm text-zinc-400 hover:text-zinc-200"
	>
		← Instance
	</a>

	<div class="flex h-[calc(100%-2rem)] items-center justify-center">
		<div class="text-center">
			<h2 class="mb-2 text-xl font-semibold text-zinc-200">Edit Instance</h2>
			<p class="mb-2 text-zinc-500">Instance ID: {instanceId}</p>
			<p class="text-sm text-zinc-600">Full implementation coming in Phase 9</p>
		</div>
	</div>
</div>
