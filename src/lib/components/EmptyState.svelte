<script lang="ts">
	interface Props {
		showCreateButton?: boolean;
	}

	let { showCreateButton = true }: Props = $props();
</script>

<div class="flex h-full flex-col items-center justify-center">
	<div class="text-center">
		<img src="/logo.png" alt="OutClaw" class="mx-auto mb-1 h-32" />

		<p class="mb-2 text-lg font-medium text-zinc-300">No instances yet</p>
		<p class="mb-6 text-sm text-zinc-500">Create your first OpenClaw instance to get started</p>

		{#if showCreateButton}
			<a
				href="/wizard"
				class="inline-block rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
			>
				Create Your First Instance
			</a>
		{/if}
	</div>
</div>
