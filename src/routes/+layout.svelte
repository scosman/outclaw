<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import '../app.css';
	import DockerStatusPill from '$lib/components/DockerStatusPill.svelte';
	import DockerOverlay from '$lib/components/DockerOverlay.svelte';
	import WobbleCrab from '$lib/components/WobbleCrab.svelte';
	import { dockerStore } from '$lib/stores/docker.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';

	let { children } = $props();

	let unlistenFocus: UnlistenFn | null = null;
	let unlistenBlur: UnlistenFn | null = null;

	// Set poller interval based on window focus state
	async function setPollerInterval(focused: boolean) {
		try {
			await invoke('set_poller_interval', { focused });
		} catch (error) {
			console.error('Failed to set poller interval:', error);
		}
	}

	// Initialize stores once at the layout level
	onMount(() => {
		dockerStore.initialize();
		instancesStore.initialize();

		// Set up window focus/blur listeners to adjust poller interval
		const setupListeners = async () => {
			unlistenFocus = await listen('tauri://focus', () => {
				setPollerInterval(true);
			});

			unlistenBlur = await listen('tauri://blur', () => {
				setPollerInterval(false);
			});

			// Set initial interval (window starts focused)
			await setPollerInterval(true);
		};

		setupListeners();

		return () => {
			dockerStore.cleanup();
			instancesStore.cleanup();
			if (unlistenFocus) unlistenFocus();
			if (unlistenBlur) unlistenBlur();
		};
	});
</script>

<div class="flex h-screen flex-col">
	<header class="flex h-12 items-center justify-between border-b border-zinc-800 bg-zinc-900 px-4">
		<h1 class="text-lg font-semibold tracking-tight text-zinc-100 flex flex-row gap-2 items-center">
			<div class="h-8 w-8 shrink-0">
				<WobbleCrab />
			</div>
			<span class="shrink-0"> OutClaw </span>
		</h1>
		<DockerStatusPill />
	</header>
	<main class="relative flex-1 overflow-auto">
		<DockerOverlay>
			{@render children()}
		</DockerOverlay>
	</main>
</div>
