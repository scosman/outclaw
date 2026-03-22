# EasyClaw — Agent Guidelines

## Automated Code Checks

Run all automated checks with:
- `npm run check` (svelte-check + TypeScript)
- `npm run lint` (ESLint)
- `npm run format:check` (Prettier)
- `cargo clippy --all-targets` (Rust/Tauri backend)
- `cargo test` (Rust tests)

## Code Style

- TypeScript: strict mode, no `any` types
- Svelte: use SvelteKit conventions, shadcn-svelte components
- Rust: standard Tauri patterns
- Dark theme by default (shadcn dark mode)

## Project Context

EasyClaw is a desktop app (Tauri) for managing OpenClaw Docker instances.
See /specs/projects/easyclaw/ for full specifications.
