<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { wizardStore } from '$lib/stores/wizard.svelte';
	import ConfigForm from '$lib/components/ConfigForm.svelte';
	import StatusDot from '$lib/components/StatusDot.svelte';

	let isLoading = $state(true);
	let isCreating = $state(false);
	let error = $state<string | null>(null);

	// Initialize wizard on mount
	onMount(async () => {
		wizardStore.reset();
		await wizardStore.initialize();
		isLoading = false;
	});

	// Computed step info
	const stepLabels: Record<string, string> = {
		'install-type': 'Install Type',
		config: 'Configuration',
		build: 'Building',
		provider: 'Provider Setup',
		channel: 'Channel Setup',
		complete: 'Complete'
	};

	const currentStepLabel = $derived(stepLabels[wizardStore.currentStep] || 'Unknown');
	const stepNumber = $derived(wizardStore.stepNumber);
	const totalSteps = $derived(wizardStore.totalSteps);

	// Navigation handlers
	function handleBack() {
		if (wizardStore.currentStep === 'config') {
			wizardStore.goToStep('install-type');
		} else if (wizardStore.currentStep === 'build') {
			wizardStore.goToStep('config');
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
			// Build complete, go to provider setup (Phase 6)
			// For Phase 4, just go to placeholder
			wizardStore.nextStep();
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

	// Cleanup on unmount
	onMount(() => {
		return () => {
			wizardStore.cleanup();
		};
	});
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

		<button
			type="button"
			class="text-sm text-zinc-400 transition-colors hover:text-zinc-200"
			onclick={handleCancel}
		>
			Cancel
		</button>
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
									<li>• Auto-generated instance name</li>
									<li>• Latest OpenClaw version</li>
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
									<li>• Choose OpenClaw version</li>
									<li>• Configure custom ports, LAN access, timezone, and more</li>
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
			<!-- Step 4: Build Progress (placeholder - full implementation in Phase 5) -->
			<div class="mx-auto w-full max-w-2xl px-6 py-8">
				<div class="mb-6 text-center">
					<h2 class="mb-2 text-xl font-semibold text-zinc-100">Building Instance</h2>
					<p class="text-sm text-zinc-400">Setting up your OpenClaw instance...</p>
				</div>

				<div class="rounded-lg border border-zinc-800 bg-zinc-900/50 p-6">
					{#if wizardStore.buildState}
						<!-- Build progress -->
						<div class="space-y-4">
							<!-- Stage indicator -->
							<div class="flex items-center gap-3">
								{#if wizardStore.buildState.error}
									<StatusDot state="error" />
								{:else if wizardStore.buildState.done}
									<StatusDot state="running" />
								{:else}
									<div
										class="h-2.5 w-2.5 animate-spin rounded-full border-2 border-zinc-700 border-t-emerald-500"
									></div>
								{/if}
								<span class="text-sm font-medium text-zinc-200">
									{wizardStore.buildState.stage || 'Starting...'}
								</span>
							</div>

							<!-- Log output -->
							<div class="max-h-64 overflow-auto rounded bg-zinc-800 p-3">
								{#each wizardStore.buildState.logs as log, i (i)}
									<pre class="whitespace-pre-wrap text-xs text-zinc-400">{log}</pre>
								{/each}
							</div>

							{#if wizardStore.buildState.error}
								<div class="rounded-lg border border-red-500/30 bg-red-500/10 p-4">
									<p class="text-sm font-medium text-red-400">Build Failed</p>
									<p class="mt-1 text-xs text-red-400/80">{wizardStore.buildState.error}</p>
								</div>
							{:else if wizardStore.buildState.done}
								<div class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-4">
									<p class="text-sm font-medium text-emerald-400">Build Complete!</p>
									<p class="mt-1 text-xs text-emerald-400/80">Your instance is ready to use.</p>
								</div>
							{/if}
						</div>
					{:else}
						<!-- Loading state -->
						<div class="flex items-center justify-center py-8">
							<div class="text-center">
								<div
									class="mb-4 inline-block h-8 w-8 animate-spin rounded-full border-4 border-zinc-700 border-t-emerald-500"
								></div>
								<p class="text-sm text-zinc-400">Starting build...</p>
							</div>
						</div>
					{/if}
				</div>
			</div>
		{:else}
			<!-- Future steps placeholder -->
			<div class="flex flex-1 items-center justify-center">
				<div class="text-center">
					<h2 class="mb-2 text-xl font-semibold text-zinc-100">{currentStepLabel}</h2>
					<p class="text-zinc-500">This step will be implemented in a future phase.</p>
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

		{#if wizardStore.currentStep === 'build' && wizardStore.buildState?.done && !wizardStore.buildState?.error}
			<button
				type="button"
				class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700"
				onclick={() => goto('/')}
			>
				Go to Dashboard
			</button>
		{:else if ['install-type', 'config', 'build'].includes(wizardStore.currentStep)}
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
				{:else if wizardStore.currentStep === 'build'}
					Continue
				{:else}
					Next
				{/if}
			</button>
		{/if}
	</footer>
</div>
