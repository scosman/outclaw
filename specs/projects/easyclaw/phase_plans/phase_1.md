---
status: complete
---

# Phase 1: Project Scaffolding, Theming, and CI

## Overview

Initialize the Tauri v2 + SvelteKit project with all tooling, theming, and CI configured. This phase creates the foundation that all subsequent phases build upon.

## Steps

### 1. Initialize Tauri v2 Project with SvelteKit

```bash
npm create tauri-app@latest easyclaw-temp -- --template svelte-ts
```

Then copy the generated files to the project root, or manually set up:

- `src-tauri/` directory with Tauri v2 configuration
- `src/` directory for SvelteKit frontend
- `package.json` with Tauri dependencies

### 2. Configure SvelteKit with Static Adapter

- Install `@sveltejs/adapter-static`
- Update `svelte.config.js` to use static adapter with SSR disabled
- Create root `+layout.ts` with `export const prerender = true` and `export const ssr = false`

### 3. Install and Configure Tailwind CSS + shadcn-svelte

- Install `tailwindcss`, `postcss`, `autoprefixer`
- Install shadcn-svelte prerequisites: `bits-ui`, `clsx`, `tailwind-merge`, `tailwind-variants`
- Initialize Tailwind with `npx tailwindcss init -p`
- Configure `tailwind.config.ts` with dark mode and content paths
- Install shadcn-svelte components via CLI
- Set dark theme as default in root layout

### 4. Add JetBrains Mono Font

- Download JetBrains Mono font files (woff2 format)
- Place in `static/fonts/`
- Add `@font-face` declarations in global CSS
- Set as global monospace font in Tailwind config

### 5. Configure ESLint + Prettier + svelte-check

- Install ESLint with Svelte plugin: `eslint`, `eslint-plugin-svelte`, `@typescript-eslint/eslint-plugin`
- Install Prettier with Svelte plugin: `prettier`, `prettier-plugin-svelte`
- Create `.eslintrc.cjs` and `.prettierrc` configs
- Add npm scripts: `lint`, `format`, `format:check`, `check`

### 6. Configure Rust Tooling

- Add `rustfmt.toml` or use defaults
- Ensure `clippy` is configured via `Cargo.toml` or `.clippy.toml`
- Add cargo scripts if needed via `package.json`

### 7. Create GitHub Actions CI Workflow

- Create `.github/workflows/ci.yml`
- Jobs: `check` (frontend) and `rust` (backend)
- Frontend: npm ci, check, lint, format:check, test
- Rust: cargo fmt --check, cargo clippy, cargo test

### 8. Create Basic App Layout

- Update `src-tauri/tauri.conf.json`:
  - Window size: 900x640 default, 720x500 minimum
  - Title: "EasyClaw"
- Create root `+layout.svelte` with header bar containing "EasyClaw" text
- Create placeholder `+page.svelte` with content area

### 9. Verify and Test

- Run `npm run tauri dev` to verify app launches
- Verify dark theme is applied
- Verify monospace font is active
- Run all linting and formatting checks

## Completion Criteria

- [ ] `npm run tauri dev` launches app with dark theme
- [ ] App window is 900x640 with 720x500 minimum
- [ ] Header bar shows "EasyClaw" text
- [ ] JetBrains Mono font is applied
- [ ] `npm run check` passes (svelte-check + TypeScript)
- [ ] `npm run lint` passes
- [ ] `npm run format:check` passes
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes (any tests that exist)
- [ ] GitHub Actions CI workflow exists and would pass
