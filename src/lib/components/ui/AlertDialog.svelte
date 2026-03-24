<script lang="ts">
	interface Props {
		/** Whether the dialog is open */
		open: boolean;
		/** Dialog title */
		title: string;
		/** Dialog description */
		description?: string;
		/** Cancel button text */
		cancelLabel?: string;
		/** Action button text */
		actionLabel?: string;
		/** Whether this is a destructive action (shows warning) */
		destructive?: boolean;
		/** Callback when cancel is clicked */
		oncancel?: () => void;
		/** Callback when action is clicked */
		onaction?: () => void;
	}

	let {
		open,
		title,
		description,
		cancelLabel = 'Cancel',
		actionLabel = 'Continue',
		destructive = false,
		oncancel,
		onaction
	}: Props = $props();

	function handleCancel() {
		oncancel?.();
	}

	function handleAction() {
		onaction?.();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleCancel();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			handleCancel();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
		onclick={handleBackdropClick}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby="alert-dialog-title"
		tabindex="-1"
	>
		<div
			class="w-full max-w-md rounded-lg border border-zinc-800 bg-zinc-900 p-5 shadow-xl"
			role="document"
		>
			<h2 id="alert-dialog-title" class="text-lg font-semibold text-zinc-100">{title}</h2>

			{#if description}
				<p class="mt-2 text-sm text-zinc-400">{description}</p>
			{/if}

			{#if destructive}
				<div class="mt-3 rounded-md border border-amber-500/30 bg-amber-500/10 p-3">
					<p class="text-sm text-amber-400">
						This action cannot be undone. All instance data will be permanently deleted.
					</p>
				</div>
			{/if}

			<div class="mt-5 flex justify-end gap-3">
				<button
					type="button"
					class="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100"
					onclick={handleCancel}
				>
					{cancelLabel}
				</button>
				<button
					type="button"
					class="rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors {destructive
						? 'bg-red-600 hover:bg-red-700'
						: 'bg-emerald-600 hover:bg-emerald-700'}"
					onclick={handleAction}
				>
					{actionLabel}
				</button>
			</div>
		</div>
	</div>
{/if}
