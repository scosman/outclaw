<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import type { AuditEntry } from '$lib/types/security';
	import { formatAuditAction } from '$lib/types/security';

	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		await loadEntries();
	});

	async function loadEntries() {
		loading = true;
		error = null;
		try {
			entries = await invoke<AuditEntry[]>('get_audit_log', { count: 100 });
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	function formatTimestamp(ts: string): string {
		try {
			return new Date(ts).toLocaleString();
		} catch {
			return ts;
		}
	}

	function outcomeColor(outcome: AuditEntry['outcome']): string {
		if (outcome === 'success') return 'text-emerald-400';
		if (outcome === 'denied') return 'text-red-400';
		return 'text-amber-400';
	}

	function outcomeLabel(outcome: AuditEntry['outcome']): string {
		if (outcome === 'success') return 'Success';
		if (outcome === 'denied') return 'Denied';
		if (typeof outcome === 'object' && 'error' in outcome) return 'Error';
		return 'Unknown';
	}
</script>

<div class="space-y-3">
	<div class="flex items-center justify-between">
		<h3 class="text-sm font-medium text-zinc-300">Security Audit Log</h3>
		<button
			type="button"
			class="rounded px-2 py-1 text-xs text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200"
			onclick={loadEntries}
			disabled={loading}
		>
			Refresh
		</button>
	</div>

	{#if loading}
		<p class="text-sm text-zinc-500">Loading audit log...</p>
	{:else if error}
		<p class="text-sm text-red-400">Failed to load audit log: {error}</p>
	{:else if entries.length === 0}
		<p class="text-sm text-zinc-500">No audit log entries yet.</p>
	{:else}
		<div class="max-h-96 overflow-y-auto rounded-lg border border-zinc-700">
			<table class="w-full text-left text-xs">
				<thead class="sticky top-0 border-b border-zinc-700 bg-zinc-800 text-zinc-400">
					<tr>
						<th class="px-3 py-2">Time</th>
						<th class="px-3 py-2">Action</th>
						<th class="px-3 py-2">Instance</th>
						<th class="px-3 py-2">Outcome</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-zinc-800">
					{#each entries as entry}
						<tr class="hover:bg-zinc-800/50">
							<td class="whitespace-nowrap px-3 py-2 text-zinc-400">{formatTimestamp(entry.timestamp)}</td>
							<td class="px-3 py-2 text-zinc-200">{formatAuditAction(entry.action)}</td>
							<td class="px-3 py-2 font-mono text-zinc-400">{entry.instance_id ?? '-'}</td>
							<td class="px-3 py-2">
								<span class="{outcomeColor(entry.outcome)} font-medium">{outcomeLabel(entry.outcome)}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
