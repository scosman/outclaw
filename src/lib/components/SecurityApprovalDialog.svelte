<script lang="ts">
	import type { ApprovalRequest } from '$lib/types/security';

	interface Props {
		approvals: ApprovalRequest[];
		onApprove: () => void;
		onDeny: () => void;
	}

	let { approvals, onApprove, onDeny }: Props = $props();
</script>

{#if approvals.length > 0}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
		<div class="mx-4 w-full max-w-lg rounded-xl border border-zinc-700 bg-zinc-900 p-6 shadow-2xl">
			<div class="mb-4 flex items-center gap-3">
				<div class="flex h-10 w-10 items-center justify-center rounded-full bg-amber-500/20">
					<svg class="h-5 w-5 text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
					</svg>
				</div>
				<div>
					<h3 class="text-lg font-semibold text-zinc-100">Security Change Approval</h3>
					<p class="text-sm text-zinc-400">The following changes weaken security and require your confirmation.</p>
				</div>
			</div>

			<div class="mb-6 space-y-3">
				{#each approvals as approval}
					<div class="rounded-lg border border-amber-500/30 bg-amber-500/5 p-3">
						<div class="mb-1 flex items-center justify-between text-sm">
							<span class="font-medium text-amber-300">{approval.current_value} &rarr; {approval.new_value}</span>
						</div>
						<p class="text-xs text-zinc-400">{approval.description}</p>
					</div>
				{/each}
			</div>

			<div class="flex justify-end gap-3">
				<button
					type="button"
					class="rounded-lg border border-zinc-700 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
					onclick={onDeny}
				>
					Cancel
				</button>
				<button
					type="button"
					class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-amber-700"
					onclick={onApprove}
				>
					I Understand the Risks
				</button>
			</div>
		</div>
	</div>
{/if}
