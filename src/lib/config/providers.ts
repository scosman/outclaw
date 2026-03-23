/**
 * Provider configuration for the wizard connection form.
 * Each provider defines the fields needed to connect via the OpenClaw CLI.
 * 
 * To update run `openclaw onboard --help` to see the available providers and fields, and use an agent to update this file.
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
	// ── Major Cloud Providers ──────────────────────────────────────────
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
		id: 'xai-api-key',
		label: 'xAI',
		description: 'Grok and other xAI models',
		fields: [
			{
				name: 'xai-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'xai-...',
				required: true
			}
		]
	},

	// ── Aggregators / Routers ──────────────────────────────────────────
	{
		id: 'openrouter-api-key',
		label: 'OpenRouter',
		description: 'Access multiple LLM providers through one API',
		fields: [
			{
				name: 'openrouter-api-key',
				label: 'API Key',
				secret: true,
				placeholder: 'sk-or-...',
				required: true
			}
		]
	},
	{
		id: 'together-api-key',
		label: 'Together AI',
		description: 'Open-source models hosted by Together',
		fields: [
			{
				name: 'together-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'chutes-api-key',
		label: 'Chutes',
		description: 'Chutes model hosting',
		fields: [
			{
				name: 'chutes-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'litellm-api-key',
		label: 'LiteLLM',
		description: 'LiteLLM proxy for unified LLM access',
		fields: [
			{
				name: 'litellm-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'huggingface-api-key',
		label: 'Hugging Face',
		description: 'Hugging Face Inference API',
		fields: [
			{
				name: 'huggingface-api-key',
				label: 'API Key (HF Token)',
				secret: true,
				placeholder: 'hf_...',
				required: true
			}
		]
	},

	// ── Gateway / Proxy Providers ──────────────────────────────────────
	{
		id: 'ai-gateway-api-key',
		label: 'Vercel AI Gateway',
		description: 'Vercel AI Gateway API',
		fields: [
			{
				name: 'ai-gateway-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'cloudflare-ai-gateway-api-key',
		label: 'Cloudflare AI Gateway',
		description: 'Cloudflare AI Gateway with Workers AI',
		fields: [
			{
				name: 'cloudflare-ai-gateway-account-id',
				label: 'Account ID',
				secret: false,
				placeholder: '...',
				required: true
			},
			{
				name: 'cloudflare-ai-gateway-gateway-id',
				label: 'Gateway ID',
				secret: false,
				placeholder: '...',
				required: true
			},
			{
				name: 'cloudflare-ai-gateway-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'kilocode-api-key',
		label: 'Kilo Gateway',
		description: 'Kilo Gateway API',
		fields: [
			{
				name: 'kilocode-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'github-copilot',
		label: 'GitHub Copilot',
		description: 'Use existing GitHub Copilot subscription',
		fields: []
	},

	// ── Z.AI Variants ──────────────────────────────────────────────────
	{
		id: 'zai-api-key',
		label: 'Z.AI',
		description: 'Z.AI API models',
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
		id: 'zai-global',
		label: 'Z.AI (Global)',
		description: 'Z.AI global endpoint',
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
		id: 'zai-cn',
		label: 'Z.AI (China)',
		description: 'Z.AI China endpoint',
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
		id: 'zai-coding-global',
		label: 'Z.AI Coding (Global)',
		description: 'Z.AI Coding global endpoint',
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
		id: 'zai-coding-cn',
		label: 'Z.AI Coding (China)',
		description: 'Z.AI Coding China endpoint',
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

	// ── Chinese / Asia-Pacific Providers ───────────────────────────────
	{
		id: 'modelstudio-api-key',
		label: 'Alibaba Model Studio (Global)',
		description: 'Alibaba Cloud Model Studio Coding Plan (International)',
		fields: [
			{
				name: 'modelstudio-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'modelstudio-api-key-cn',
		label: 'Alibaba Model Studio (China)',
		description: 'Alibaba Cloud Model Studio Coding Plan (China)',
		fields: [
			{
				name: 'modelstudio-api-key-cn',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'qwen-portal',
		label: 'Qwen Portal',
		description: 'Qwen models via Alibaba Cloud',
		fields: [
			{
				name: 'modelstudio-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'qianfan-api-key',
		label: 'Baidu QIANFAN',
		description: 'Baidu QIANFAN platform models',
		fields: [
			{
				name: 'qianfan-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'moonshot-api-key',
		label: 'Moonshot',
		description: 'Moonshot AI models',
		fields: [
			{
				name: 'moonshot-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'moonshot-api-key-cn',
		label: 'Moonshot (China)',
		description: 'Moonshot AI models (China endpoint)',
		fields: [
			{
				name: 'moonshot-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'kimi-code-api-key',
		label: 'Kimi Code',
		description: 'Kimi Code AI assistant',
		fields: [
			{
				name: 'kimi-code-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'minimax-global-api',
		label: 'MiniMax (Global API)',
		description: 'MiniMax models via API (International)',
		fields: [
			{
				name: 'minimax-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'minimax-cn-api',
		label: 'MiniMax (China API)',
		description: 'MiniMax models via API (China)',
		fields: [
			{
				name: 'minimax-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'minimax-global-oauth',
		label: 'MiniMax (Global OAuth)',
		description: 'MiniMax models via OAuth (International)',
		fields: []
	},
	{
		id: 'minimax-cn-oauth',
		label: 'MiniMax (China OAuth)',
		description: 'MiniMax models via OAuth (China)',
		fields: []
	},
	{
		id: 'byteplus-api-key',
		label: 'BytePlus',
		description: 'BytePlus (ByteDance international) models',
		fields: [
			{
				name: 'byteplus-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'volcengine-api-key',
		label: 'Volcano Engine',
		description: 'Volcano Engine (ByteDance) models',
		fields: [
			{
				name: 'volcengine-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'xiaomi-api-key',
		label: 'Xiaomi',
		description: 'Xiaomi AI models',
		fields: [
			{
				name: 'xiaomi-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},

	// ── Other API Providers ────────────────────────────────────────────
	{
		id: 'venice-api-key',
		label: 'Venice',
		description: 'Venice AI privacy-focused models',
		fields: [
			{
				name: 'venice-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'synthetic-api-key',
		label: 'Synthetic',
		description: 'Synthetic AI models',
		fields: [
			{
				name: 'synthetic-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},

	// ── CLI Tools (use locally installed CLI auth) ─────────────────────
	{
		id: 'claude-cli',
		label: 'Claude CLI',
		description: 'Use locally installed Claude CLI credentials',
		fields: []
	},
	{
		id: 'codex-cli',
		label: 'Codex CLI',
		description: 'Use locally installed Codex CLI credentials',
		fields: []
	},
	{
		id: 'openai-codex',
		label: 'OpenAI Codex CLI',
		description: 'Use locally installed OpenAI Codex credentials',
		fields: []
	},
	{
		id: 'google-gemini-cli',
		label: 'Google Gemini CLI',
		description: 'Use locally installed Google Gemini CLI credentials',
		fields: []
	},
	{
		id: 'opencode-zen',
		label: 'OpenCode (Zen)',
		description: 'OpenCode Zen catalog',
		fields: [
			{
				name: 'opencode-zen-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},
	{
		id: 'opencode-go',
		label: 'OpenCode (Go)',
		description: 'OpenCode Go catalog',
		fields: [
			{
				name: 'opencode-go-api-key',
				label: 'API Key',
				secret: true,
				placeholder: '...',
				required: true
			}
		]
	},

	// ── Custom / Advanced ──────────────────────────────────────────────
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
				required: false
			},
			{
				name: 'custom-compatibility',
				label: 'Compatibility Mode',
				secret: false,
				placeholder: 'openai or anthropic',
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
