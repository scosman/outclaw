<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import DockerStatusPill from '$lib/components/DockerStatusPill.svelte';
	import DockerOverlay from '$lib/components/DockerOverlay.svelte';
	import { dockerStore } from '$lib/stores/docker.svelte';

	let { children } = $props();
	let wobbling = $state(false);

	function wobble() {
		wobbling = true;
		setTimeout(() => (wobbling = false), 600);
	}

	// Initialize Docker store once at the layout level
	onMount(() => {
		dockerStore.initialize();
		return dockerStore.cleanup;
	});
</script>

<div class="flex h-screen flex-col">
	<header class="flex h-12 items-center justify-between border-b border-zinc-800 bg-zinc-900 px-4">
		<h1 class="text-lg font-semibold tracking-tight text-zinc-100 flex flex-row gap-2 items-center">
			<button onclick={wobble} class="focus:outline-none" aria-label="Wobble the crab">
				<img
					src="/logo.png"
					alt="OutClaw"
					class="h-8 w-8 cursor-pointer"
					class:crab-wobble={wobbling}
				/>
			</button>
			OutClaw
		</h1>
		<DockerStatusPill />
	</header>
	<main class="relative flex-1 overflow-auto">
		<DockerOverlay>
			{@render children()}
		</DockerOverlay>
	</main>
</div>

<style>
	@keyframes crab-wobble {
		0% {
			transform: rotate(0deg);
		}
		15% {
			transform: rotate(14deg);
		}
		30% {
			transform: rotate(-12deg);
		}
		45% {
			transform: rotate(10deg);
		}
		60% {
			transform: rotate(-8deg);
		}
		75% {
			transform: rotate(4deg);
		}
		90% {
			transform: rotate(-2deg);
		}
		100% {
			transform: rotate(0deg);
		}
	}

	:global(.crab-wobble) {
		animation: crab-wobble 0.6s ease-in-out;
	}
</style>
