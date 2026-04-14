<script lang="ts">
	import type { SecurityPolicy, SeccompProfile, NetworkPreset } from '$lib/types/security';
	import { formatNetworkPreset, formatSeccompProfile, networkPresetColor } from '$lib/types/security';

	interface Props {
		policy: SecurityPolicy;
		onChange: (policy: SecurityPolicy) => void;
	}

	let { policy, onChange }: Props = $props();

	function update(updates: Partial<SecurityPolicy>) {
		onChange({ ...policy, ...updates });
	}

	function updateSandbox(updates: Partial<SecurityPolicy['sandbox']>) {
		update({ sandbox: { ...policy.sandbox, ...updates } });
	}

	function updateNetwork(updates: Partial<SecurityPolicy['network']>) {
		update({ network: { ...policy.network, ...updates } });
	}

	const securityLevel = $derived(() => {
		let score = 0;
		if (policy.sandbox.drop_all_capabilities) score++;
		if (policy.sandbox.no_new_privileges) score++;
		if (policy.sandbox.seccomp_profile !== 'unconfined') score++;
		if (policy.sandbox.pids_limit !== null) score++;
		if (policy.network.preset === 'strict') score += 2;
		else if (policy.network.preset === 'moderate') score++;
		return score;
	});

	const securityColor = $derived(
		securityLevel() >= 5 ? 'text-emerald-400' : securityLevel() >= 3 ? 'text-amber-400' : 'text-red-400'
	);

	const securityLabel = $derived(
		securityLevel() >= 5 ? 'Strong' : securityLevel() >= 3 ? 'Moderate' : 'Weak'
	);
</script>

<div class="space-y-5">
	<!-- Security Summary -->
	<div class="flex items-center justify-between rounded-lg border border-zinc-700 bg-zinc-800/50 p-3">
		<div>
			<span class="block text-sm font-medium text-zinc-100">Security Posture</span>
			<span class="block text-xs text-zinc-500">Overall security level of this instance</span>
		</div>
		<span class="text-sm font-semibold {securityColor}">{securityLabel}</span>
	</div>

	<!-- Sandbox Hardening Section -->
	<div>
		<h4 class="mb-3 text-sm font-medium text-zinc-300">Sandbox Hardening</h4>
		<div class="space-y-3">
			<!-- Drop All Capabilities -->
			<div class="flex items-center justify-between rounded-lg border border-zinc-700 p-3">
				<div>
					<span class="block text-sm font-medium text-zinc-100">Drop All Capabilities</span>
					<span class="block text-xs text-zinc-500">Remove all Linux capabilities from container</span>
				</div>
				<button
					type="button"
					role="switch"
					aria-checked={policy.sandbox.drop_all_capabilities}
					aria-label="Drop all capabilities toggle"
					class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 focus:ring-offset-zinc-900 {policy.sandbox.drop_all_capabilities ? 'bg-emerald-500' : 'bg-zinc-600'}"
					onclick={() => updateSandbox({ drop_all_capabilities: !policy.sandbox.drop_all_capabilities })}
				>
					<span class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {policy.sandbox.drop_all_capabilities ? 'translate-x-5' : 'translate-x-0'}"></span>
				</button>
			</div>

			<!-- No New Privileges -->
			<div class="flex items-center justify-between rounded-lg border border-zinc-700 p-3">
				<div>
					<span class="block text-sm font-medium text-zinc-100">No New Privileges</span>
					<span class="block text-xs text-zinc-500">Prevent processes from gaining additional privileges</span>
				</div>
				<button
					type="button"
					role="switch"
					aria-checked={policy.sandbox.no_new_privileges}
					aria-label="No new privileges toggle"
					class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 focus:ring-offset-zinc-900 {policy.sandbox.no_new_privileges ? 'bg-emerald-500' : 'bg-zinc-600'}"
					onclick={() => updateSandbox({ no_new_privileges: !policy.sandbox.no_new_privileges })}
				>
					<span class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {policy.sandbox.no_new_privileges ? 'translate-x-5' : 'translate-x-0'}"></span>
				</button>
			</div>

			<!-- Seccomp Profile -->
			<div>
				<label for="seccomp-profile" class="mb-1.5 block text-sm font-medium text-zinc-200">
					Seccomp Profile
				</label>
				<select
					id="seccomp-profile"
					class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
					value={policy.sandbox.seccomp_profile}
					onchange={(e) => updateSandbox({ seccomp_profile: (e.target as HTMLSelectElement).value as SeccompProfile })}
				>
					<option value="default">{formatSeccompProfile('default')} - Docker's built-in profile</option>
					<option value="strict">{formatSeccompProfile('strict')} - Block additional dangerous syscalls</option>
					<option value="unconfined">{formatSeccompProfile('unconfined')} - No restrictions (not recommended)</option>
				</select>
				{#if policy.sandbox.seccomp_profile === 'unconfined'}
					<p class="mt-1 text-xs text-red-400">
						Warning: Unconfined seccomp allows all syscalls. This significantly reduces container security.
					</p>
				{/if}
			</div>

			<!-- PID Limit -->
			<div>
				<label for="pids-limit" class="mb-1.5 block text-sm font-medium text-zinc-200">
					PID Limit
				</label>
				<input
					id="pids-limit"
					type="number"
					min="0"
					max="32768"
					class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
					placeholder="256 (default)"
					value={policy.sandbox.pids_limit ?? ''}
					oninput={(e) => {
						const val = (e.target as HTMLInputElement).value;
						updateSandbox({ pids_limit: val ? parseInt(val) : null });
					}}
				/>
				<p class="mt-1 text-xs text-zinc-500">Maximum number of processes in container. Leave empty for unlimited.</p>
			</div>

			<!-- Memory Limit -->
			<div>
				<label for="memory-limit" class="mb-1.5 block text-sm font-medium text-zinc-200">
					Memory Limit
				</label>
				<input
					id="memory-limit"
					type="text"
					class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
					placeholder="e.g., 2g, 512m (empty for unlimited)"
					value={policy.sandbox.memory_limit ?? ''}
					oninput={(e) => {
						const val = (e.target as HTMLInputElement).value;
						updateSandbox({ memory_limit: val || null });
					}}
				/>
			</div>

			<!-- CPU Limit -->
			<div>
				<label for="cpu-limit" class="mb-1.5 block text-sm font-medium text-zinc-200">
					CPU Limit
				</label>
				<input
					id="cpu-limit"
					type="number"
					min="0"
					max="128"
					step="0.5"
					class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
					placeholder="e.g., 2.0 (empty for unlimited)"
					value={policy.sandbox.cpu_limit ?? ''}
					oninput={(e) => {
						const val = (e.target as HTMLInputElement).value;
						updateSandbox({ cpu_limit: val ? parseFloat(val) : null });
					}}
				/>
			</div>
		</div>
	</div>

	<!-- Network Security Section -->
	<div>
		<h4 class="mb-3 text-sm font-medium text-zinc-300">Network Security</h4>
		<div class="space-y-3">
			<!-- Network Preset -->
			<div>
				<span class="mb-1.5 block text-sm font-medium text-zinc-200">Network Policy</span>
				<div class="space-y-2">
					{#each ['strict', 'moderate', 'permissive'] as preset}
						{@const presetTyped = preset as NetworkPreset}
						<label
							class="flex cursor-pointer items-start gap-3 rounded-lg border border-zinc-700 p-3 transition-colors hover:border-zinc-600 has-[:checked]:border-emerald-500 has-[:checked]:bg-emerald-500/10"
						>
							<input
								type="radio"
								name="network-preset"
								value={preset}
								class="mt-0.5 h-4 w-4 border-zinc-600 bg-zinc-800 text-emerald-500 focus:ring-emerald-500 focus:ring-offset-zinc-900"
								checked={policy.network.preset === preset}
								onchange={() => updateNetwork({ preset: presetTyped })}
							/>
							<div class="flex-1">
								<div class="flex items-center gap-2">
									<span class="block text-sm font-medium text-zinc-100">{formatNetworkPreset(presetTyped)}</span>
									<span class="inline-block h-2 w-2 rounded-full bg-{networkPresetColor(presetTyped)}-400"></span>
								</div>
								<span class="block text-xs text-zinc-500">
									{#if preset === 'strict'}
										Container has no outbound network access
									{:else if preset === 'moderate'}
										Allow HTTPS and DNS traffic only
									{:else}
										No network restrictions (current default)
									{/if}
								</span>
							</div>
						</label>
					{/each}
				</div>
			</div>

			<!-- Allowed Domains (for strict/moderate) -->
			{#if policy.network.preset !== 'permissive'}
				<div>
					<label for="allowed-domains" class="mb-1.5 block text-sm font-medium text-zinc-200">
						Allowed Egress Domains
					</label>
					<textarea
						id="allowed-domains"
						rows="3"
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 placeholder-zinc-500 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
						placeholder="One domain per line, e.g.&#10;api.openai.com&#10;api.anthropic.com"
						value={policy.network.allowed_egress_domains.join('\n')}
						oninput={(e) => {
							const val = (e.target as HTMLTextAreaElement).value;
							updateNetwork({
								allowed_egress_domains: val.split('\n').map(d => d.trim()).filter(d => d.length > 0)
							});
						}}
					></textarea>
					<p class="mt-1 text-xs text-zinc-500">Domains the container is allowed to reach</p>
				</div>
			{/if}
		</div>
	</div>
</div>
