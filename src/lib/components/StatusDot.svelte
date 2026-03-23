<script lang="ts">
	import type { InstanceState, DockerState } from '$lib/types/instance';

	interface Props {
		state: InstanceState | DockerState;
		size?: 'sm' | 'md' | 'lg';
	}

	let { state, size = 'md' }: Props = $props();

	const sizeClasses = {
		sm: 'w-2 h-2',
		md: 'w-2.5 h-2.5',
		lg: 'w-3 h-3'
	};

	const colorClasses: Record<InstanceState | DockerState, string> = {
		running: 'bg-emerald-500',
		stopped: 'bg-zinc-500',
		building: 'bg-amber-500 animate-pulse',
		error: 'bg-red-500',
		'docker-not-running': 'bg-amber-500',
		'not-running': 'bg-amber-500',
		'not-installed': 'bg-red-500'
	};

	const glowClasses: Record<InstanceState | DockerState, string> = {
		running: 'shadow-[0_0_6px_rgba(16,185,129,0.6)]',
		stopped: '',
		building: 'shadow-[0_0_6px_rgba(245,158,11,0.6)]',
		error: 'shadow-[0_0_6px_rgba(239,68,68,0.6)]',
		'docker-not-running': '',
		'not-running': '',
		'not-installed': ''
	};
</script>

<div
	class="rounded-full {sizeClasses[size]} {colorClasses[state]} {glowClasses[state]}"
	aria-label="Status: {state}"
></div>
