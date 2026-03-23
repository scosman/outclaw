<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		/** The text to copy to clipboard */
		text: string;
		/** Optional additional CSS classes */
		class?: string;
	}

	let { text, class: className = '' }: Props = $props();

	let copied = $state(false);
	let copyTimeout: ReturnType<typeof setTimeout> | null = null;

	async function handleCopy() {
		try {
			await navigator.clipboard.writeText(text);
			copied = true;

			// Clear any existing timeout
			if (copyTimeout) {
				clearTimeout(copyTimeout);
			}

			// Reset copied state after 2 seconds
			copyTimeout = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy text:', err);
		}
	}

	// Cleanup timeout on unmount
	onMount(() => {
		return () => {
			if (copyTimeout) {
				clearTimeout(copyTimeout);
			}
		};
	});
</script>

<button
	type="button"
	class="flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors {copied
		? 'bg-emerald-500/20 text-emerald-400'
		: 'bg-zinc-700 text-zinc-400 hover:bg-zinc-600 hover:text-zinc-200'} {className}"
	onclick={handleCopy}
	title={copied ? 'Copied!' : 'Copy to clipboard'}
>
	{#if copied}
		<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
		</svg>
		<span>Copied!</span>
	{:else}
		<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
			/>
		</svg>
		<span>Copy</span>
	{/if}
</button>
