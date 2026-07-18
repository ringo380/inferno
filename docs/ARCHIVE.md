# Archived Features

This document records features that have been removed from Inferno, why they were
removed, and where equivalent functionality now lives. It exists so that anyone
searching the history for a removed capability can understand the decision without
digging through commits.

## Web admin dashboard (`inferno dashboard`)

**Removed:** 2026-07 (issue #44)

**What it was:** An axum-based web server (`src/dashboard.rs`) exposing an admin
dashboard, driven by the `inferno dashboard` CLI subcommand (`src/cli/dashboard.rs`)
and a `dashboard` section in the main configuration.

**Why it was archived:**

- The only consumer was the `inferno dashboard` CLI command. Nothing in the
  library or the desktop app ever instantiated the server.
- The endpoints served mock/placeholder data rather than live system state, so the
  dashboard never provided a working operational view.
- The desktop application ships its own Next.js frontend (`dashboard/src-tauri/`),
  which is the supported graphical interface on macOS and does not use this server.

Maintaining a second, non-functional web frontend added compile time and surface
area with no user-facing benefit, so it was removed in favor of the desktop app.

**Where functionality lives now:**

- Graphical interface: the Tauri v2 desktop app under `dashboard/src-tauri/`.
- Metrics and observability: `inferno metrics`, `inferno observability`, and the
  HTTP API (`inferno serve`).

**Related cleanup:** the dashboard integration-test suites (`dashboard_api_tests`,
`dashboard_api_workflow_tests`) were removed in #54; the module itself, its CLI
command, and its configuration field were removed in the follow-up.

## `logging-audit` CLI (`inferno logging-audit`)

**Removed:** 2026-07 (issue #44)

**What it was:** A ~3,700-line CLI command surface (`src/cli/logging_audit.rs`)
plus a `logging_audit` module of supporting types, providing log/audit export,
compliance reporting, and integrity checks.

**Why it was archived:**

- The command duplicated `inferno audit`, which is the maintained audit
  interface. The two overlapped heavily in purpose (audit trail, compliance,
  export) with no clear division of responsibility.
- Parts of it were unfinished - e.g. CSV export returned the literal string
  `"CSV export not implemented"` rather than producing output.

**What was kept:** the small set of types the rest of the codebase actually
depends on stays in `src/logging_audit.rs`:

- `LoggingAuditConfig` / `AuditConfig` - referenced by the main `Config`.
- `ComplianceReport`, `ComplianceStandard`, `ComplianceFinding`,
  `IntegrityReport`, `IntegrityStatus` - constructed by `crate::audit`
  (`validate_audit_integrity` and compliance reporting).

The CLI-only types (`LogEntry`, `ExportRequest`, `DateRange`, `ActionOutcome`,
`ActorFilter`, `ResourceFilter`, `AnomalyAlert`, `ExportStatus`, and the audit
type re-exports) were removed with the command.

**Where functionality lives now:** `inferno audit` (audit trail, querying,
compliance, export).

**What would have to be true to want it back:** a concrete need for an audit CLI
surface distinct from `inferno audit` - at which point it should extend that
command rather than reintroduce a parallel one.

## `advanced-cache` module and CLI (`inferno advanced-cache`)

**Removed:** 2026-07 (issue #44)

**What it was:** A ~5,100-line module (`src/advanced_cache.rs`) plus CLI
(`src/cli/advanced_cache.rs`) modelling a multi-tier cache (L1/L2/L3 hierarchy,
memory management, prefetching, compression, persistence, distributed topology,
tiering) with an `AdvancedCacheSystem` and an `inferno advanced-cache` command.

**Why it was archived:**

- The entire system ran on mock backends: `AdvancedCacheSystem` was constructed
  with `MockCacheBackend`, `MockCacheMonitor`, `MockCacheOptimizer`, and
  `MockCompressionEngine`, and the CLI emitted demo output ("Mock listing", "For
  CLI demo purposes, show mock optimization results", "Mock restore"). Per the
  #44 rule, fabricated output is a delete signal, not a keep signal.
- Its only consumer was its own `inferno advanced-cache` command. Nothing in the
  runtime/inference path used it.
- The vast majority of the module was configuration types; the `config`
  `advanced_cache` field was never read by anything (even the CLI built its own
  `AdvancedCacheConfig::default()`), so it was removed too.

**Where functionality lives now:** the real, used cache is `crate::cache`
(`ModelCache`, model loading + warm-up) and `crate::response_cache` (response
deduplication), both re-exported from `infrastructure::cache`.

**What would have to be true to want it back:** a concrete need for a real
multi-tier cache, at which point it should be built against real backends and
wired into the inference path - not restored as mock scaffolding.

## `advanced-monitoring` module and CLI (`inferno advanced-monitoring`)

**Removed:** 2026-07 (issue #44)

**What it was:** A ~6,000-line module (`src/advanced_monitoring.rs`) plus CLI
(`src/cli/advanced_monitoring.rs`) modelling advanced monitoring and alerting with
"Prometheus integration" - metric collection, alert rules, notification channels,
metric exporters, and an `inferno advanced-monitoring` command.

**Why it was archived:**

- The entire system fabricated its output: `collect_system_metrics` returned a
  hardcoded 45.2% CPU / 2 GB memory, `collect_application_metrics` returned a
  hardcoded 1250 requests for "llama-7b", `evaluate_alert_rule` always returned
  `false`, Prometheus queries were mocked, and the `FileExporter`/`HttpExporter`
  `export()` methods were no-ops. Per the #44 rule, fabricated output is a delete
  signal, not a keep signal.
- Its only references were its own `inferno advanced-monitoring` command and a
  `pub use` re-export. Nothing in the library, desktop app, or HTTP server used it.
- The `config` `advanced_monitoring` field was never read by anything, so it was
  removed too.
- It overlapped the real `crate::monitoring` module (alerting, thresholds,
  `PrometheusConfig`), which is smaller but actually wired into live callers
  (`response_cache`, `distributed`, `cli/context`).

**Where functionality lives now:** the used monitoring/metrics surface is
`crate::monitoring` (re-exported from `infrastructure::monitoring`), `inferno
metrics`, `inferno observability`, and the HTTP API (`inferno serve`).

**What would have to be true to want it back:** a real Prometheus/OpenTelemetry
integration that scrapes live system state (`sysinfo`) and wires exporters to real
data - a from-scratch build against the real monitoring path, not a revival of this
mock scaffold.

## Model marketplace and package manager (`inferno marketplace` / `package` / `install` / `remove` / `search` / `list` / `repo`)

**Removed:** 2026-07 (issue #44)

**What it was:** A ~6,600-line cluster implementing a model marketplace and an
apt-style package manager:

- `src/marketplace.rs` + `src/cli/marketplace.rs` - the `ModelMarketplace` engine
  and the `inferno marketplace` command.
- `src/cli/package.rs` - the `inferno package` command plus the simplified
  aliases `inferno install` / `remove` / `search` / `list`.
- `src/cli/repo.rs` - the `inferno repo` command for managing model repositories.

`package.rs` and `repo.rs` were built entirely on `ModelMarketplace`, so the three
were a single unit and were removed together.

**Why it was archived:**

- The feature was aspirational. Although the config pointed at real registries
  (HuggingFace, Ollama, ONNX, PyTorch, TF Hub) and some paths used real HTTP, the
  core repository fetch was an explicit placeholder (`fetch_repository_models`:
  "TODO: Implement actual HTTP client to fetch models from repository ... This is a
  placeholder implementation"), so end-to-end install/search never worked against a
  real registry.
- The `config.marketplace` field was never read by anything.
- There is no operating Inferno model registry to back it, so the whole
  package-manager UX advertised a workflow that could not complete.

**Where functionality lives now:** model discovery and use go through the real
paths - `inferno models` (list/inspect local models), `inferno run --model <path>`,
and the models directory (`INFERNO_MODELS_DIR`). There is no supported
install-from-registry flow.

**Related cleanup:** the `fuzzy` command matcher (`src/cli/fuzzy.rs`) had its
vocabulary and aliases pruned so it no longer suggests the removed commands, and the
"Common commands" hint in the enhanced parser was updated to point at surviving
commands.

**Known follow-up:** the help/onboarding subsystem (`src/cli/help.rs` and the
example/prerequisite helpers) still contains guidance and examples written around the
package-manager workflow (e.g. suggesting `inferno install ...` as a remedy for a
missing model). Scrubbing it requires deciding the replacement onboarding narrative
and is tracked separately; it does not affect compilation.

**What would have to be true to want it back:** a real, operating model registry
(first-party or a concrete third-party API) plus a real download/verify/install
implementation wired into the models directory - not a revival of the placeholder
fetch.

## `backup-recovery` module and CLI (`inferno backup-recovery`)

**Removed:** 2026-07 (issue #44)

**What it was:** A ~5,970-line module (`src/backup_recovery.rs`) plus CLI
(`src/cli/backup_recovery.rs`) modelling enterprise backup and disaster recovery -
scheduled backups, multi-destination storage, encryption, and verification, with an
`inferno backup-recovery` command.

**Why it was archived:**

- Every backend was a mock, under a section literally headed "Implementation structs
  (mock implementations for compilation)". `upload_backup` returned a random UUID and
  stored nothing; `download_backup` was a no-op that restored nothing;
  `verify_backup` always returned `true`; `encrypt_data`/`decrypt_data` returned the
  data unchanged (no encryption); the scheduler's `schedule_backup` was a no-op.
- A backup/DR feature that silently backs up nothing, restores nothing, and encrypts
  nothing is worse than absent - it invites false confidence. Per the #44 rule,
  fabricated output is a delete signal.
- Its only references were its own CLI and a `pub use` re-export in
  `operations::mod`. Nothing in the library, desktop app, or HTTP server used it, and
  the `config.backup_recovery` field was never read.

**Where functionality lives now:** nothing in-tree replaces it; there is no supported
backup/DR feature. Model files live in the models directory
(`INFERNO_MODELS_DIR`) and can be backed up with standard filesystem tooling.

**What would have to be true to want it back:** a real implementation - actual
storage-destination I/O (local/S3/etc.), real encryption via a vetted crate, and a
restore path verified end-to-end - not a revival of the mock scaffold.
