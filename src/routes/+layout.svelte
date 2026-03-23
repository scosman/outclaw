<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import DockerStatusPill from '$lib/components/DockerStatusPill.svelte';
	import DockerOverlay from '$lib/components/DockerOverlay.svelte';
	import { dockerStore } from '$lib/stores/docker.svelte';

	let { children } = $props();

	// Initialize Docker store once at the layout level
	onMount(() => {
		dockerStore.initialize();
		return dockerStore.cleanup;
	});
</script>

<div class="flex h-screen flex-col">
	<header class="flex h-12 items-center justify-between border-b border-zinc-800 bg-zinc-900 px-4">
		<h1 class="text-lg font-semibold tracking-tight text-zinc-100">OutClaw</h1>
		<DockerStatusPill />
	</header>
	<main class="relative flex-1 overflow-auto">
		<DockerOverlay>
			{@render children()}
		</DockerOverlay>
	</main>
</div>
