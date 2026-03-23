<script lang="ts">
	let wobbling = $state(false);

	function wobble() {
		wobbling = true;
		setTimeout(() => (wobbling = false), 600);
	}
</script>

<button onclick={wobble} class="h-full w-full focus:outline-none" aria-label="Wobble the crab">
	<img
		src="/logo.png"
		alt="OutClaw"
		class="h-full w-full cursor-pointer"
		class:crab-wobble={wobbling}
	/>
</button>

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

	.crab-wobble {
		animation: crab-wobble 0.6s ease-in-out;
	}
</style>
