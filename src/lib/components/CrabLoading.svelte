<script lang="ts">
	let { loading = false }: { loading: boolean } = $props();
</script>

<div class="h-8 flex-1 min-w-0 flex items-center justify-center">
	{#if loading}
		<div class="w-full overflow-hidden max-w-[150px]">
			<img src="/logo.png" alt="OutClaw Loading" class="h-8 w-8 crab-bounce" />
		</div>
	{/if}
</div>

<style>
	@keyframes crab-walk {
		0% {
			left: 0;
		}
		50% {
			left: calc(100% - 2rem);
		}
		100% {
			left: 0;
		}
	}

	@keyframes crab-jitter {
		0%,
		100% {
			transform: rotate(0deg);
		}
		25% {
			transform: rotate(5deg);
		}
		75% {
			transform: rotate(-5deg);
		}
	}

	.crab-bounce {
		position: relative;
		animation:
			crab-walk 3s ease-in-out infinite,
			crab-jitter 0.2s ease-in-out infinite;
	}
</style>
