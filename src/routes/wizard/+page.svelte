<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';
	import { wizardStore } from '$lib/stores/wizard.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import ConfigForm from '$lib/components/ConfigForm.svelte';
	import BuildProgress from '$lib/components/BuildProgress.svelte';
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import StatusDot from '$lib/components/StatusDot.svelte';
	import {
		PROVIDERS,
		getProviderById,
		getDefaultProvider,
		type ProviderConfig
	} from '$lib/config/providers';
	import type { InstanceWithStatus } from '$lib/types/instance';
	import { getGatewayUrl, formatInstanceState } from '$lib/types/instance';

	let isLoading = $state(true);
	let isCreating = $state(false);
	let error = $state<string | null>(null);
	let buildComplete = $state(false);
	let buildError = $state<string | null>(null);
	let createdInstance = $state<InstanceWithStatus | null>(null);

	// Provider connection state
	let selectedProviderId = $state<string>(getDefaultProvider().id);
	let providerFieldValues = $state<Record<string, string>>({});
	let isConnecting = $state(false);
	let connectionError = $state<string | null>(null);
	let connectionSuccess = $state(false);

	// Get the currently selected provider config
	const selectedProvider = $derived(getProviderById(selectedProviderId) || getDefaultProvider());

	// Check if all required fields are filled for the provider form
	const isProviderFormValid = $derived(() => {
		return selectedProvider.fields
			.filter((f) => f.required !== false)
			.every((f) => providerFieldValues[f.name]?.trim());
	});

	// Initialize provider field values when provider changes
	$effect(() => {
		if (selectedProviderId) {
			const newValues: Record<string, string> = {};
			const provider = getProviderById(selectedProviderId);
			if (provider) {
				for (const field of provider.fields) {
					newValues[field.name] = field.defaultValue || '';
				}
			}
			providerFieldValues = newValues;
		}
	});

	// Initialize wizard on mount
	onMount(() => {
		// Initialize wizard (fire and forget)
		wizardStore.reset();
		wizardStore.initialize().then(() => {
			isLoading = false;
		});

		// Cleanup on unmount
		return () => {
			wizardStore.cleanup();
		};
	});

	// Computed step info
	const stepNumber = $derived(wizardStore.stepNumber);
	const totalSteps = $derived(wizardStore.totalSteps);

	// CLI commands for channel setup
	const telegramCommand = $derived(
		wizardStore.createdInstanceConfig
			? `docker exec outclaw-${wizardStore.createdInstanceConfig.container_id}-gateway openclaw mauth link telegram`
			: ''
	);
	const whatsappCommand = $derived(
		wizardStore.createdInstanceConfig
			? `docker exec outclaw-${wizardStore.createdInstanceConfig.container_id}-gateway openclaw mauth link whatsapp`
			: ''
	);

	// Navigation handlers
	function handleBack() {
		if (wizardStore.currentStep === 'config') {
			wizardStore.goToStep('install-type');
		} else if (wizardStore.currentStep === 'build') {
			wizardStore.goToStep('config');
		} else if (wizardStore.currentStep === 'provider') {
			wizardStore.goToStep('build');
		} else if (wizardStore.currentStep === 'channel') {
			wizardStore.goToStep('provider');
		}
	}

	function handleCancel() {
		goto('/');
	}

	async function handleNext() {
		error = null;

		if (wizardStore.currentStep === 'install-type') {
			if (wizardStore.installType === 'standard') {
				// Standard install: create instance immediately and go to build
				await createAndBuild();
			} else {
				// Custom install: show config form
				wizardStore.goToStep('config');
			}
		} else if (wizardStore.currentStep === 'config') {
			// Validate and create instance
			if (wizardStore.validateForm()) {
				await createAndBuild();
			}
		} else if (wizardStore.currentStep === 'build') {
			// Build complete, go to provider setup
			wizardStore.nextStep();
		} else if (wizardStore.currentStep === 'provider') {
			// Provider setup done/skipped, go to channel setup
			wizardStore.nextStep();
		} else if (wizardStore.currentStep === 'channel') {
			// Channel setup done/skipped, go to complete
			wizardStore.nextStep();
			// Fetch the instance with status for the complete screen
			await fetchCreatedInstance();
		}
	}

	async function createAndBuild() {
		isCreating = true;
		error = null;

		try {
			await wizardStore.createInstance();
			wizardStore.goToStep('build');
			// Build will be triggered when user lands on build step
		} catch (e) {
			error = `Failed to create instance: ${e}`;
		} finally {
			isCreating = false;
		}
	}

	async function fetchCreatedInstance() {
		if (!wizardStore.createdInstanceId) return;

		try {
			const instance = await invoke<InstanceWithStatus>('get_instance', {
				id: wizardStore.createdInstanceId
			});
			createdInstance = instance;
			// Also update the instances store
			instancesStore.setInstance(instance);
		} catch (e) {
			console.error('Failed to fetch instance:', e);
		}
	}

	// Watch for build step to start build
	$effect(() => {
		if (
			wizardStore.currentStep === 'build' &&
			wizardStore.createdInstanceId &&
			!wizardStore.buildState
		) {
			wizardStore.startBuild().catch((e) => {
				console.error('Build failed:', e);
			});
		}
	});

	// Open gateway in browser
	async function openGateway() {
		if (!wizardStore.createdInstanceConfig) return;

		const url = getGatewayUrl(wizardStore.createdInstanceConfig);
		try {
			await invoke('open_in_browser', { url });
		} catch (e) {
			console.error('Failed to open browser:', e);
		}
	}

	// Go to dashboard
	function goToDashboard() {
		goto('/');
	}
</script>

<svelte:head>
	<title>Setup Wizard - OutClaw</title>
</svelte:head>

<!-- Full-screen wizard layout (overlays main app) -->
<div class="fixed inset-0 z-50 flex flex-col bg-zinc-900">
	<!-- Wizard Header -->
	<header
		class="flex h-14 flex-shrink-0 items-center justify-between border-b border-zinc-800 bg-zinc-900 px-6"
	>
		{#if wizardStore.currentStep !== 'install-type'}
			<button
				type="button"
				class="flex items-center gap-1 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
				onclick={handleBack}
			>
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 19l-7-7 7-7"
					/>
				</svg>
				Back
			</button>
		{:else}
			<div class="w-16"></div>
		{/if}

		<div class="flex items-center gap-3">
			<span class="text-sm font-medium text-zinc-300">Setup Wizard</span>
			<span class="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-500">
				Step {stepNumber} of {totalSteps}
			</span>
		</div>

		{#if wizardStore.currentStep !== 'complete'}
			<button
				type="button"
				class="text-sm text-zinc-400 transition-colors hover:text-zinc-200"
				onclick={handleCancel}
			>
				Cancel
			</button>
		{:else}
			<div class="w-16"></div>
		{/if}
	</header>

	<!-- Main Content Area -->
	<div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
		{#if isLoading}
			<!-- Loading State -->
			<div class="flex flex-1 items-center justify-center">
				<div class="text-center">
					<div
						class="mb-4 inline-block h-8 w-8 animate-spin rounded-full border-4 border-zinc-700 border-t-emerald-500"
					></div>
					<p class="text-sm text-zinc-400">Preparing setup wizard...</p>
				</div>
			</div>
		{:else if wizardStore.currentStep === 'install-type'}
			<!-- Step 1: Install Type Selection -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<!-- InstallTypeStep content -->
				<div class="space-y-6">
					<div class="text-center">
						<h2 class="mb-2 text-xl font-semibold text-zinc-100">Choose Install Type</h2>
						<p class="text-sm text-zinc-400">
							Select how you want to configure your OpenClaw instance
						</p>
					</div>

					<div class="grid gap-4">
						<!-- Standard Install -->
						<button
							type="button"
							class="group flex cursor-pointer items-start gap-4 rounded-lg border-2 p-4 text-left transition-all {wizardStore.installType ===
							'standard'
								? 'border-emerald-500 bg-emerald-500/10'
								: 'border-zinc-700 hover:border-zinc-600 hover:bg-zinc-800/50'}"
							onclick={() => wizardStore.setInstallType('standard')}
						>
							<div
								class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg {wizardStore.installType ===
								'standard'
									? 'bg-emerald-500/20 text-emerald-400'
									: 'bg-zinc-800 text-zinc-400'}"
							>
								<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M13 10V3L4 14h7v7l9-11h-7z"
									/>
								</svg>
							</div>
							<div class="flex-1">
								<div class="flex items-center gap-2">
									<span class="font-medium text-zinc-100">Standard Install</span>
									{#if wizardStore.installType === 'standard'}
										<span
											class="rounded-full bg-emerald-500/20 px-2 py-0.5 text-xs font-medium text-emerald-400"
										>
											Recommended
										</span>
									{/if}
								</div>
								<p class="mt-1 text-sm text-zinc-400">
									Quick setup with sensible defaults. Perfect for most users.
								</p>
								<ul class="mt-2 space-y-1 text-xs text-zinc-500">
									<li>Auto-generated instance name</li>
									<li>Latest OpenClaw version</li>
								</ul>
							</div>
							<div class="flex h-6 w-6 items-center justify-center">
								{#if wizardStore.installType === 'standard'}
									<svg class="h-5 w-5 text-emerald-500" fill="currentColor" viewBox="0 0 20 20">
										<path
											fill-rule="evenodd"
											d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
											clip-rule="evenodd"
										/>
									</svg>
								{/if}
							</div>
						</button>

						<!-- Custom Install -->
						<button
							type="button"
							class="group flex cursor-pointer items-start gap-4 rounded-lg border-2 p-4 text-left transition-all {wizardStore.installType ===
							'custom'
								? 'border-emerald-500 bg-emerald-500/10'
								: 'border-zinc-700 hover:border-zinc-600 hover:bg-zinc-800/50'}"
							onclick={() => wizardStore.setInstallType('custom')}
						>
							<div
								class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg {wizardStore.installType ===
								'custom'
									? 'bg-emerald-500/20 text-emerald-400'
									: 'bg-zinc-800 text-zinc-400'}"
							>
								<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
									/>
								</svg>
							</div>
							<div class="flex-1">
								<span class="font-medium text-zinc-100">Custom Install</span>
								<p class="mt-1 text-sm text-zinc-400">
									Configure ports, networking, extensions, and advanced options.
								</p>
								<ul class="mt-2 space-y-1 text-xs text-zinc-500">
									<li>Choose OpenClaw version</li>
									<li>Configure custom ports, LAN access, timezone, and more</li>
								</ul>
							</div>
							<div class="flex h-6 w-6 items-center justify-center">
								{#if wizardStore.installType === 'custom'}
									<svg class="h-5 w-5 text-emerald-500" fill="currentColor" viewBox="0 0 20 20">
										<path
											fill-rule="evenodd"
											d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
											clip-rule="evenodd"
										/>
									</svg>
								{/if}
							</div>
						</button>
					</div>
				</div>
			</div>
		{:else if wizardStore.currentStep === 'config'}
			<!-- Step 3: Custom Configuration -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<div class="mb-6 text-center">
					<h2 class="mb-2 text-xl font-semibold text-zinc-100">Configure Instance</h2>
					<p class="text-sm text-zinc-400">Customize your OpenClaw instance settings</p>
				</div>

				<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-6">
					<ConfigForm mode="create" />
				</div>

				{#if error}
					<div class="mt-4 rounded-lg border border-red-500/30 bg-red-500/10 p-4">
						<p class="text-sm text-red-400">{error}</p>
					</div>
				{/if}
			</div>
		{:else if wizardStore.currentStep === 'build'}
			<!-- Step 4: Build Progress -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<div class="mb-6 text-center">
					<h2 class="mb-2 text-xl font-semibold text-zinc-100">Building Instance</h2>
					<p class="text-sm text-zinc-400">Setting up your OpenClaw instance...</p>
				</div>

				{#if wizardStore.createdInstanceId}
					<BuildProgress
						instanceId={wizardStore.createdInstanceId}
						onComplete={() => {
							buildComplete = true;
							buildError = null;
						}}
						onError={(err) => {
							buildError = err;
							buildComplete = true;
						}}
						onBackToSettings={() => {
							buildComplete = false;
							buildError = null;
							wizardStore.goToStep('config');
						}}
						onRetry={() => {
							buildComplete = false;
							buildError = null;
							wizardStore.startBuild().catch((e) => {
								console.error('Build failed:', e);
							});
						}}
					/>
				{:else}
					<div class="flex items-center justify-center py-8">
						<div class="text-center">
							<div
								class="mb-4 inline-block h-8 w-8 animate-spin rounded-full border-4 border-zinc-700 border-t-emerald-500"
							></div>
							<p class="text-sm text-zinc-400">Preparing build...</p>
						</div>
					</div>
				{/if}
			</div>
		{:else if wizardStore.currentStep === 'provider'}
			<!-- Step 5: Provider Setup -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<div class="mb-6 text-center">
					<h2 class="mb-2 text-xl font-semibold text-zinc-100">Provider Setup</h2>
					<p class="text-sm text-zinc-400">
						Connect an AI provider to enable intelligent messaging capabilities
					</p>
				</div>

				<div class="space-y-6">
					<!-- Provider Selection Form -->
					<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-6">
						<div class="space-y-4">
							<!-- Provider Dropdown -->
							<div>
								<label for="provider-select" class="mb-1.5 block text-sm font-medium text-zinc-200">
									Select Provider
								</label>
								<select
									id="provider-select"
									class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
									bind:value={selectedProviderId}
									onchange={() => {
										connectionError = null;
										connectionSuccess = false;
									}}
								>
									{#each PROVIDERS as provider (provider.id)}
										<option value={provider.id}>{provider.label}</option>
									{/each}
								</select>
								<p class="mt-1 text-xs text-zinc-500">{selectedProvider.description}</p>
							</div>

							<!-- Dynamic Fields -->
							{#each selectedProvider.fields as field (field.name)}
								<div>
									<label for={field.name} class="mb-1.5 block text-sm font-medium text-zinc-200">
										{field.label}
										{#if field.required}
											<span class="text-red-400">*</span>
										{/if}
									</label>
									<input
										id={field.name}
										type={field.secret ? 'password' : 'text'}
										class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
										placeholder={field.placeholder || ''}
										bind:value={providerFieldValues[field.name]}
										oninput={() => {
											connectionError = null;
										}}
									/>
								</div>
							{/each}

							<!-- Connection Error -->
							{#if connectionError}
								<div class="rounded-lg border border-red-500/30 bg-red-500/10 p-3">
									<p class="text-sm text-red-400">{connectionError}</p>
								</div>
							{/if}

							<!-- Connection Success -->
							{#if connectionSuccess}
								<div class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3">
									<div class="flex items-center gap-2">
										<svg
											class="h-5 w-5 text-emerald-400"
											fill="none"
											viewBox="0 0 24 24"
											stroke="currentColor"
										>
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M5 13l4 4L19 7"
											/>
										</svg>
										<p class="text-sm text-emerald-400">Provider connected successfully!</p>
									</div>
								</div>
							{/if}

							<!-- Connect Button -->
							<button
								type="button"
								class="flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-600 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-50"
								disabled={isConnecting || !isProviderFormValid()}
								onclick={async () => {
									if (!wizardStore.createdInstanceId) {
										connectionError =
											'No instance found. Please go back and create an instance first.';
										return;
									}

									isConnecting = true;
									connectionError = null;
									connectionSuccess = false;

									try {
										await invoke('connect_provider', {
											instanceId: wizardStore.createdInstanceId,
											authChoice: selectedProviderId,
											fields: providerFieldValues
										});
										connectionSuccess = true;
									} catch (e) {
										connectionError = `Failed to connect provider: ${e}`;
									} finally {
										isConnecting = false;
									}
								}}
							>
								{#if isConnecting}
									<div
										class="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white"
									></div>
									Connecting...
								{:else}
									<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
										/>
									</svg>
									Connect Provider
								{/if}
							</button>
						</div>
					</div>

					<!-- Info box -->
					<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
						<div class="flex gap-3">
							<svg
								class="mt-0.5 h-5 w-5 flex-shrink-0 text-zinc-500"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
								/>
							</svg>
							<div class="text-sm text-zinc-400">
								<p class="font-medium text-zinc-300">What is a provider?</p>
								<p class="mt-1">
									A provider connects your OpenClaw instance to an AI service like Anthropic Claude
									or OpenAI. This enables intelligent messaging capabilities. You can skip this step
									and configure it later.
								</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		{:else if wizardStore.currentStep === 'channel'}
			<!-- Step 6: Channel Setup -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<div class="mb-6 text-center">
					<h2 class="mb-2 text-xl font-semibold text-zinc-100">Channel Setup</h2>
					<p class="text-sm text-zinc-400">
						Link messaging channels to communicate with your OpenClaw instance
					</p>
				</div>

				<div class="space-y-6">
					<!-- Telegram Section -->
					<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
						<div class="mb-4 flex items-center gap-3">
							<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-500/20">
								<svg class="h-5 w-5 text-blue-400" viewBox="0 0 24 24" fill="currentColor">
									<path
										d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z"
									/>
								</svg>
							</div>
							<div>
								<h3 class="font-medium text-zinc-100">Telegram</h3>
								<p class="text-xs text-zinc-500">Connect via Telegram Bot</p>
							</div>
						</div>
						<p class="mb-4 text-sm text-zinc-400">
							Run this command to link your Telegram bot to your OpenClaw instance.
						</p>
						{#if telegramCommand}
							<CodeBlock code={telegramCommand} language="bash" />
						{/if}
					</div>

					<!-- WhatsApp Section -->
					<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
						<div class="mb-4 flex items-center gap-3">
							<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-green-500/20">
								<svg class="h-5 w-5 text-green-400" viewBox="0 0 24 24" fill="currentColor">
									<path
										d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413z"
									/>
								</svg>
							</div>
							<div>
								<h3 class="font-medium text-zinc-100">WhatsApp</h3>
								<p class="text-xs text-zinc-500">Connect via WhatsApp Business API</p>
							</div>
						</div>
						<p class="mb-4 text-sm text-zinc-400">
							Run this command to link your WhatsApp Business account to your OpenClaw instance.
						</p>
						{#if whatsappCommand}
							<CodeBlock code={whatsappCommand} language="bash" />
						{/if}
					</div>

					<!-- Info box -->
					<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
						<div class="flex gap-3">
							<svg
								class="mt-0.5 h-5 w-5 flex-shrink-0 text-zinc-500"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
								/>
							</svg>
							<div class="text-sm text-zinc-400">
								<p class="font-medium text-zinc-300">You can skip this step</p>
								<p class="mt-1">
									Channel setup is optional. You can always link channels later from the instance
									detail screen.
								</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		{:else if wizardStore.currentStep === 'complete'}
			<!-- Step 7: Completion Screen -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<div class="space-y-8">
					<!-- ASCII Art Logo -->
					<div class="text-center">
						<pre class="inline-block text-xs leading-tight text-emerald-500">
  ____          _
 / __ \___  ___(_)___  _  __
/ /_/ / _ \/ _ \ / _ \| |/_/
\____/ .__/\___/_/\___/|_|
    /_/
						</pre>
					</div>

					<!-- Success Message -->
					<div class="text-center">
						<h2 class="mb-2 text-xl font-semibold text-zinc-100">Setup Complete!</h2>
						<p class="text-sm text-zinc-400">Your OpenClaw instance is ready to use</p>
					</div>

					<!-- Instance Summary Card -->
					{#if wizardStore.createdInstanceConfig}
						<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-4">
							<h3 class="mb-4 text-sm font-medium text-zinc-300">Instance Summary</h3>
							<div class="space-y-3">
								<div class="flex items-center justify-between">
									<span class="text-sm text-zinc-500">Name</span>
									<span class="text-sm font-medium text-zinc-100"
										>{wizardStore.createdInstanceConfig.name}</span
									>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-zinc-500">Version</span>
									<span class="text-sm text-zinc-100"
										>{wizardStore.createdInstanceConfig.openclaw_version}</span
									>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-zinc-500">Status</span>
									<div class="flex items-center gap-2">
										<StatusDot state={createdInstance?.status?.state || 'running'} size="sm" />
										<span class="text-sm text-zinc-100"
											>{createdInstance
												? formatInstanceState(createdInstance.status.state)
												: 'Running'}</span
										>
									</div>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-zinc-500">Gateway URL</span>
									<span class="font-mono text-sm text-emerald-400"
										>{getGatewayUrl(wizardStore.createdInstanceConfig)}</span
									>
								</div>
							</div>
						</div>
					{/if}

					<!-- Action Buttons -->
					<div class="flex flex-col gap-3">
						<button
							type="button"
							class="flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-600 px-4 py-3 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
							onclick={openGateway}
						>
							<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
								/>
							</svg>
							Open Gateway
						</button>
						<button
							type="button"
							class="flex w-full items-center justify-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-3 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-700"
							onclick={goToDashboard}
						>
							<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"
								/>
							</svg>
							Go to Dashboard
						</button>
					</div>
				</div>
			</div>
		{/if}
	</div>

	<!-- Wizard Footer -->
	<footer
		class="flex h-16 flex-shrink-0 items-center justify-end gap-3 border-t border-zinc-800 bg-zinc-900 px-6"
	>
		{#if error}
			<span class="mr-auto text-sm text-red-400">{error}</span>
		{/if}

		{#if wizardStore.currentStep === 'build' && buildComplete && !buildError}
			<button
				type="button"
				class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
				onclick={handleNext}
			>
				Continue
			</button>
		{:else if wizardStore.currentStep === 'build' && buildComplete && buildError}
			<button
				type="button"
				class="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-700"
				onclick={() => wizardStore.goToStep('config')}
			>
				Back to Settings
			</button>
		{:else if wizardStore.currentStep === 'provider'}
			<button
				type="button"
				class="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-700"
				onclick={handleNext}
			>
				Skip
			</button>
			{#if connectionSuccess}
				<button
					type="button"
					class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
					onclick={handleNext}
				>
					Continue
				</button>
			{:else}
				<button
					type="button"
					class="rounded-lg bg-zinc-700 px-4 py-2 text-sm font-medium text-zinc-400"
					disabled
					title="Connect a provider first or skip"
				>
					Continue
				</button>
			{/if}
		{:else if wizardStore.currentStep === 'channel'}
			<button
				type="button"
				class="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-700"
				onclick={handleNext}
			>
				Skip
			</button>
			<button
				type="button"
				class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
				onclick={handleNext}
			>
				Done
			</button>
		{:else if ['install-type', 'config'].includes(wizardStore.currentStep)}
			<button
				type="button"
				class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-50"
				disabled={isCreating || (wizardStore.currentStep === 'config' && wizardStore.hasFormErrors)}
				onclick={handleNext}
			>
				{#if isCreating}
					<span class="flex items-center gap-2">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white"
						></div>
						Creating...
					</span>
				{:else}
					Next
				{/if}
			</button>
		{/if}
	</footer>
</div>
