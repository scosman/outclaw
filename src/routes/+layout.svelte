<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<div class="flex h-screen flex-col">
	<header class="flex h-12 items-center border-b border-zinc-800 bg-zinc-900 px-4">
		<h1 class="text-lg font-semibold tracking-tight text-zinc-100">EasyClaw</h1>
	</header>
	<main class="flex-1 overflow-auto">
		{@render children()}
	</main>
</div>
