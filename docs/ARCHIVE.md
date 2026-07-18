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
