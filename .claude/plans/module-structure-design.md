# Inferno Module Structure Design

**Created**: 2025-09-29
**Status**: Design Phase
**Related Plan**: 2025-09-29_major-refactoring.md

## Current State (Before Refactoring)

### Root-level modules (40 modules)
```
src/
├── lib.rs
├── main.rs
├── config.rs
├── security.rs
├── backends/
├── models/
├── io/
├── cli/          (46 command files)
├── api/
├── tui/
├── dashboard.rs
├── batch.rs
├── cache.rs
├── response_cache.rs
├── advanced_cache.rs
├── monitoring.rs
├── advanced_monitoring.rs
├── observability.rs
├── metrics/
├── audit.rs
├── logging_audit.rs
├── deployment.rs
├── distributed.rs
├── multi_tenancy.rs
├── resilience.rs
├── backup_recovery.rs
├── upgrade/
├── versioning.rs
├── model_versioning.rs
├── optimization.rs
├── performance_optimization.rs
├── performance_baseline.rs
├── conversion.rs
├── multimodal.rs
├── streaming.rs
├── federated.rs
├── gpu.rs
├── marketplace.rs
├── api_gateway.rs
├── data_pipeline.rs
├── qa_framework.rs
└── tauri_app.rs
```

## New Structure (Target State)

### Proposed Organization
```
src/
├── lib.rs                    # Main library entry point
├── main.rs                   # CLI binary entry point
│
├── core/                     # Core platform functionality
│   ├── mod.rs
│   ├── config/               # Configuration system (CONSOLIDATED)
│   │   ├── mod.rs
│   │   ├── loader.rs         # Config loading logic
│   │   ├── builder.rs        # Builder pattern
│   │   ├── validation.rs     # Config validation
│   │   └── presets.rs        # Dev/prod/test presets
│   ├── backends/             # Model execution backends
│   │   ├── mod.rs
│   │   ├── gguf.rs
│   │   └── onnx.rs
│   ├── models/               # Model discovery & metadata
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   ├── validation.rs
│   │   └── metadata.rs
│   ├── io/                   # I/O format handling
│   │   ├── mod.rs
│   │   ├── text.rs
│   │   ├── image.rs
│   │   └── audio.rs
│   ├── security/             # Security & sandboxing
│   │   ├── mod.rs
│   │   ├── sandbox.rs
│   │   └── validation.rs
│   └── errors.rs             # Core error types
│
├── infrastructure/           # Infrastructure & observability
│   ├── mod.rs
│   ├── cache/                # CONSOLIDATED caching (3 modules → 1)
│   │   ├── mod.rs
│   │   ├── model_cache.rs    # Model caching
│   │   ├── response_cache.rs # Response caching
│   │   ├── advanced/         # Advanced features
│   │   │   ├── mod.rs
│   │   │   ├── hierarchy.rs
│   │   │   ├── compression.rs
│   │   │   └── persistence.rs
│   │   └── strategies.rs
│   ├── monitoring/           # CONSOLIDATED monitoring (2 modules → 1)
│   │   ├── mod.rs
│   │   ├── metrics.rs
│   │   ├── alerts.rs
│   │   ├── advanced/
│   │   │   ├── mod.rs
│   │   │   ├── apm.rs
│   │   │   └── distributed_tracing.rs
│   │   └── collectors.rs
│   ├── observability/        # Tracing & telemetry
│   │   ├── mod.rs
│   │   ├── tracing.rs
│   │   └── telemetry.rs
│   ├── metrics/              # Metrics collection
│   │   ├── mod.rs
│   │   ├── collector.rs
│   │   └── exporter.rs
│   └── audit/                # CONSOLIDATED audit (2 modules → 1)
│       ├── mod.rs
│       ├── logger.rs
│       ├── compliance.rs
│       └── retention.rs
│
├── operations/               # DevOps & operations
│   ├── mod.rs
│   ├── batch/                # Batch processing
│   │   ├── mod.rs
│   │   ├── processor.rs
│   │   ├── queue.rs
│   │   └── scheduler.rs
│   ├── deployment/           # Deployment automation
│   │   ├── mod.rs
│   │   ├── strategies.rs
│   │   └── health_checks.rs
│   ├── backup/               # Backup & recovery
│   │   ├── mod.rs
│   │   ├── backup.rs
│   │   ├── recovery.rs
│   │   └── strategies.rs
│   ├── upgrade/              # Auto-update system
│   │   ├── mod.rs
│   │   ├── checker.rs
│   │   ├── downloader.rs
│   │   └── installer.rs
│   ├── resilience/           # Resilience patterns
│   │   ├── mod.rs
│   │   ├── retry.rs
│   │   ├── circuit_breaker.rs
│   │   └── fallback.rs
│   └── versioning/           # Version management
│       ├── mod.rs
│       └── model_versions.rs
│
├── ai_features/              # AI/ML specialized features
│   ├── mod.rs
│   ├── conversion/           # Model format conversion
│   │   ├── mod.rs
│   │   ├── gguf.rs
│   │   ├── onnx.rs
│   │   └── safetensors.rs
│   ├── optimization/         # CONSOLIDATED optimization (3 → 1)
│   │   ├── mod.rs
│   │   ├── performance.rs
│   │   ├── baseline.rs
│   │   └── profiling.rs
│   ├── multimodal/           # Multimodal support
│   │   ├── mod.rs
│   │   ├── vision.rs
│   │   └── audio.rs
│   ├── streaming/            # Real-time streaming
│   │   ├── mod.rs
│   │   └── token_stream.rs
│   └── gpu/                  # GPU management
│       ├── mod.rs
│       └── device_manager.rs
│
├── enterprise/               # Enterprise features
│   ├── mod.rs
│   ├── distributed/          # Distributed inference
│   │   ├── mod.rs
│   │   ├── coordinator.rs
│   │   └── worker.rs
│   ├── multi_tenancy/        # Multi-tenant support
│   │   ├── mod.rs
│   │   ├── isolation.rs
│   │   └── resource_limits.rs
│   ├── federated/            # Federated learning
│   │   ├── mod.rs
│   │   └── aggregation.rs
│   ├── marketplace/          # Model marketplace
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   └── download.rs
│   ├── api_gateway/          # API gateway
│   │   ├── mod.rs
│   │   ├── routing.rs
│   │   └── rate_limiting.rs
│   ├── data_pipeline/        # ETL pipeline (SPLIT from 3702 lines)
│   │   ├── mod.rs
│   │   ├── ingestion/
│   │   │   ├── mod.rs
│   │   │   └── sources.rs
│   │   ├── transformation/
│   │   │   ├── mod.rs
│   │   │   └── rules.rs
│   │   ├── validation/
│   │   │   ├── mod.rs
│   │   │   └── quality.rs
│   │   └── storage/
│   │       ├── mod.rs
│   │       └── backends.rs
│   └── qa_framework/         # QA & testing
│       ├── mod.rs
│       └── validators.rs
│
├── interfaces/               # User interfaces
│   ├── mod.rs
│   ├── cli/                  # Command-line interface
│   │   ├── mod.rs
│   │   ├── commands/         # All CLI commands
│   │   │   ├── mod.rs
│   │   │   ├── run.rs
│   │   │   ├── serve.rs
│   │   │   ├── models.rs
│   │   │   ├── bench.rs
│   │   │   └── ... (46 commands organized)
│   │   ├── middleware/       # Command middleware
│   │   │   ├── mod.rs
│   │   │   ├── logging.rs
│   │   │   └── validation.rs
│   │   ├── parser.rs         # Enhanced parser
│   │   └── help.rs           # Help system
│   ├── api/                  # HTTP API
│   │   ├── mod.rs
│   │   ├── openai.rs
│   │   └── websocket.rs
│   ├── tui/                  # Terminal UI
│   │   ├── mod.rs
│   │   ├── app.rs
│   │   ├── components.rs
│   │   └── events.rs
│   ├── dashboard/            # Web dashboard (SPLIT from 3608 lines)
│   │   ├── mod.rs
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   └── api.rs
│   │   ├── ui/
│   │   │   ├── mod.rs
│   │   │   └── components.rs
│   │   └── aggregation/
│   │       ├── mod.rs
│   │       └── metrics.rs
│   └── desktop/              # Desktop app (Tauri)
│       └── mod.rs
│
└── testing/                  # Shared test utilities
    ├── mod.rs
    ├── fixtures.rs
    ├── helpers.rs
    └── mocks.rs
```

## Module Consolidation Plan

### Cache Consolidation
**Before**: `cache.rs` + `response_cache.rs` + `advanced_cache.rs`
**After**: `infrastructure/cache/` with submodules
- Reduces duplication
- Clear hierarchy (basic → advanced)
- Unified interface

### Monitoring Consolidation
**Before**: `monitoring.rs` + `advanced_monitoring.rs`
**After**: `infrastructure/monitoring/` with submodules
- Single monitoring interface
- Advanced features optional
- Better feature organization

### Audit Consolidation
**Before**: `audit.rs` + `logging_audit.rs`
**After**: `infrastructure/audit/` with submodules
- Unified audit logging
- Clear compliance features
- Consistent API

### Optimization Consolidation
**Before**: `optimization.rs` + `performance_optimization.rs` + `performance_baseline.rs`
**After**: `ai_features/optimization/` with submodules
- Single optimization namespace
- Clear feature separation
- Easier to extend

### Versioning Consolidation
**Before**: `versioning.rs` + `model_versioning.rs`
**After**: `operations/versioning/` with submodules
- Unified version management
- Clear model vs app versioning

## Benefits of New Structure

1. **Clear Boundaries**: Each top-level directory has a clear purpose
2. **Reduced Clutter**: 40+ modules → 6 main categories
3. **Better Navigation**: Related features grouped together
4. **Scalability**: Easy to add new features in the right place
5. **Testability**: Clear testing boundaries
6. **Maintainability**: Easier to understand module relationships

## Migration Strategy

### Phase 1: Create new structure
1. Create all new directories
2. Keep old files in place initially

### Phase 2: Move modules incrementally
1. Start with smallest modules
2. Update imports as we go
3. Test compilation after each move

### Phase 3: Consolidate duplicates
1. Merge cache modules
2. Merge monitoring modules
3. Merge audit modules
4. Merge optimization modules
5. Merge versioning modules

### Phase 4: Verify and cleanup
1. Run full test suite
2. Update documentation
3. Remove old empty directories

## API Stability

### Public API Preservation
To maintain backward compatibility, keep public re-exports in `lib.rs`:
```rust
// Re-export for backward compatibility
pub use crate::core::config::Config;
pub use crate::core::backends::{Backend, BackendType};
pub use crate::infrastructure::cache as cache_api;
// ... etc
```

This allows existing code to continue working while we improve internal organization.

## Next Steps

1. Review and approve this structure
2. Create directory skeleton
3. Begin moving modules
4. Update imports systematically
5. Test continuously