<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

	interface Props {
		/** Instance ID to connect */
		instanceId: string;
		/** Callback when connection succeeds */
		onSuccess?: () => void;
		/** Callback when connection fails */
		onError?: (error: string) => void;
	}

	let { instanceId, onSuccess, onError }: Props = $props();

	// Step state: 1 = create bot (enter token), 2 = enter pairing code
	let step = $state(1);
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);

	// Form fields
	let botToken = $state('');
	let pairingCode = $state('');

	async function handleConnectToken() {
		if (!botToken.trim()) {
			error = 'Please enter a bot token';
			return;
		}

		isSubmitting = true;
		error = null;

		try {
			await invoke('add_telegram_channel', {
				instanceId,
				token: botToken.trim()
			});
			// Success - move to step 2
			step = 2;
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			onError?.(error ?? 'Failed to add Telegram channel');
		} finally {
			isSubmitting = false;
		}
	}

	async function handlePair() {
		if (!pairingCode.trim()) {
			error = 'Please enter a pairing code';
			return;
		}

		isSubmitting = true;
		error = null;

		try {
			await invoke('approve_telegram_pairing', {
				instanceId,
				pairingCode: pairingCode.trim()
			});
			// Success!
			onSuccess?.();
		} catch {
			error = 'Pairing request failed. Please check the code and try again.';
			// Don't call onError here - user can retry
		} finally {
			isSubmitting = false;
		}
	}

	function handleRestart() {
		step = 1;
		botToken = '';
		pairingCode = '';
		error = null;
	}

	function handleKeydown(e: KeyboardEvent, handler: () => void) {
		if (e.key === 'Enter' && !isSubmitting) {
			handler();
		}
	}
</script>

<div class="flex flex-col gap-4">
	{#if step === 1}
		<!-- Step 1: Create Bot -->
		<div class="space-y-4">
			<div class="rounded-lg border border-zinc-700 bg-zinc-800/50 p-4">
				<h4 class="mb-2 text-sm font-medium text-zinc-200">Create a Telegram Bot</h4>
				<ol class="list-inside list-decimal space-y-2 text-sm text-zinc-400">
					<li>
						Open Telegram and chat with
						<a
							href="https://t.me/BotFather"
							target="_blank"
							rel="noopener noreferrer"
							class="text-blue-400 underline hover:text-blue-300">@BotFather</a
						>
						(confirm the handle is exactly <span class="font-mono text-zinc-300">@BotFather</span>)
					</li>
					<li>
						Run <span class="rounded bg-zinc-700 px-1.5 py-0.5 font-mono text-xs text-zinc-300"
							>/newbot</span
						> and follow the prompts to create a bot
					</li>
					<li>Once you have a token, paste it below</li>
				</ol>
			</div>

			<div class="space-y-2">
				<label for="bot-token" class="block text-sm font-medium text-zinc-300">Bot Token</label>
				<input
					id="bot-token"
					type="text"
					bind:value={botToken}
					placeholder="1234567890:ABCdefGHIjklMNOpqrsTUVwxyz"
					class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					disabled={isSubmitting}
					onkeydown={(e) => handleKeydown(e, handleConnectToken)}
				/>
			</div>

			{#if error}
				<div class="rounded-lg border border-red-500/30 bg-red-500/10 p-3">
					<p class="text-sm text-red-400">{error}</p>
				</div>
			{/if}

			<div class="flex justify-end">
				<button
					type="button"
					class="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
					disabled={isSubmitting || !botToken.trim()}
					onclick={handleConnectToken}
				>
					{#if isSubmitting}
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white"
						></div>
						<span>Connecting...</span>
					{:else}
						<span>Connect</span>
					{/if}
				</button>
			</div>
		</div>
	{:else}
		<!-- Step 2: Pairing Code -->
		<div class="space-y-4">
			<div class="rounded-lg border border-zinc-700 bg-zinc-800/50 p-4">
				<h4 class="mb-2 text-sm font-medium text-zinc-200">Pair Your Bot</h4>
				<ol class="list-inside list-decimal space-y-2 text-sm text-zinc-400">
					<li>Open Telegram and start a conversation with your bot</li>
					<li>Send any message to the bot</li>
					<li>The bot will reply with a pairing code</li>
					<li>Paste the pairing code below</li>
				</ol>
			</div>

			<div class="space-y-2">
				<label for="pairing-code" class="block text-sm font-medium text-zinc-300"
					>Pairing Code</label
				>
				<input
					id="pairing-code"
					type="text"
					bind:value={pairingCode}
					placeholder="Enter pairing code"
					class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					disabled={isSubmitting}
					onkeydown={(e) => handleKeydown(e, handlePair)}
				/>
			</div>

			{#if error}
				<div class="rounded-lg border border-red-500/30 bg-red-500/10 p-3">
					<p class="text-sm text-red-400">{error}</p>
				</div>
			{/if}

			<div class="flex items-center justify-between">
				<button
					type="button"
					class="text-sm text-zinc-500 hover:text-zinc-300"
					disabled={isSubmitting}
					onclick={handleRestart}
				>
					Restart connection
				</button>
				<button
					type="button"
					class="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
					disabled={isSubmitting || !pairingCode.trim()}
					onclick={handlePair}
				>
					{#if isSubmitting}
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white"
						></div>
						<span>Pairing...</span>
					{:else}
						<span>Pair</span>
					{/if}
				</button>
			</div>
		</div>
	{/if}
</div>
