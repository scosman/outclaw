/**
 * Provider configuration for the wizard connection form.
 * Each provider defines the fields needed to connect via the OpenClaw CLI.
 */

export interface ProviderField {
	name: string;
	label: string;
	secret: boolean;
	placeholder?: string;
	required?: boolean;
	defaultValue?: string;
}

export interface ProviderConfig {
	id: string;
	label: string;
	description: string;
	fields: ProviderField[];
}

/**
 * All supported LLM providers.
 * The `id` must match the --auth-choice value expected by the OpenClaw CLI.
 */
export const PROVIDERS: ProviderConfig[] = [
	{
		id: 'apiKey',
		label: 'Anthropic',
		description: 'Claude and other Anthropic models',
		fields: [
			{
				name: 'anthropic-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'sk-ant-...',
				required: true
			}
		]
	},
	{
		id: 'openai-api-key',
		label: 'OpenAI',
		description: 'GPT-4, GPT-3.5, and other OpenAI models',
		fields: [
			{
				name: 'openai-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'sk-...',
				required: true
			}
		]
	},
	{
		id: 'gemini-api-key',
		label: 'Google Gemini',
		description: 'Gemini Pro and other Google AI models',
		fields: [
			{
				name: 'gemini-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'AI...',
				required: true
			}
		]
	},
	{
		id: 'mistral-api-key',
		label: 'Mistral',
		description: 'Mistral and Mixtral models',
		fields: [
			{
				name: 'mistral-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'zai-api-key',
		label: 'Z.ai',
		description: 'Z.ai API models',
		fields: [
			{
				name: 'zai-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'openrouter-api-key',
		label: 'OpenRouter',
		description: 'Access multiple LLM providers through one API',
		fields: [
			{
				name: 'token',
				label: 'API Key',
				secret: true,
				placeholder: 'sk-or-...',
				required: true
			}
		]
	},
	{
		id: 'custom-api-key',
		label: 'Custom / OpenAI-compatible',
		description: 'Any OpenAI-compatible API endpoint',
		fields: [
			{
				name: 'custom-base-url',
				label: 'Base URL',
				secret: false,
				placeholder: 'https://api.example.com/v1',
				required: true
			},
			{
				name: 'custom-model-id',
				label: 'Model ID',
				secret: false,
				placeholder: 'gpt-4',
				required: true
			},
			{
				name: 'custom-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			},
			{
				name: 'custom-provider-id',
				label: 'Provider ID',
				secret: false,
				placeholder: 'my-provider',
				required: true
			},
			{
				name: 'custom-compatibility',
				label: 'Compatibility Mode',
				secret: false,
				placeholder: 'openai',
				required: false,
				defaultValue: 'openai'
			}
		]
	}
];

/**
 * Get a provider configuration by its ID.
 */
export function getProviderById(id: string): ProviderConfig | undefined {
	return PROVIDERS.find((p) => p.id === id);
}

/**
 * Get the default provider (first in the list).
 */
export function getDefaultProvider(): ProviderConfig {
	return PROVIDERS[0];
}
