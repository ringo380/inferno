# Archived Features

This document records features that have been removed from Inferno, why they were
removed, and where equivalent functionality now lives. It exists so that anyone
searching the history for a removed capability can understand the decision without
digging through commits.

Archiving (as opposed to deleting) means the code is taken off the maintenance path
but kept **recoverable**. "It's in git history somewhere" is not a salvage path; a
tag plus this index is. See #40 (the reduction epic) and #41 (this mechanism).

## Recovering archived code

Every file listed below is preserved at a single immutable tag pointing at the last
commit before the reduction began:

```
archive/enterprise-surface-v0.10.7   ->   f2eaf12   (parent of the first removal, #54)
```

The tree at that tag still contains every archived module in full. To restore a file,
use the runnable command in its entry, which is always of the form:

```
git show archive/enterprise-surface-v0.10.7:<path> > <path>
```

For a module that was only **partially** removed (a salvage - e.g. `deployment`), the
file still exists on `main`; recover the removed portion by diffing against the tag
rather than overwriting:

```
git diff archive/enterprise-surface-v0.10.7 -- <path>
```

**The tag is immutable - never move or delete it.** On a public repo GitHub keeps
tagged commits reachable indefinitely, which is exactly the durability wanted here.

## Entry format

Each entry is written **at removal time, in the same PR as the removal**, and states:

- what was removed, and its size
- **Recover:** the runnable `git show ...` command(s) for the removed path(s)
- why it was removed (no consumer / duplicated by X / fabricated output)
- what was kept or promoted out of it before removal, and where that landed
- what would have to be true to want it back

## Salvage rules (from #40)

Order of preference for any candidate, in the same PR:

1. **Promote** - if a piece is genuinely useful to the core product, move it into core
   in its own commit *before* the removal commit, so the removal stays a pure subtraction.
2. **Archive** - coherent work with no current consumer: remove it from the build and add
   an entry here (tag + recovery command).
3. **Delete** - only for code that is dead, duplicated, or fabricated output.

**No archival PR merges without an entry in this file.** A removal that isn't indexed
here is functionally a deletion, which defeats the purpose.

### Promotions (kept in core, not archived)

- **`resilience` -> `RetryPolicy`** was promoted into core and wired into the
  `inferno models search` HuggingFace fetch (#59), with the module's first real unit
  tests. The rest of the module was unused; only the promoted piece survives, so there
  is no archive entry for it.
- **`distributed`** was evaluated and **kept** (not archived): it is wired into
  `serve --distributed` and the `src/api/openai.rs` worker-pool path.

## Web admin dashboard (`inferno dashboard`)

**Removed:** 2026-07 (issue #44)

**What it was:** An axum-based web server (`src/dashboard.rs`) exposing an admin
dashboard, driven by the `inferno dashboard` CLI subcommand (`src/cli/dashboard.rs`)
and a `dashboard` section in the main configuration.

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/dashboard.rs > src/dashboard.rs
git show archive/enterprise-surface-v0.10.7:src/cli/dashboard.rs > src/cli/dashboard.rs
```

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

**Recover:** (the shared types in `src/logging_audit.rs` were kept; only the CLI was removed)

```
git show archive/enterprise-surface-v0.10.7:src/cli/logging_audit.rs > src/cli/logging_audit.rs
```

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

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/advanced_cache.rs > src/advanced_cache.rs
git show archive/enterprise-surface-v0.10.7:src/cli/advanced_cache.rs > src/cli/advanced_cache.rs
```

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

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/advanced_monitoring.rs > src/advanced_monitoring.rs
git show archive/enterprise-surface-v0.10.7:src/cli/advanced_monitoring.rs > src/cli/advanced_monitoring.rs
```

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

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/marketplace.rs > src/marketplace.rs
git show archive/enterprise-surface-v0.10.7:src/cli/marketplace.rs > src/cli/marketplace.rs
git show archive/enterprise-surface-v0.10.7:src/cli/package.rs > src/cli/package.rs
git show archive/enterprise-surface-v0.10.7:src/cli/repo.rs > src/cli/repo.rs
```

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

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/backup_recovery.rs > src/backup_recovery.rs
git show archive/enterprise-surface-v0.10.7:src/cli/backup_recovery.rs > src/cli/backup_recovery.rs
```

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

---

## `multimodal` (removed 2026-07-18, issue #44)

**What it was:** a "multi-modal inference" subsystem (`src/multimodal.rs`, 1,244 lines)
and its CLI (`src/cli/multimodal.rs`, 1,572 lines) exposing an `inferno multimodal`
command with `process`/`batch`/`analyze`/`convert`/`capabilities` subcommands for
vision, audio, video, and mixed-media input.

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/multimodal.rs > src/multimodal.rs
git show archive/enterprise-surface-v0.10.7:src/cli/multimodal.rs > src/cli/multimodal.rs
```

**Why archived:**
- Every result was fabricated. `perform_multimodal_inference` returned hardcoded
  strings independent of the input - an image "analysis" always reported
  `"Detected objects ... cars, buildings, people"` and an audio "transcription" always
  returned `"Hello, this is a test recording"`. It never loaded a model or called an
  `InferenceBackend`. Image, audio, video, and metadata handling were all marked
  `// Mock`, and the CLI `convert` read the input file and wrote the same bytes back
  after a fake 500ms sleep.
- The feature is aspirational for a text-only GGUF/ONNX runner: there is no
  vision/audio-capable backend anywhere in the project to power it.
- It was self-contained. Its only references were its own CLI (dispatched from
  `main.rs`) and a `pub use crate::multimodal` re-export in `ai_features::mod`. There
  was no `config.multimodal` field, and nothing outside the module consumed its types
  (the `AudioFeatures` in `src/io/mod.rs` is a separate, real struct - a name
  collision, not `crate::multimodal::AudioFeatures`).
- Per the #44 rule, fabricated output is a delete signal.

**What was kept:** nothing from the module. The genuine media code it pretended to use
already lives elsewhere and is untouched: the `image` crate in `src/io/mod.rs` and
`src/icon_generator.rs`, and the `hound` crate (real audio feature extraction) in
`src/io/mod.rs`.

**Where functionality lives now:** nothing in-tree replaces it; Inferno remains a
text-only model runner.

**What would have to be true to want it back:** a real vision/audio-capable backend
(for example a llava or whisper GGUF path wired to `InferenceBackend`), at which point
the pipeline would be built fresh against that backend rather than revived from this
mock scaffold.

---

## `performance_optimization` (removed 2026-07-18, issue #44)

**What it was:** an "enterprise performance optimization and auto-tuning" subsystem
(`src/performance_optimization.rs`, 3,226 lines) and its CLI
(`src/cli/performance_optimization.rs`, 3,048 lines) exposing an
`inferno performance-optimization` command for profiling, optimization, auto-tuning,
resource management, and ML-driven tuning. At 6,274 lines it was the largest module
removed under #44.

**Recover:**

```
git show archive/enterprise-surface-v0.10.7:src/performance_optimization.rs > src/performance_optimization.rs
git show archive/enterprise-surface-v0.10.7:src/cli/performance_optimization.rs > src/cli/performance_optimization.rs
```

**Why archived:**
- Every result was fabricated. All seven trait implementations (`Profiler`,
  `Optimizer`, `AutoTuner`, `ResourceManager`, `CacheManager`, `PerformanceMonitoring`,
  `MlEngine`) lived under a section literally headed `// Mock implementations`.
  `SystemProfiler` returned a hardcoded 45% CPU profile with fixed memory/IO/network
  numbers; `SystemOptimizer.optimize` reported a fixed 20% latency / 30% throughput
  gain and `$500` cost savings while applying nothing; `apply_optimization`,
  `rollback_optimization`, `scale_resources`, and `apply_tuning` were no-ops;
  `MlEngine.predict` and `AutoTuner.evaluate_tuning` both returned a constant `0.85`;
  `CacheManager.get` always returned `None`. Nothing read a real system counter or ran
  a real optimization.
- It was self-contained. Its only references were its own CLI (dispatched from
  `main.rs`) and a `pub use crate::performance_optimization` re-export in
  `ai_features::optimization::mod`. The `config.performance_optimization` field was
  declared and defaulted but never read - the CLI built its own
  `PerformanceOptimizationConfig::default()` and ignored it. Nothing outside the module
  consumed its types (the `PerformanceProfile` in
  `src/infrastructure/sys_monitor.rs` is a separate, real enum - a name collision, not
  this module's `PerformanceProfile` struct).
- Per the #44 rule, fabricated output is a delete signal.

**What was kept:** nothing from the module. The real optimization surface is untouched:
`inferno optimization` (quantization/pruning/distillation) in `optimization.rs` +
`cli/optimization.rs`, and genuine system metrics in
`src/infrastructure/sys_monitor.rs` and the `monitoring` module. The dead
`config.performance_optimization` field was also removed; because serde ignores
unknown fields on deserialize, existing `.inferno.toml` files that still carry a
`[performance_optimization]` section continue to load unchanged.

**Where functionality lives now:** nothing in-tree replaces the auto-tuning feature;
real model optimization is available through `inferno optimization`.

**What would have to be true to want it back:** real profiling and tuning wired to
actual system counters and a genuine optimizer/ML engine - built fresh against those,
not revived from this mock scaffold.

---

## `deployment` mock cluster operations (partial salvage, issue #44)

**Removed:** 2026-07-18 (#64). This is a **salvage, not a full archive**: the module
(`src/deployment.rs`) and its CLI (`src/cli/deployment.rs`) still exist on `main` -
only the fabricated cluster-apply half was removed.

**What was kept (real):** manifest and Helm-chart **generation**. `create_deployment_manifest`
/ `_service` / `_configmap` / `_hpa` emit valid Kubernetes YAML, and `generate_helm_chart`
writes a real `Chart.yaml`, `values.yaml`, and templates. The surviving command is
`inferno deployment generate`. The `DeploymentConfig` type is read from config, and
`DeploymentStatus` / `DeploymentState` are consumed by `model_versioning`, so those
type definitions were retained.

**What was removed (mock):** the cluster-**apply** path. `execute_deployment` only slept
and logged; the Kubernetes/Helm connection code was a stub that connected to nothing; and
`deploy` / `rollback` / `scale` / `status` all reported success while touching no cluster.
The module + CLI dropped from 4,414 to 1,945 lines (module 2,695 -> 1,807, CLI 1,719 -> 138).

**Recover:** the files still exist on `main`, so recover the removed mock apply path by
diffing the current file against the pre-salvage tag rather than overwriting:

```
git diff archive/enterprise-surface-v0.10.7 -- src/deployment.rs src/cli/deployment.rs
git show archive/enterprise-surface-v0.10.7:src/deployment.rs   # full pre-salvage module
```

**What would have to be true to want it back:** a real cluster client (a vetted
`kube`/Helm integration) that actually applies manifests and reports true status - at
which point the apply path is built fresh against that client, not revived from the
sleep-and-log stub. The generation half stands on its own and needs nothing.
