<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import { dockerStore } from '$lib/stores/docker.svelte';
	import type { InstanceWithStatus } from '$lib/types/instance';
	import { formatInstanceState, getGatewayUrl } from '$lib/types/instance';
	import StatusDot from '$lib/components/StatusDot.svelte';
	import CopyButton from '$lib/components/CopyButton.svelte';
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import SectionHeader from '$lib/components/SectionHeader.svelte';
	import AlertDialog from '$lib/components/ui/AlertDialog.svelte';

	const instanceId = $derived($page.params.id ?? '');

	// Local state
	let instance = $state<InstanceWithStatus | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let actionInProgress = $state<string | null>(null);

	// Inline editing state
	let isEditingName = $state(false);
	let editNameValue = $state('');
	let nameInputRef: HTMLInputElement | undefined = $state();

	// Token reveal state
	let showToken = $state(false);

	// Dialog states
	let showRebuildDialog = $state(false);
	let showDeleteDialog = $state(false);
	let deleting = $state(false);
	let deleteError = $state<string | null>(null);

	// Computed state flags
	const dockerAvailable = $derived(dockerStore.isRunning);
	const effectiveState = $derived(
		dockerAvailable && instance ? instance.status.state : 'docker-not-running'
	);
	const isRunning = $derived(effectiveState === 'running');
	const isStopped = $derived(effectiveState === 'stopped');
	const hasError = $derived(effectiveState === 'error');
	const isDockerNotRunning = $derived(effectiveState === 'docker-not-running');

	// Load instance data
	onMount(() => {
		loadInstance();
	});

	async function loadInstance() {
		loading = true;
		error = null;

		try {
			// First check the store
			const storeInstance = instancesStore.getInstance(instanceId);
			if (storeInstance) {
				instance = storeInstance;
			} else {
				// If not in store, fetch directly
				const result = await invoke<InstanceWithStatus>('get_instance', { id: instanceId });
				instance = result;
				instancesStore.setInstance(result);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load instance';
		} finally {
			loading = false;
		}
	}

	// React to store updates
	$effect(() => {
		if (instancesStore.initialized) {
			const storeInstance = instancesStore.getInstance(instanceId);
			if (storeInstance) {
				instance = storeInstance;
			}
		}
	});

	// Inline name editing
	function startEditName() {
		if (!instance) return;
		editNameValue = instance.name;
		isEditingName = true;
		// Focus input after render
		setTimeout(() => nameInputRef?.focus(), 0);
	}

	async function saveName() {
		if (!instance || !editNameValue.trim()) return;

		const newName = editNameValue.trim();
		if (newName === instance.name) {
			isEditingName = false;
			return;
		}

		try {
			await invoke('rename_instance', { id: instanceId, name: newName });
			// Update local state
			instance = { ...instance, name: newName };
			// Update store
			instancesStore.setInstance(instance);
		} catch (e) {
			console.error('Failed to rename instance:', e);
			// Revert on error
			editNameValue = instance.name;
		} finally {
			isEditingName = false;
		}
	}

	function cancelEditName() {
		isEditingName = false;
		editNameValue = instance?.name ?? '';
	}

	function handleNameKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			saveName();
		} else if (e.key === 'Escape') {
			cancelEditName();
		}
	}

	// Instance actions
	async function handleOpenGateway() {
		if (!instance || actionInProgress) return;
		actionInProgress = 'open';
		try {
			await invoke('open_in_browser', { url: getGatewayUrl(instance) });
		} catch (e) {
			console.error('Failed to open browser:', e);
		} finally {
			actionInProgress = null;
		}
	}

	async function handleStart() {
		if (actionInProgress) return;
		actionInProgress = 'start';
		try {
			await invoke('start_instance', { id: instanceId });
		} catch (e) {
			console.error('Failed to start instance:', e);
		} finally {
			actionInProgress = null;
		}
	}

	async function handleStop() {
		if (actionInProgress) return;
		actionInProgress = 'stop';
		try {
			await invoke('stop_instance', { id: instanceId });
		} catch (e) {
			console.error('Failed to stop instance:', e);
		} finally {
			actionInProgress = null;
		}
	}

	async function handleRestart() {
		if (actionInProgress) return;
		actionInProgress = 'restart';
		try {
			await invoke('restart_instance', { id: instanceId });
		} catch (e) {
			console.error('Failed to restart instance:', e);
		} finally {
			actionInProgress = null;
		}
	}

	// Get config path
	function getConfigPath(): string {
		return `~/.outclaw/instances/${instanceId}/config`;
	}

	// Get workspace path
	function getWorkspacePath(): string {
		return `~/.outclaw/instances/${instanceId}/workspace`;
	}

	// Get masked token
	function getMaskedToken(token: string): string {
		return '\u2022'.repeat(token.length);
	}

	// Get CLI commands for provider/channel setup
	function getProviderSetupCommand(): string {
		return `cd ~/.outclaw/docker-containers/${instance?.container_id} && docker compose exec gateway openclaw onboard`;
	}

	function getChannelSetupCommand(): string {
		return `cd ~/.outclaw/docker-containers/${instance?.container_id} && docker compose exec gateway openclaw channels add --channel <channel-name>`;
	}

	// Rebuild action
	function handleRebuild() {
		goto(`/instances/${instanceId}/build?mode=rebuild`);
	}

	// Delete action
	async function handleDelete() {
		if (!instance || deleting) return;

		deleting = true;
		deleteError = null;
		try {
			await invoke('delete_instance', { id: instanceId });
			// Remove from store
			instancesStore.removeInstance(instanceId);
			// Navigate to instance list
			goto('/');
		} catch (e) {
			console.error('Failed to delete instance:', e);
			deleteError = e instanceof Error ? e.message : 'Failed to delete instance';
			deleting = false;
		}
	}

	function cancelDelete() {
		showDeleteDialog = false;
		deleteError = null;
	}
</script>

<svelte:head>
	<title>{instance?.name ?? 'Instance'} - OutClaw</title>
</svelte:head>

<div class="h-full overflow-y-auto p-6">
	<!-- Back navigation -->
	<a
		href="/"
		class="mb-6 inline-flex items-center gap-1 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
	>
		<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Instances
	</a>

	{#if loading}
		<div class="flex h-64 items-center justify-center">
			<div class="text-zinc-500">Loading instance...</div>
		</div>
	{:else if error || !instance}
		<div class="flex h-64 flex-col items-center justify-center gap-4">
			<div class="text-red-400">{error ?? 'Instance not found'}</div>
			<button
				class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700"
				onclick={() => goto('/')}
			>
				Back to Instances
			</button>
		</div>
	{:else}
		<!-- Header -->
		<div class="mb-6">
			<div class="flex items-start gap-3">
				<StatusDot state={effectiveState} size="lg" />
				<div class="flex-1">
					<!-- Name (editable) -->
					{#if isEditingName}
						<input
							bind:this={nameInputRef}
							type="text"
							bind:value={editNameValue}
							onkeydown={handleNameKeydown}
							onblur={saveName}
							class="bg-transparent text-2xl font-semibold text-zinc-100 outline-none"
						/>
					{:else}
						<button
							type="button"
							onclick={startEditName}
							class="text-2xl font-semibold text-zinc-100 transition-colors hover:text-emerald-400"
							title="Click to rename"
						>
							{instance.name}
						</button>
					{/if}

					<!-- Status and version -->
					<div class="mt-1 flex items-center gap-3">
						<span class="text-sm text-zinc-400">{formatInstanceState(effectiveState)}</span>
						<span class="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
							{instance.openclaw_version}
						</span>
					</div>
				</div>
			</div>

			<!-- Action buttons -->
			<div class="mt-4 flex gap-2">
				{#if isDockerNotRunning}
					<button
						class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-500"
						disabled
						title="Docker is not running"
					>
						Docker Not Running
					</button>
				{:else if isRunning}
					<button
						class="rounded bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700 disabled:opacity-50"
						onclick={handleOpenGateway}
						disabled={actionInProgress !== null}
					>
						{#if actionInProgress === 'open'}
							Opening...
						{:else}
							Open Gateway
						{/if}
					</button>
					<button
						class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
						onclick={handleStop}
						disabled={actionInProgress !== null}
					>
						{#if actionInProgress === 'stop'}
							Stopping...
						{:else}
							Stop
						{/if}
					</button>
					<button
						class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
						onclick={handleRestart}
						disabled={actionInProgress !== null}
					>
						{#if actionInProgress === 'restart'}
							Restarting...
						{:else}
							Restart
						{/if}
					</button>
				{:else if isStopped}
					<button
						class="rounded bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700 disabled:opacity-50"
						onclick={handleStart}
						disabled={actionInProgress !== null}
					>
						{#if actionInProgress === 'start'}
							Starting...
						{:else}
							Start
						{/if}
					</button>
					<button
						class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
						onclick={handleRestart}
						disabled={actionInProgress !== null}
					>
						{#if actionInProgress === 'restart'}
							Restarting...
						{:else}
							Restart
						{/if}
					</button>
				{:else if hasError}
					<button
						class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100 disabled:opacity-50"
						onclick={handleRestart}
						disabled={actionInProgress !== null}
					>
						{#if actionInProgress === 'restart'}
							Restarting...
						{:else}
							Restart
						{/if}
					</button>
				{/if}
			</div>

			<!-- Error message -->
			{#if hasError && instance.status.error_message}
				<div class="mt-4 rounded border border-red-800/50 bg-red-900/20 p-3">
					<p class="text-sm text-red-400">{instance.status.error_message}</p>
				</div>
			{/if}
		</div>

		<!-- Details Section -->
		<div class="mb-8">
			<SectionHeader title="Details" />

			<div class="space-y-3">
				<!-- Gateway URL -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">Gateway URL</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{getGatewayUrl(instance)}</div>
					</div>
					<CopyButton text={getGatewayUrl(instance)} />
				</div>

				<!-- Gateway Token -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div class="flex-1">
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
							Gateway Token
						</div>
						<div class="mt-1 flex items-center gap-2">
							<span class="font-mono text-sm text-zinc-300">
								{#if showToken}
									{instance.gateway_token}
								{:else}
									{getMaskedToken(instance.gateway_token)}
								{/if}
							</span>
							<button
								type="button"
								onclick={() => (showToken = !showToken)}
								class="rounded p-1 text-zinc-500 transition-colors hover:bg-zinc-700 hover:text-zinc-300"
								title={showToken ? 'Hide token' : 'Show token'}
							>
								{#if showToken}
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
										/>
									</svg>
								{:else}
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
										/>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
										/>
									</svg>
								{/if}
							</button>
						</div>
					</div>
					<CopyButton text={instance.gateway_token} />
				</div>

				<!-- Bridge Port -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">Bridge Port</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{instance.bridge_port}</div>
					</div>
				</div>

				<!-- Network Access -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
							Network Access
						</div>
						<div class="mt-1 text-sm text-zinc-300">
							{instance.gateway_bind === 'lan'
								? 'LAN (accessible from network)'
								: 'Loopback (localhost only)'}
						</div>
					</div>
				</div>

				<!-- Timezone -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">Timezone</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{instance.timezone}</div>
					</div>
				</div>

				<!-- Config Path -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">Config Path</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{getConfigPath()}</div>
					</div>
					<CopyButton text={getConfigPath()} />
				</div>

				<!-- Workspace Path -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
							Workspace Path
						</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{getWorkspacePath()}</div>
					</div>
					<CopyButton text={getWorkspacePath()} />
				</div>

				<!-- Instance ID -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">Instance ID</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{instance.id}</div>
					</div>
					<CopyButton text={instance.id} />
				</div>

				<!-- Container ID -->
				<div
					class="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900/50 px-4 py-3"
				>
					<div>
						<div class="text-xs font-medium uppercase tracking-wide text-zinc-500">
							Container ID
						</div>
						<div class="mt-1 font-mono text-sm text-zinc-300">{instance.container_id}</div>
					</div>
					<CopyButton text={instance.container_id} />
				</div>
			</div>
		</div>

		<!-- Actions Section -->
		<div class="mb-8">
			<SectionHeader title="Actions" description="Manage and configure this instance" />

			<div class="flex flex-wrap gap-2">
				<button
					class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100"
					onclick={() => goto(`/instances/${instanceId}/edit`)}
				>
					Edit Settings
				</button>
				<button
					class="rounded bg-zinc-800 px-4 py-2 text-sm text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100"
					onclick={() => (showRebuildDialog = true)}
				>
					Rebuild
				</button>
			</div>
		</div>

		<!-- CLI Setup Section -->
		<div class="mb-8">
			<SectionHeader
				title="CLI Setup"
				description="Manual setup commands for advanced configuration"
			/>

			<div class="space-y-4">
				<!-- Provider Setup -->
				<div>
					<div class="mb-2 text-sm font-medium text-zinc-300">Provider Setup</div>
					<CodeBlock code={getProviderSetupCommand()} language="bash" />
				</div>

				<!-- Channel Setup -->
				<div>
					<div class="mb-2 text-sm font-medium text-zinc-300">Channel Setup</div>
					<CodeBlock code={getChannelSetupCommand()} language="bash" />
				</div>
			</div>
		</div>

		<!-- Danger Zone -->
		<div class="rounded-lg border border-red-800/50 bg-red-900/10 p-4">
			<h3 class="mb-2 text-sm font-medium uppercase tracking-wide text-red-400">Danger Zone</h3>
			<p class="mb-4 text-sm text-zinc-400">
				Deleting this instance will permanently remove all configuration and data.
			</p>
			{#if deleteError}
				<div class="mb-4 rounded border border-red-800/50 bg-red-900/20 p-3">
					<p class="text-sm text-red-400">{deleteError}</p>
				</div>
			{/if}
			<button
				class="rounded bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 disabled:opacity-50"
				onclick={() => (showDeleteDialog = true)}
				disabled={deleting}
			>
				{deleting ? 'Deleting...' : 'Delete Instance'}
			</button>
		</div>
	{/if}
</div>

<!-- Rebuild Confirmation Dialog -->
<AlertDialog
	open={showRebuildDialog}
	title="Rebuild Instance"
	description="This will rebuild the Docker image for this instance. The instance will be temporarily unavailable during the rebuild process."
	cancelLabel="Cancel"
	actionLabel="Rebuild"
	oncancel={() => (showRebuildDialog = false)}
	onaction={handleRebuild}
/>

<!-- Delete Confirmation Dialog -->
<AlertDialog
	open={showDeleteDialog}
	title="Delete Instance"
	description="Are you sure you want to delete this instance? This will stop the container, remove the Docker image, and delete all instance data including your workspace."
	cancelLabel="Cancel"
	actionLabel="Delete"
	destructive={true}
	oncancel={cancelDelete}
	onaction={handleDelete}
/>
