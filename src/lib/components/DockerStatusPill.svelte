<script lang="ts">
	import StatusDot from './StatusDot.svelte';
	import { dockerStore } from '$lib/stores/docker.svelte';
	import { formatDockerState } from '$lib/types/instance';
</script>

<div class="flex items-center gap-2 rounded-md bg-zinc-800/50 px-2.5 py-1">
	<StatusDot state={dockerStore.status.state} size="sm" />
	<span class="text-xs font-medium text-zinc-300">
		{formatDockerState(dockerStore.status.state)}
	</span>
</div>
