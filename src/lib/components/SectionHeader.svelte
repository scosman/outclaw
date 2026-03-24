<script lang="ts">
	interface Props {
		/** Section title */
		title: string;
		/** Optional description */
		description?: string;
	}

	let { title, description }: Props = $props();
</script>

<div class="mb-4">
	<h3 class="text-sm font-medium uppercase tracking-wide text-zinc-400">{title}</h3>
	{#if description}
		<p class="mt-1 text-xs text-zinc-500">{description}</p>
	{/if}
</div>
