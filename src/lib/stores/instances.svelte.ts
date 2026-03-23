import { SvelteMap } from 'svelte/reactivity';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { InstanceWithStatus, InstanceStatus } from '$lib/types/instance';

// Reactive state using Svelte 5 runes (module-level, shared)
let instances = new SvelteMap<string, InstanceWithStatus>();
let loading = $state(true);
let initialized = $state(false);

let unlisten: UnlistenFn | null = null;

// Initialize the store (call once)
async function initialize() {
	if (initialized) return;

	loading = true;
	try {
		const result = await invoke<InstanceWithStatus[]>('list_instances');
		instances = new SvelteMap(result.map((inst) => [inst.id, inst]));
	} catch (error) {
		console.error('Failed to list instances:', error);
		instances = new SvelteMap();
	} finally {
		loading = false;
		initialized = true;
	}

	// Listen for instance status changes from the backend poller
	unlisten = await listen<{ id: string; status: InstanceStatus }>(
		'instance-status-changed',
		(event) => {
			const { id, status } = event.payload;
			const instance = instances.get(id);
			if (instance) {
				instances.set(id, { ...instance, status });
			}
		}
	);
}

// Cleanup function
function cleanup() {
	if (unlisten) {
		unlisten();
		unlisten = null;
	}
}

// Get a single instance by ID
function getInstance(id: string): InstanceWithStatus | undefined {
	return instances.get(id);
}

// Add or update an instance
function setInstance(instance: InstanceWithStatus) {
	instances.set(instance.id, instance);
}

// Remove an instance
function removeInstance(id: string) {
	instances.delete(id);
}

// Computed properties
const instanceList = $derived(Array.from(instances.values()));
const instanceCount = $derived(instances.size);
const hasInstances = $derived(instances.size > 0);
const runningCount = $derived(instanceList.filter((i) => i.status.state === 'running').length);

// Export store object
export const instancesStore = {
	get instances() {
		return instances;
	},
	get loading() {
		return loading;
	},
	get instanceList() {
		return instanceList;
	},
	get instanceCount() {
		return instanceCount;
	},
	get hasInstances() {
		return hasInstances;
	},
	get runningCount() {
		return runningCount;
	},
	initialize,
	cleanup,
	getInstance,
	setInstance,
	removeInstance
};
