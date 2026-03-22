import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';

/** @type {import('eslint').Linter.Config[]} */
export default [
	js.configs.recommended,
	...ts.configs.recommended,
	...svelte.configs['flat/recommended'],
	{
		ignores: ['build/', '.svelte-kit/', 'dist/', 'node_modules/', 'src-tauri/']
	},
	{
		languageOptions: {
			parserOptions: {
				parser: ts.parser
			},
			globals: {
				process: 'readonly'
			}
		}
	}
];
