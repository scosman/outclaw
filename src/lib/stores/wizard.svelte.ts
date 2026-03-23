import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { InstanceSettings, Release, InstanceConfig } from '$lib/types/instance';

// Wizard steps
export type WizardStep = 'install-type' | 'config' | 'build' | 'provider' | 'channel' | 'complete';

export type InstallType = 'standard' | 'custom';

// Build progress event from backend
interface BuildProgressEvent {
	id: string;
	stage: string;
	log: string;
	done: boolean;
	error?: string;
}

// Build state tracked during build step
interface BuildState {
	stage: string;
	logs: string[];
	done: boolean;
	error?: string;
}

// Form validation errors
interface FormErrors {
	name?: string;
	openclaw_version?: string;
	gateway_port?: string;
	bridge_port?: string;
	timezone?: string;
	apt_packages?: string;
	extensions?: string;
	home_volume?: string;
	extra_mounts?: string;
}

// Reactive state using Svelte 5 runes
let currentStep = $state<WizardStep>('install-type');
let installType = $state<InstallType>('standard');
let settings = $state<InstanceSettings>({
	name: undefined,
	openclaw_version: 'latest',
	gateway_port: undefined,
	bridge_port: undefined,
	gateway_bind: 'loopback',
	timezone: 'UTC',
	install_browser: true,
	apt_packages: '',
	extensions: '',
	home_volume: '',
	extra_mounts: '',
	allow_insecure_ws: false
});
let formErrors = $state<FormErrors>({});
let buildState = $state<BuildState | null>(null);
let createdInstanceId = $state<string | null>(null);
let createdInstanceConfig = $state<InstanceConfig | null>(null);
let releases = $state<Release[]>([]);
let releasesLoading = $state(false);
let generatedName = $state<string>('');
let systemTimezone = $state<string>('UTC');

let unlistenBuild: UnlistenFn | null = null;

// Step order for navigation
const stepOrder: WizardStep[] = [
	'install-type',
	'config',
	'build',
	'provider',
	'channel',
	'complete'
];

// Get step index
function getStepIndex(step: WizardStep): number {
	return stepOrder.indexOf(step);
}

// Computed properties
const stepNumber = $derived(getStepIndex(currentStep) + 1);
const totalSteps = $derived(stepOrder.length);
const canGoBack = $derived(getStepIndex(currentStep) > 0);
const isStandardInstall = $derived(installType === 'standard');
const hasFormErrors = $derived(Object.values(formErrors).some((v) => v !== undefined));

// Reset wizard state
function reset() {
	currentStep = 'install-type';
	installType = 'standard';
	settings = {
		name: undefined,
		openclaw_version: 'latest',
		gateway_port: undefined,
		bridge_port: undefined,
		gateway_bind: 'loopback',
		timezone: 'UTC',
		install_browser: true,
		apt_packages: '',
		extensions: '',
		home_volume: '',
		extra_mounts: '',
		allow_insecure_ws: false
	};
	formErrors = {};
	buildState = null;
	createdInstanceId = null;
	createdInstanceConfig = null;
}

// Initialize wizard data (fetch releases, generate name, get timezone)
async function initialize() {
	// Fetch releases
	releasesLoading = true;
	try {
		const result = await invoke<Release[]>('get_releases');
		releases = result;
		// Set default version to first non-prerelease
		const latest = result.find((r) => !r.prerelease);
		if (latest) {
			settings.openclaw_version = latest.tag;
		}
	} catch (error) {
		console.error('Failed to fetch releases:', error);
	} finally {
		releasesLoading = false;
	}

	// Generate instance name
	try {
		generatedName = await invoke<string>('generate_instance_name');
		settings.name = generatedName;
	} catch (error) {
		console.error('Failed to generate name:', error);
		settings.name = 'My OpenClaw';
	}

	// Get system timezone
	try {
		systemTimezone = await invoke<string>('get_system_timezone');
		settings.timezone = systemTimezone;
	} catch (error) {
		console.error('Failed to get timezone:', error);
	}
}

// Navigate to a specific step
function goToStep(step: WizardStep) {
	currentStep = step;
}

// Go to next step
function nextStep() {
	const currentIndex = getStepIndex(currentStep);
	if (currentIndex < stepOrder.length - 1) {
		currentStep = stepOrder[currentIndex + 1];
	}
}

// Go to previous step
function prevStep() {
	const currentIndex = getStepIndex(currentStep);
	if (currentIndex > 0) {
		currentStep = stepOrder[currentIndex - 1];
	}
}

// Validate form fields
function validateForm(): boolean {
	formErrors = {};

	// Name validation
	if (settings.name && settings.name.trim().length < 2) {
		formErrors.name = 'Name must be at least 2 characters';
	}

	// Port validation
	if (settings.gateway_port !== undefined) {
		if (settings.gateway_port < 1024 || settings.gateway_port > 65535) {
			formErrors.gateway_port = 'Port must be between 1024 and 65535';
		}
	}

	if (settings.bridge_port !== undefined) {
		if (settings.bridge_port < 1024 || settings.bridge_port > 65535) {
			formErrors.bridge_port = 'Port must be between 1024 and 65535';
		}
	}

	// Port conflict check
	if (
		settings.gateway_port !== undefined &&
		settings.bridge_port !== undefined &&
		settings.gateway_port === settings.bridge_port
	) {
		formErrors.bridge_port = 'Bridge port must be different from gateway port';
	}

	return Object.keys(formErrors).length === 0;
}

// Update settings
function updateSettings(updates: Partial<InstanceSettings>) {
	settings = { ...settings, ...updates };
}

// Set install type
function setInstallType(type: InstallType) {
	installType = type;
}

// Create instance (calls backend)
async function createInstance(): Promise<string | null> {
	try {
		// Prepare settings with defaults
		const finalSettings: InstanceSettings = {
			name: settings.name || generatedName,
			openclaw_version: settings.openclaw_version,
			gateway_port: settings.gateway_port,
			bridge_port: settings.bridge_port,
			gateway_bind: settings.gateway_bind,
			timezone: settings.timezone,
			install_browser: settings.install_browser,
			apt_packages: settings.apt_packages || '',
			extensions: settings.extensions || '',
			home_volume: settings.home_volume || '',
			extra_mounts: settings.extra_mounts || '',
			allow_insecure_ws: settings.allow_insecure_ws
		};

		const config = await invoke<InstanceConfig>('create_instance', { settings: finalSettings });
		createdInstanceId = config.id;
		createdInstanceConfig = config;
		return config.id;
	} catch (error) {
		console.error('Failed to create instance:', error);
		throw error;
	}
}

// Start build and listen for progress
async function startBuild(): Promise<void> {
	if (!createdInstanceId) {
		throw new Error('No instance created');
	}

	// Reset build state
	buildState = {
		stage: 'starting',
		logs: [],
		done: false
	};

	// Listen for build progress events
	unlistenBuild = await listen<BuildProgressEvent>('build-progress', (event) => {
		const { stage, log, done, error } = event.payload;

		if (buildState) {
			buildState.stage = stage;
			if (log) {
				buildState.logs.push(log);
			}
			buildState.done = done;
			if (error) {
				buildState.error = error;
			}
		}
	});

	try {
		await invoke('build_instance', { id: createdInstanceId });
	} catch (error) {
		if (buildState) {
			buildState.error = String(error);
			buildState.done = true;
		}
		throw error;
	}
}

// Cleanup build listener
function cleanup() {
	if (unlistenBuild) {
		unlistenBuild();
		unlistenBuild = null;
	}
}

// Export store object
export const wizardStore = {
	get currentStep() {
		return currentStep;
	},
	get installType() {
		return installType;
	},
	get settings() {
		return settings;
	},
	get formErrors() {
		return formErrors;
	},
	get buildState() {
		return buildState;
	},
	get createdInstanceId() {
		return createdInstanceId;
	},
	get createdInstanceConfig() {
		return createdInstanceConfig;
	},
	get releases() {
		return releases;
	},
	get releasesLoading() {
		return releasesLoading;
	},
	get generatedName() {
		return generatedName;
	},
	get systemTimezone() {
		return systemTimezone;
	},
	get stepNumber() {
		return stepNumber;
	},
	get totalSteps() {
		return totalSteps;
	},
	get canGoBack() {
		return canGoBack;
	},
	get isStandardInstall() {
		return isStandardInstall;
	},
	get hasFormErrors() {
		return hasFormErrors;
	},
	reset,
	initialize,
	goToStep,
	nextStep,
	prevStep,
	validateForm,
	updateSettings,
	setInstallType,
	createInstance,
	startBuild,
	cleanup
};
