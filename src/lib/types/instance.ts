// TypeScript types matching Rust models in instance/models.rs

export type GatewayBind = 'loopback' | 'lan';

export type InstanceState = 'building' | 'running' | 'stopped' | 'error' | 'docker-not-running';

export type DockerState = 'running' | 'not-running' | 'not-installed';

export interface InstanceConfig {
	id: string;
	name: string;
	openclaw_version: string;
	container_id: string;
	gateway_port: number;
	bridge_port: number;
	gateway_bind: GatewayBind;
	gateway_token: string;
	timezone: string;
	install_browser: boolean;
	apt_packages: string;
	extensions: string;
	home_volume: string;
	extra_mounts: string;
	allow_insecure_ws: boolean;
	created_at: string;
	updated_at: string;
}

export interface InstanceSettings {
	name?: string;
	openclaw_version: string;
	gateway_port?: number;
	bridge_port?: number;
	gateway_bind: GatewayBind;
	timezone: string;
	install_browser: boolean;
	apt_packages: string;
	extensions: string;
	home_volume: string;
	extra_mounts: string;
	allow_insecure_ws: boolean;
}

export interface InstanceStatus {
	state: InstanceState;
	container_id?: string;
	error_message?: string;
}

export interface InstanceWithStatus extends InstanceConfig {
	status: InstanceStatus;
}

export interface DockerStatus {
	state: DockerState;
	compose_available: boolean;
}

export interface Release {
	tag: string;
	name: string;
	published_at: string;
	prerelease: boolean;
	commit_sha: string;
}

// Helper function to get gateway URL
export function getGatewayUrl(config: InstanceConfig): string {
	return `http://localhost:${config.gateway_port}?token=${config.gateway_token}`;
}

// Helper function to format instance state for display
export function formatInstanceState(state: InstanceState): string {
	switch (state) {
		case 'building':
			return 'Building';
		case 'running':
			return 'Running';
		case 'stopped':
			return 'Stopped';
		case 'error':
			return 'Error';
		case 'docker-not-running':
			return 'Docker Not Running';
		default:
			return 'Unknown';
	}
}

// Helper function to format Docker state for display
export function formatDockerState(state: DockerState): string {
	switch (state) {
		case 'running':
			return 'Docker Running';
		case 'not-running':
			return 'Docker Not Running';
		case 'not-installed':
			return 'Docker Not Installed';
		default:
			return 'Unknown';
	}
}
