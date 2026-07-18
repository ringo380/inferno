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
