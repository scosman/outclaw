<script lang="ts">
	import { wizardStore } from '$lib/stores/wizard.svelte';
	import type { GatewayBind, InstanceSettings } from '$lib/types/instance';

	interface Props {
		mode?: 'create' | 'edit';
		initialSettings?: Partial<InstanceSettings>;
		onSave?: (settings: InstanceSettings) => void;
		onCancel?: () => void;
	}

	let { mode = 'create', initialSettings, onSave, onCancel }: Props = $props();

	// Local state for form fields
	let localSettings = $state<InstanceSettings>({
		name: initialSettings?.name ?? wizardStore.settings.name ?? '',
		openclaw_version: initialSettings?.openclaw_version ?? wizardStore.settings.openclaw_version,
		gateway_port: initialSettings?.gateway_port ?? wizardStore.settings.gateway_port,
		bridge_port: initialSettings?.bridge_port ?? wizardStore.settings.bridge_port,
		gateway_bind: initialSettings?.gateway_bind ?? wizardStore.settings.gateway_bind,
		timezone: initialSettings?.timezone ?? wizardStore.settings.timezone,
		install_browser: initialSettings?.install_browser ?? wizardStore.settings.install_browser,
		apt_packages: initialSettings?.apt_packages ?? wizardStore.settings.apt_packages,
		extensions: initialSettings?.extensions ?? wizardStore.settings.extensions,
		home_volume: initialSettings?.home_volume ?? wizardStore.settings.home_volume,
		extra_mounts: initialSettings?.extra_mounts ?? wizardStore.settings.extra_mounts,
		allow_insecure_ws: initialSettings?.allow_insecure_ws ?? wizardStore.settings.allow_insecure_ws
	});

	let showAdvanced = $state(false);
	let nameTouched = $state(false);
	let gatewayPortTouched = $state(false);
	let bridgePortTouched = $state(false);

	// Validation
	const nameError = $derived(
		nameTouched && localSettings.name && localSettings.name.trim().length < 2
			? 'Name must be at least 2 characters'
			: undefined
	);

	const gatewayPortError = $derived(
		gatewayPortTouched &&
			localSettings.gateway_port !== undefined &&
			(localSettings.gateway_port < 1024 || localSettings.gateway_port > 65535)
			? 'Port must be between 1024 and 65535'
			: undefined
	);

	const bridgePortError = $derived(
		bridgePortTouched && localSettings.bridge_port !== undefined
			? localSettings.bridge_port < 1024 || localSettings.bridge_port > 65535
				? 'Port must be between 1024 and 65535'
				: localSettings.gateway_port !== undefined &&
					  localSettings.bridge_port === localSettings.gateway_port
					? 'Must be different from gateway port'
					: undefined
			: undefined
	);

	const hasErrors = $derived(!!nameError || !!gatewayPortError || !!bridgePortError);

	// Get releases from store
	const releases = $derived(wizardStore.releases);
	const releasesLoading = $derived(wizardStore.releasesLoading);

	// Update store when local settings change
	function updateLocalSettings(updates: Partial<InstanceSettings>) {
		localSettings = { ...localSettings, ...updates };
		wizardStore.updateSettings(localSettings);
	}

	// Handle save
	function handleSave() {
		if (hasErrors) return;
		wizardStore.updateSettings(localSettings);
		onSave?.(localSettings);
	}

	// Handle cancel
	function handleCancel() {
		onCancel?.();
	}

	// Toggle advanced section
	function toggleAdvanced() {
		showAdvanced = !showAdvanced;
	}
</script>

<div class="space-y-6">
	<!-- Instance Name -->
	<div>
		<label for="name" class="mb-1.5 block text-sm font-medium text-zinc-200">Instance Name</label>
		<input
			id="name"
			type="text"
			class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
			placeholder="e.g., Cosmic Otter"
			bind:value={localSettings.name}
			onblur={() => (nameTouched = true)}
			oninput={() => updateLocalSettings({ name: localSettings.name })}
		/>
		{#if nameError}
			<p class="mt-1 text-sm text-red-400">{nameError}</p>
		{/if}
	</div>

	<!-- OpenClaw Version -->
	<div>
		<label for="version" class="mb-1.5 block text-sm font-medium text-zinc-200">
			OpenClaw Version
		</label>
		<select
			id="version"
			class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
			bind:value={localSettings.openclaw_version}
			onchange={() => updateLocalSettings({ openclaw_version: localSettings.openclaw_version })}
			disabled={releasesLoading}
		>
			{#if releasesLoading}
				<option value="latest">Loading versions...</option>
			{:else}
				{#each releases as release (release.tag)}
					<option value={release.tag}>
						{release.tag}
						{release.prerelease ? ' (prerelease)' : ''}
					</option>
				{/each}
			{/if}
		</select>
		{#if releasesLoading}
			<p class="mt-1 text-xs text-zinc-500">Fetching available versions...</p>
		{/if}
	</div>

	<!-- Port Configuration (Side by Side) -->
	<div class="grid grid-cols-2 gap-4">
		<div>
			<label for="gateway-port" class="mb-1.5 block text-sm font-medium text-zinc-200">
				Gateway Port
			</label>
			<input
				id="gateway-port"
				type="number"
				min="1024"
				max="65535"
				class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
				placeholder="18789"
				bind:value={localSettings.gateway_port}
				onblur={() => (gatewayPortTouched = true)}
				oninput={() => updateLocalSettings({ gateway_port: localSettings.gateway_port })}
			/>
			{#if gatewayPortError}
				<p class="mt-1 text-sm text-red-400">{gatewayPortError}</p>
			{:else}
				<p class="mt-1 text-xs text-zinc-500">Default: 18789</p>
			{/if}
		</div>

		<div>
			<label for="bridge-port" class="mb-1.5 block text-sm font-medium text-zinc-200">
				Bridge Port
			</label>
			<input
				id="bridge-port"
				type="number"
				min="1024"
				max="65535"
				class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
				placeholder="18790"
				bind:value={localSettings.bridge_port}
				onblur={() => (bridgePortTouched = true)}
				oninput={() => updateLocalSettings({ bridge_port: localSettings.bridge_port })}
			/>
			{#if bridgePortError}
				<p class="mt-1 text-sm text-red-400">{bridgePortError}</p>
			{:else}
				<p class="mt-1 text-xs text-zinc-500">Default: 18790</p>
			{/if}
		</div>
	</div>

	<!-- Timezone -->
	<div>
		<label for="timezone" class="mb-1.5 block text-sm font-medium text-zinc-200">Timezone</label>
		<select
			id="timezone"
			class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
			bind:value={localSettings.timezone}
			onchange={() => updateLocalSettings({ timezone: localSettings.timezone })}
		>
			<option value="UTC">UTC</option>
			<optgroup label="Americas">
				<option value="America/New_York">America/New_York (Eastern)</option>
				<option value="America/Chicago">America/Chicago (Central)</option>
				<option value="America/Denver">America/Denver (Mountain)</option>
				<option value="America/Los_Angeles">America/Los_Angeles (Pacific)</option>
				<option value="America/Anchorage">America/Anchorage (Alaska)</option>
				<option value="America/Phoenix">America/Phoenix (Arizona)</option>
				<option value="America/Toronto">America/Toronto</option>
				<option value="America/Vancouver">America/Vancouver</option>
				<option value="America/Winnipeg">America/Winnipeg</option>
				<option value="America/Edmonton">America/Edmonton</option>
				<option value="America/Halifax">America/Halifax (Atlantic)</option>
				<option value="America/St_Johns">America/St_Johns (Newfoundland)</option>
				<option value="America/Mexico_City">America/Mexico_City</option>
				<option value="America/Tijuana">America/Tijuana</option>
				<option value="America/Bogota">America/Bogota</option>
				<option value="America/Lima">America/Lima</option>
				<option value="America/Santiago">America/Santiago</option>
				<option value="America/Argentina/Buenos_Aires">America/Argentina/Buenos_Aires</option>
				<option value="America/Sao_Paulo">America/Sao_Paulo</option>
				<option value="America/Caracas">America/Caracas</option>
			</optgroup>
			<optgroup label="Europe">
				<option value="Europe/London">Europe/London</option>
				<option value="Europe/Dublin">Europe/Dublin</option>
				<option value="Europe/Paris">Europe/Paris</option>
				<option value="Europe/Berlin">Europe/Berlin</option>
				<option value="Europe/Amsterdam">Europe/Amsterdam</option>
				<option value="Europe/Brussels">Europe/Brussels</option>
				<option value="Europe/Zurich">Europe/Zurich</option>
				<option value="Europe/Vienna">Europe/Vienna</option>
				<option value="Europe/Rome">Europe/Rome</option>
				<option value="Europe/Madrid">Europe/Madrid</option>
				<option value="Europe/Lisbon">Europe/Lisbon</option>
				<option value="Europe/Stockholm">Europe/Stockholm</option>
				<option value="Europe/Oslo">Europe/Oslo</option>
				<option value="Europe/Copenhagen">Europe/Copenhagen</option>
				<option value="Europe/Helsinki">Europe/Helsinki</option>
				<option value="Europe/Warsaw">Europe/Warsaw</option>
				<option value="Europe/Prague">Europe/Prague</option>
				<option value="Europe/Budapest">Europe/Budapest</option>
				<option value="Europe/Bucharest">Europe/Bucharest</option>
				<option value="Europe/Athens">Europe/Athens</option>
				<option value="Europe/Istanbul">Europe/Istanbul</option>
				<option value="Europe/Moscow">Europe/Moscow</option>
				<option value="Europe/Kiev">Europe/Kyiv</option>
			</optgroup>
			<optgroup label="Africa">
				<option value="Africa/Cairo">Africa/Cairo</option>
				<option value="Africa/Lagos">Africa/Lagos</option>
				<option value="Africa/Nairobi">Africa/Nairobi</option>
				<option value="Africa/Johannesburg">Africa/Johannesburg</option>
				<option value="Africa/Casablanca">Africa/Casablanca</option>
				<option value="Africa/Accra">Africa/Accra</option>
			</optgroup>
			<optgroup label="Asia">
				<option value="Asia/Tokyo">Asia/Tokyo</option>
				<option value="Asia/Shanghai">Asia/Shanghai</option>
				<option value="Asia/Hong_Kong">Asia/Hong_Kong</option>
				<option value="Asia/Taipei">Asia/Taipei</option>
				<option value="Asia/Seoul">Asia/Seoul</option>
				<option value="Asia/Singapore">Asia/Singapore</option>
				<option value="Asia/Kuala_Lumpur">Asia/Kuala_Lumpur</option>
				<option value="Asia/Bangkok">Asia/Bangkok</option>
				<option value="Asia/Ho_Chi_Minh">Asia/Ho_Chi_Minh</option>
				<option value="Asia/Jakarta">Asia/Jakarta</option>
				<option value="Asia/Manila">Asia/Manila</option>
				<option value="Asia/Kolkata">Asia/Kolkata</option>
				<option value="Asia/Colombo">Asia/Colombo</option>
				<option value="Asia/Karachi">Asia/Karachi</option>
				<option value="Asia/Dhaka">Asia/Dhaka</option>
				<option value="Asia/Dubai">Asia/Dubai</option>
				<option value="Asia/Riyadh">Asia/Riyadh</option>
				<option value="Asia/Tehran">Asia/Tehran</option>
				<option value="Asia/Jerusalem">Asia/Jerusalem</option>
				<option value="Asia/Almaty">Asia/Almaty</option>
				<option value="Asia/Vladivostok">Asia/Vladivostok</option>
			</optgroup>
			<optgroup label="Australia & Pacific">
				<option value="Australia/Sydney">Australia/Sydney</option>
				<option value="Australia/Melbourne">Australia/Melbourne</option>
				<option value="Australia/Brisbane">Australia/Brisbane</option>
				<option value="Australia/Perth">Australia/Perth</option>
				<option value="Australia/Adelaide">Australia/Adelaide</option>
				<option value="Australia/Darwin">Australia/Darwin</option>
				<option value="Pacific/Auckland">Pacific/Auckland</option>
				<option value="Pacific/Fiji">Pacific/Fiji</option>
				<option value="Pacific/Honolulu">Pacific/Honolulu</option>
				<option value="Pacific/Guam">Pacific/Guam</option>
			</optgroup>
		</select>
		<p class="mt-1 text-xs text-zinc-500">Detected: {wizardStore.systemTimezone}</p>
	</div>

	<!-- Install Browser Toggle -->
	<div class="flex items-center justify-between rounded-lg border border-zinc-700 p-3">
		<div>
			<span class="block text-sm font-medium text-zinc-100">Install Browser</span>
			<span class="block text-xs text-zinc-500"
				>Install a browser for web browsing capabilities</span
			>
		</div>
		<button
			type="button"
			role="switch"
			aria-checked={localSettings.install_browser}
			aria-label="Install browser toggle"
			class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 focus:ring-offset-zinc-900 {localSettings.install_browser
				? 'bg-emerald-500'
				: 'bg-zinc-600'}"
			onclick={() => updateLocalSettings({ install_browser: !localSettings.install_browser })}
		>
			<span
				class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {localSettings.install_browser
					? 'translate-x-5'
					: 'translate-x-0'}"
			></span>
		</button>
	</div>

	<!-- Advanced Options Section -->
	<div class="border-t border-zinc-700 pt-4">
		<button
			type="button"
			class="flex w-full items-center justify-between text-sm font-medium text-zinc-300 hover:text-zinc-100"
			onclick={toggleAdvanced}
		>
			<span>Advanced Options</span>
			<svg
				class="h-5 w-5 transition-transform {showAdvanced ? 'rotate-180' : ''}"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</button>

		{#if showAdvanced}
			<div class="mt-4 space-y-4">
				<!-- Network Access -->
				<div>
					<span class="mb-1.5 block text-sm font-medium text-zinc-200">Network Access</span>
					<div class="flex gap-4">
						<label
							class="flex cursor-pointer items-start gap-3 rounded-lg border border-zinc-700 p-3 transition-colors hover:border-zinc-600 has-[:checked]:border-emerald-500 has-[:checked]:bg-emerald-500/10"
						>
							<input
								type="radio"
								name="network-access"
								value="loopback"
								class="mt-0.5 h-4 w-4 border-zinc-600 bg-zinc-800 text-emerald-500 focus:ring-emerald-500 focus:ring-offset-zinc-900"
								checked={localSettings.gateway_bind === 'loopback'}
								onchange={() => updateLocalSettings({ gateway_bind: 'loopback' as GatewayBind })}
							/>
							<div>
								<span class="block text-sm font-medium text-zinc-100">Local Only</span>
								<span class="block text-xs text-zinc-500">Only accessible from this computer</span>
							</div>
						</label>

						<label
							class="flex cursor-pointer items-start gap-3 rounded-lg border border-zinc-700 p-3 transition-colors hover:border-zinc-600 has-[:checked]:border-emerald-500 has-[:checked]:bg-emerald-500/10"
						>
							<input
								type="radio"
								name="network-access"
								value="lan"
								class="mt-0.5 h-4 w-4 border-zinc-600 bg-zinc-800 text-emerald-500 focus:ring-emerald-500 focus:ring-offset-zinc-900"
								checked={localSettings.gateway_bind === 'lan'}
								onchange={() => updateLocalSettings({ gateway_bind: 'lan' as GatewayBind })}
							/>
							<div>
								<span class="block text-sm font-medium text-zinc-100">LAN Access</span>
								<span class="block text-xs text-zinc-500"
									>Accessible from other devices on your network</span
								>
							</div>
						</label>
					</div>
					<p class="mt-1 text-xs text-zinc-500">
						Allow portal access from other devices on your network. Does not affect chat access or
						the container's own internet access.
					</p>
				</div>
				<!-- APT Packages -->
				<div>
					<label for="apt-packages" class="mb-1.5 block text-sm font-medium text-zinc-200">
						Additional System Packages
					</label>
					<input
						id="apt-packages"
						type="text"
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
						placeholder="e.g., vim curl git"
						bind:value={localSettings.apt_packages}
						oninput={() => updateLocalSettings({ apt_packages: localSettings.apt_packages })}
					/>
					<p class="mt-1 text-xs text-zinc-500">Space-separated apt package names</p>
				</div>

				<!-- Extensions -->
				<div>
					<label for="extensions" class="mb-1.5 block text-sm font-medium text-zinc-200">
						Extensions
					</label>
					<input
						id="extensions"
						type="text"
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
						placeholder="e.g., @openclaw/extension-one @openclaw/extension-two"
						bind:value={localSettings.extensions}
						oninput={() => updateLocalSettings({ extensions: localSettings.extensions })}
					/>
					<p class="mt-1 text-xs text-zinc-500">Space-separated extension identifiers</p>
				</div>

				<!-- Home Volume -->
				<div>
					<label for="home-volume" class="mb-1.5 block text-sm font-medium text-zinc-200">
						Home Volume
					</label>
					<input
						id="home-volume"
						type="text"
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
						placeholder="Named volume or host path (optional)"
						bind:value={localSettings.home_volume}
						oninput={() => updateLocalSettings({ home_volume: localSettings.home_volume })}
					/>
					<p class="mt-1 text-xs text-zinc-500">Leave empty for default</p>
					{#if localSettings.home_volume}
						<p class="mt-1 text-xs text-amber-400">
							Warning: OpenClaw will have access to resources you provide here. Don't set unless you
							understand the security implications of this.
						</p>
					{/if}
				</div>

				<!-- Extra Mounts -->
				<div>
					<label for="extra-mounts" class="mb-1.5 block text-sm font-medium text-zinc-200">
						Extra Volume Mounts
					</label>
					<textarea
						id="extra-mounts"
						rows="2"
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
						placeholder="source:target[:options], e.g., /host/path:/container/path:ro"
						bind:value={localSettings.extra_mounts}
						oninput={() => updateLocalSettings({ extra_mounts: localSettings.extra_mounts })}
					></textarea>
					<p class="mt-1 text-xs text-zinc-500">Comma-separated volume mount specifications</p>
					{#if localSettings.extra_mounts}
						<p class="mt-1 text-xs text-amber-400">
							Warning: OpenClaw will have access to resources you provide here. Don't set unless you
							understand the security implications of this.
						</p>
					{/if}
				</div>

				<!-- Allow Insecure WebSocket -->
				<div class="flex items-center justify-between rounded-lg border border-zinc-700 p-3">
					<div>
						<span class="block text-sm font-medium text-zinc-100">Allow Insecure WebSocket</span>
						<span class="block text-xs text-amber-400"
							>Warning: May expose traffic to interception</span
						>
					</div>
					<button
						type="button"
						role="switch"
						aria-checked={localSettings.allow_insecure_ws}
						aria-label="Allow insecure WebSocket toggle"
						class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 focus:ring-offset-zinc-900 {localSettings.allow_insecure_ws
							? 'bg-emerald-500'
							: 'bg-zinc-600'}"
						onclick={() =>
							updateLocalSettings({ allow_insecure_ws: !localSettings.allow_insecure_ws })}
					>
						<span
							class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {localSettings.allow_insecure_ws
								? 'translate-x-5'
								: 'translate-x-0'}"
						></span>
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

<!-- Action Buttons (for edit mode) -->
{#if mode === 'edit'}
	<div class="mt-6 flex justify-end gap-3 border-t border-zinc-700 pt-4">
		<button
			type="button"
			class="rounded-lg border border-zinc-700 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
			onclick={handleCancel}
		>
			Cancel
		</button>
		<button
			type="button"
			class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-50"
			disabled={hasErrors}
			onclick={handleSave}
		>
			Save Changes
		</button>
	</div>
{/if}
