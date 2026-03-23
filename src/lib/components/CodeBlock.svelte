<script lang="ts">
	import CopyButton from './CopyButton.svelte';

	interface Props {
		/** The code to display and copy */
		code: string;
		/** Optional language for syntax hint (not used for highlighting yet) */
		language?: string;
		/** Optional additional CSS classes */
		class?: string;
	}

	let { code, language = 'bash', class: className = '' }: Props = $props();
</script>

<div class="relative rounded-lg border border-zinc-800 bg-zinc-900 {className}">
	<div class="flex items-center justify-between border-b border-zinc-800 px-3 py-1.5">
		<span class="text-xs font-medium text-zinc-500">{language}</span>
		<CopyButton text={code} />
	</div>
	<pre class="overflow-x-auto p-3 font-mono text-sm text-zinc-300"><code
			class="whitespace-pre-wrap break-all">{code}</code
		></pre>
</div>
