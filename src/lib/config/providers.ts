/**
 * Provider configuration for OutClaw wizard
 * Maps provider auth choices to their display info and required fields
 */

export interface ProviderField {
	name: string; // e.g., "anthropic-api-key"
	label: string; // e.g., "API Key"
	secret: boolean; // whether to mask the input
	placeholder?: string;
	required?: boolean;
	defaultValue?: string;
}

export interface ProviderConfig {
	id: string; // matches --auth-choice value
	label: string; // display name
	description: string; // short description
	fields: ProviderField[];
}

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
		id: 'zai-api-key',
		label: 'Z.ai',
		description: 'Z.ai AI models',
		fields: [
			{
				name: 'zai-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'Enter your Z.ai API key',
				required: true
			}
		]
	},
	{
		id: 'gemini-api-key',
		label: 'Google Gemini',
		description: 'Google Gemini models',
		fields: [
			{
				name: 'gemini-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'Enter your Google AI API key',
				required: true
			}
		]
	},
	{
		id: 'mistral-api-key',
		label: 'Mistral',
		description: 'Mistral AI models',
		fields: [
			{
				name: 'mistral-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'Enter your Mistral API key',
				required: true
			}
		]
	},
	{
		id: 'openai-api-key',
		label: 'OpenAI',
		description: 'GPT and other OpenAI models',
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
		id: 'openrouter-api-key',
		label: 'OpenRouter',
		description: 'Access multiple AI providers through one API',
		fields: [
			{
				name: 'token',
				label: 'API Token',
				secret: true,
				placeholder: 'sk-or-...',
				required: true
			}
		]
	},
	{
		id: 'custom-api-key',
		label: 'Custom/OpenAI-compatible',
		description: 'Connect to any OpenAI-compatible API',
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
				placeholder: 'gpt-4 or custom-model',
				required: true
			},
			{
				name: 'custom-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'Enter your API key',
				required: true
			},
			{
				name: 'custom-provider-id',
				label: 'Provider ID',
				secret: false,
				placeholder: 'my-custom-provider',
				required: false
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
 * Get a provider config by ID
 */
export function getProviderById(id: string): ProviderConfig | undefined {
	return PROVIDERS.find((p) => p.id === id);
}

/**
 * Get the default provider (Anthropic)
 */
export function getDefaultProvider(): ProviderConfig {
	return PROVIDERS[0];
}
