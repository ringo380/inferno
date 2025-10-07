# Repository Guidelines

## Project Structure & Module Organization
Inferno's Rust core lives in `src/`, with modules for backends, API, CLI, and TUI. Tests sit in `tests/` (fixtures under `tests/common`), and Criterion suites live in `benches/`. The Next.js + Tauri dashboard is under `dashboard/` (`src/` for the web UI, `src-tauri/` for desktop bindings). Docs and planning files stay in `docs/`, `plans/`, and `examples/`; automation scripts—including `verify.sh`—reside in `scripts/`.

## Build, Test, and Development Commands
- `cargo build` / `cargo run --bin inferno`: build or run the Rust CLI locally.
- `cargo test` and `cargo test --test integration_tests`: execute unit plus targeted integration suites.
- `cargo fmt`, `cargo clippy --all-targets --all-features`: enforce formatting and lint gates.
- `./verify.sh`: full preflight (fmt, clippy, builds, tests, audit hooks).
- `npm run dev` (port 3457) and `npm run build`: iterate or bundle the dashboard; pair with `npm run tauri:dev` for desktop work.
- `npm run test`, `npm run test:e2e`, `npm run lint`: Jest, Playwright, and ESLint checks for dashboard code.

## Coding Style & Naming Conventions
Rust code follows default `rustfmt` rules (4-space indent) with snake_case modules, CamelCase types, and SCREAMING_SNAKE_CASE consts. Use `anyhow::Result` for CLI flows and `thiserror` enums at library boundaries. TypeScript/React files keep PascalCase components, camelCase hooks and utilities, and shared primitives in `dashboard/src/components/ui/`. Run `npm run type-check` ahead of PRs that touch TS types.

## Testing Guidelines
Place unit tests alongside the relevant module and stage broader flows in `tests/` (mirror names like `*_integration_tests.rs`). Performance runs belong in `benches/` using Criterion. Dashboard contributors should colocate component tests with each feature folder and keep Playwright specs under `dashboard/src/__tests__/e2e/`. Target ≥80% coverage for core runtime changes and call out new entry points in PRs.

## Commit & Pull Request Guidelines
Follow the conventional commit style seen in history (`feat(cli): ...`, `fix(cache): ...`, `refactor: ...`) and squash work-in-progress commits. Each PR should link its issue, describe behavior changes, and flag rollout risks. Run `./verify.sh` and dashboard checks (`npm run lint && npm run test`) before requesting review, attach logs or screenshots for UI changes, and highlight migrations or model format shifts.

## Security & Configuration Tips
Keep secrets in untracked `.env` files (e.g., `dashboard/.env.local`) and avoid committing API tokens or model binaries; stash samples in `models/`. Request a focused review on crypto or auth updates under `src/security` or `dashboard/src/lib/auth`. Before packaging releases, prune build artifacts such as `target/`, `node_modules/`, and generated bundles.
