// TypeScript types matching Rust security models

export type SeccompProfile = 'default' | 'strict' | 'unconfined';

export type NetworkPreset = 'strict' | 'moderate' | 'permissive';

export interface SandboxPolicy {
	drop_all_capabilities: boolean;
	added_capabilities: string[];
	seccomp_profile: SeccompProfile;
	no_new_privileges: boolean;
	pids_limit: number | null;
	memory_limit: string | null;
	cpu_limit: number | null;
}

export interface NetworkPolicy {
	preset: NetworkPreset;
	allowed_egress_domains: string[];
	allowed_egress_cidrs: string[];
}

export interface SecurityPolicy {
	sandbox: SandboxPolicy;
	network: NetworkPolicy;
}

export type ApprovalType = 'seccomp_unconfined' | 'network_permissive' | 'disable_cap_drop';

export interface ApprovalRequest {
	approval_type: ApprovalType;
	description: string;
	current_value: string;
	new_value: string;
}

export type AuditAction =
	| 'instance_created'
	| 'instance_deleted'
	| 'instance_started'
	| 'instance_stopped'
	| 'instance_restarted'
	| 'config_changed'
	| 'security_policy_changed'
	| 'security_approval_granted'
	| 'provider_connected'
	| 'build_started'
	| 'build_completed'
	| 'build_failed'
	| 'input_validation_failed'
	| 'ssrf_blocked'
	| 'rate_limit_hit';

export type AuditOutcome = 'success' | 'denied' | { error: string };

export interface AuditEntry {
	timestamp: string;
	action: AuditAction;
	instance_id?: string;
	details?: Record<string, unknown>;
	outcome: AuditOutcome;
}

/** Default security policy matching Rust defaults */
export function defaultSecurityPolicy(): SecurityPolicy {
	return {
		sandbox: {
			drop_all_capabilities: true,
			added_capabilities: [],
			seccomp_profile: 'default',
			no_new_privileges: true,
			pids_limit: 256,
			memory_limit: null,
			cpu_limit: null
		},
		network: {
			preset: 'permissive',
			allowed_egress_domains: [],
			allowed_egress_cidrs: []
		}
	};
}

/** Get a human-readable label for a network preset */
export function formatNetworkPreset(preset: NetworkPreset): string {
	switch (preset) {
		case 'strict':
			return 'Strict (No Network)';
		case 'moderate':
			return 'Moderate (HTTPS/DNS Only)';
		case 'permissive':
			return 'Permissive (No Restrictions)';
		default:
			return 'Unknown';
	}
}

/** Get a human-readable label for a seccomp profile */
export function formatSeccompProfile(profile: SeccompProfile): string {
	switch (profile) {
		case 'default':
			return 'Default';
		case 'strict':
			return 'Strict';
		case 'unconfined':
			return 'Unconfined';
		default:
			return 'Unknown';
	}
}

/** Get a security level indicator color for a network preset */
export function networkPresetColor(preset: NetworkPreset): 'green' | 'yellow' | 'red' {
	switch (preset) {
		case 'strict':
			return 'green';
		case 'moderate':
			return 'yellow';
		case 'permissive':
			return 'red';
		default:
			return 'red';
	}
}

/** Format an audit action for display */
export function formatAuditAction(action: AuditAction): string {
	return action
		.split('_')
		.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
		.join(' ');
}
