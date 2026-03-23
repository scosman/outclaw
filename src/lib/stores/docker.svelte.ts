import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { DockerStatus } from '$lib/types/instance';

// Reactive state using Svelte 5 runes (module-level, shared)
let status = $state<DockerStatus>({
	state: 'not-running',
	compose_available: false
});
let loading = $state(true);
let initialized = $state(false);

let unlisten: UnlistenFn | null = null;

// Initialize the store (call once)
async function initialize() {
	if (initialized) return;

	loading = true;
	try {
		const result = await invoke<DockerStatus>('check_docker');
		status = result;
	} catch (error) {
		console.error('Failed to check Docker status:', error);
		status = { state: 'not-installed', compose_available: false };
	} finally {
		loading = false;
		initialized = true;
	}

	// Listen for Docker status changes from the backend poller
	unlisten = await listen<DockerStatus>('docker-status-changed', (event) => {
		status = event.payload;
	});
}

// Cleanup function
function cleanup() {
	if (unlisten) {
		unlisten();
		unlisten = null;
	}
}

// Computed properties
const isRunning = $derived(status.state === 'running');
const isNotInstalled = $derived(status.state === 'not-installed');
const isNotRunning = $derived(status.state === 'not-running');
const isAvailable = $derived(status.state === 'running' && status.compose_available);

// Export store object - call initialize() in onMount
export const dockerStore = {
	get status() {
		return status;
	},
	get loading() {
		return loading;
	},
	get isRunning() {
		return isRunning;
	},
	get isNotInstalled() {
		return isNotInstalled;
	},
	get isNotRunning() {
		return isNotRunning;
	},
	get isAvailable() {
		return isAvailable;
	},
	initialize,
	cleanup
};
