# Inferno v0.5.0: Silicon-Optimized macOS Desktop Application

**Created**: 2025-09-29
**Status**: Active
**Complexity**: Complex
**Estimated Duration**: 19-26 days

## 🎯 Project Vision

Transform Inferno from a multi-platform CLI/API tool into a **first-class macOS desktop application** optimized for Apple Silicon with Metal GPU acceleration, featuring an intuitive interface-driven experience that feels native to macOS.

## 📊 Current State Analysis (v0.4.0)

### Strengths
- ✅ Clean modular v0.4.0 architecture (6 main categories)
- ✅ Working Tauri v2 desktop app in `dashboard/src-tauri/`
- ✅ Comprehensive feature set (batch processing, monitoring, security, caching)
- ✅ Robust backend system (GGUF via llama-cpp-2, ONNX via ort)
- ✅ macOS-specific integration code exists (`src/macos_integration.rs`)
- ✅ Professional icon assets and DMG workflow

### Critical Issues
1. 🔴 **Dual Desktop Systems**: Both Tauri v1 (`src/tauri_app.rs`) and Tauri v2 (`dashboard/`) exist
2. 🔴 **Metal GPU**: Declared in features but not implemented
3. 🟡 **Module Duplication**: Old flat structure coexists with new organized structure
4. 🟡 **Empty Desktop Interface**: `src/interfaces/desktop/` is empty
5. 🟡 **Deprecated APIs**: `src/macos_integration.rs` uses Tauri v1 APIs

### Codebase Metrics
- Total modules: 40+ at root level (target: <15)
- Desktop code: Split between 2 locations
- Duplicate modules: ~12 (cache, monitoring, optimization)
- macOS integration: 336 lines using deprecated APIs

---

## Phase 1: Consolidate Desktop Architecture ⚡ PRIORITY

**Goal**: Single, unified desktop application using Tauri v2
**Duration**: 3-4 days
**Status**: 🔄 In Progress

### Task Breakdown

#### 1.1 Audit Current Functionality
- [ ] Document all commands in `src/tauri_app.rs` (Tauri v1)
- [ ] Document all commands in `dashboard/src-tauri/src/main.rs` (Tauri v2)
- [ ] Identify feature gaps between implementations
- [ ] Document macOS-specific features in use
- [ ] List all shared state and dependencies

**Key Findings**:
- Tauri v1: Uses old `tauri::command` API, deprecated menu system
- Tauri v2: Modern plugin system, full feature parity achieved
- macOS integration: Needs API version update for Tauri v2

#### 1.2 Create Unified Desktop Interface Structure
- [ ] Create `src/interfaces/desktop/mod.rs` with module exports
- [ ] Create `src/interfaces/desktop/commands.rs` (Tauri commands)
- [ ] Create `src/interfaces/desktop/state.rs` (app state management)
- [ ] Create `src/interfaces/desktop/macos.rs` (macOS-specific features)
- [ ] Create `src/interfaces/desktop/events.rs` (event system)

**Target Structure**:
```
src/interfaces/desktop/
├── mod.rs           # Module exports and main entry point
├── commands.rs      # All Tauri command handlers
├── state.rs         # AppState and shared state management
├── macos.rs         # macOS-specific features (Metal, menu, notifications)
└── events.rs        # Event emission and management system
```

#### 1.3 Migrate Functionality
- [ ] Extract common command logic from both implementations
- [ ] Port Tauri v1 commands to Tauri v2 API
- [ ] Consolidate state management patterns
- [ ] Update event emission system for Tauri v2
- [ ] Migrate macOS integration to Tauri v2 APIs

**Migration Checklist**:
- [ ] Model management commands (get_models, load_model, unload_model)
- [ ] Inference commands (infer, infer_stream)
- [ ] Metrics and monitoring commands
- [ ] Settings management commands
- [ ] Notification system commands
- [ ] Batch job management commands
- [ ] Security/API key management commands
- [ ] File dialog and system integration

#### 1.4 Update Build Configuration
- [ ] Remove `src/bin/inferno_app.rs` (redundant)
- [ ] Update `Cargo.toml` to point to `dashboard/src-tauri/`
- [ ] Configure single `inferno-app` binary target
- [ ] Set up universal binary build for ARM64 + x86_64
- [ ] Update `.cargo/config.toml` for Apple Silicon optimization
- [ ] Remove deprecated `tauri-app` feature flag

**Build Configuration**:
```toml
[[bin]]
name = "inferno-app"
path = "dashboard/src-tauri/src/main.rs"
required-features = ["desktop"]

[features]
desktop = ["tauri", "tauri-build"]
metal = ["desktop"]  # Metal GPU acceleration
```

#### 1.5 Remove Deprecated Code
- [ ] Delete `src/tauri_app.rs` (Tauri v1 implementation)
- [ ] Archive old `src/macos_integration.rs` for reference
- [ ] Remove Tauri v1 dependencies from `Cargo.toml`
- [ ] Update all imports to use new desktop module paths
- [ ] Clean up unused feature flags

### Success Criteria for Phase 1
- ✅ Single desktop binary: `inferno-app`
- ✅ Zero Tauri v1 code remaining
- ✅ All features working in Tauri v2
- ✅ Desktop code organized in `src/interfaces/desktop/`
- ✅ Clean build with no deprecated warnings

---

## Phase 2: Apple Silicon & Metal GPU Optimization 🚀

**Goal**: 3-5x faster inference on Apple Silicon via Metal acceleration
**Duration**: 5-7 days
**Status**: 📋 Planned

### Task Breakdown

#### 2.1 Metal Backend Foundation
- [ ] Create `src/core/backends/metal.rs`
- [ ] Implement `InferenceBackend` trait for Metal
- [ ] Add Metal feature detection at runtime
- [ ] Implement GPU memory management for Unified Memory Architecture
- [ ] Add Metal device enumeration and selection

**Key Technologies**:
- `metal-rs` crate for Metal API bindings
- Metal Performance Shaders (MPS) for neural network ops
- Metal Compute Pipeline for custom kernels
- Core ML integration for Apple Neural Engine (ANE)

#### 2.2 GGUF Metal Integration
- [ ] Port llama.cpp Metal kernels to Rust
- [ ] Implement quantized model loading (Q4_0, Q4_1, Q5_0, Q5_1, Q8_0)
- [ ] Create Metal compute shaders for matrix operations
- [ ] Implement Metal buffer management
- [ ] Add memory mapping for large models

**Performance Targets**:
- 7B parameter model: <2GB VRAM, >30 tokens/sec
- 13B parameter model: <8GB VRAM, >15 tokens/sec
- 70B parameter model: <40GB VRAM, >5 tokens/sec (unified memory)

#### 2.3 ARM64 NEON Optimization
- [ ] Enable ARM NEON SIMD intrinsics
- [ ] Optimize matrix multiplication for NEON
- [ ] Implement vectorized quantization/dequantization
- [ ] Add ARM64-specific compiler flags
- [ ] Create performance benchmarks for CPU vs GPU

**Compiler Flags**:
```toml
[target.'cfg(all(target_arch = "aarch64", target_os = "macos"))']
rustflags = [
    "-C", "target-cpu=apple-m1",
    "-C", "target-feature=+neon,+fp-armv8,+crc",
    "-C", "opt-level=3"
]
```

#### 2.4 System Integration
- [ ] Implement chip detection (M1/M2/M3/M4)
- [ ] Add performance core detection and affinity
- [ ] Integrate with macOS power management API
- [ ] Add thermal throttling awareness
- [ ] Implement battery-aware performance profiles

### Success Criteria for Phase 2
- ✅ Metal backend fully functional
- ✅ 3x faster inference on GPU vs CPU
- ✅ <50% battery impact vs CPU inference
- ✅ Automatic GPU/CPU fallback
- ✅ Thermal throttling handled gracefully

---

## Phase 3: macOS Native Experience 🎨

**Goal**: Indistinguishable from native Apple applications
**Duration**: 4-5 days
**Status**: 📋 Planned

### Task Breakdown

#### 3.1 Native UI Components
- [ ] Implement native menu bar with standard macOS shortcuts
- [ ] Add Touch Bar support for MacBook Pro
- [ ] Create SF Symbols icon set
- [ ] Implement window vibrancy effects
- [ ] Add native animations (Core Animation)
- [ ] Support light/dark mode auto-switching

**Menu Structure**:
```
Inferno
├── About Inferno
├── Preferences... ⌘,
├── Services
├── Hide Inferno ⌘H
├── Hide Others ⌥⌘H
├── Show All
└── Quit Inferno ⌘Q

File
├── New Inference ⌘N
├── Open Model... ⌘O
├── Import Model... ⌘I
└── Close Window ⌘W

Models
├── Load Model ⌘L
├── Unload All ⌘⇧U
└── Model Info ⌘⇧I

Inference
├── Run Inference ⌘R
├── Stream Inference ⌘⇧R
└── Stop Inference ⌘.
```

#### 3.2 System Integration Features
- [ ] Spotlight search for models (`.gguf`, `.onnx` indexing)
- [ ] Quick Look plugin for model file preview
- [ ] Drag-and-drop model installation
- [ ] Share Sheet for exporting inference results
- [ ] Handoff support between Apple devices
- [ ] Stage Manager compatibility
- [ ] Mission Control integration

#### 3.3 Notification System
- [ ] Native notification center integration
- [ ] Action buttons in notifications
- [ ] Notification grouping by type
- [ ] Do Not Disturb awareness
- [ ] Critical alerts for errors

#### 3.4 System Tray Enhancement
- [ ] Live metrics in menu bar (CPU/GPU usage)
- [ ] Quick actions menu
- [ ] Active inference indicator
- [ ] Battery status awareness
- [ ] One-click model switching

### Success Criteria for Phase 3
- ✅ Passes macOS Human Interface Guidelines
- ✅ Native look and feel
- ✅ All standard macOS shortcuts work
- ✅ Spotlight indexing functional
- ✅ Touch Bar support for compatible devices

---

## Phase 4: Codebase Cleanup 🧹

**Goal**: Zero module duplication, <15 root-level modules
**Duration**: 2-3 days
**Status**: 📋 Planned

### Task Breakdown

#### 4.1 Remove Duplicate Modules
**Cache System** (3 → 1):
- [ ] Delete `src/cache.rs`
- [ ] Delete `src/response_cache.rs`
- [ ] Delete `src/advanced_cache.rs`
- [ ] Keep only `src/infrastructure/cache/`
- [ ] Update all imports

**Monitoring System** (2 → 1):
- [ ] Delete `src/monitoring.rs`
- [ ] Delete `src/advanced_monitoring.rs`
- [ ] Keep only `src/infrastructure/monitoring/`
- [ ] Update all imports

**Optimization System** (3 → 1):
- [ ] Delete `src/optimization.rs`
- [ ] Delete `src/performance_optimization.rs`
- [ ] Delete `src/performance_baseline.rs`
- [ ] Keep only `src/ai_features/optimization/`
- [ ] Update all imports

**Audit System** (2 → 1):
- [ ] Delete `src/audit.rs`
- [ ] Delete `src/logging_audit.rs`
- [ ] Keep only `src/infrastructure/audit/`
- [ ] Update all imports

**Versioning System** (2 → 1):
- [ ] Delete `src/versioning.rs`
- [ ] Delete `src/model_versioning.rs`
- [ ] Keep only `src/operations/versioning/`
- [ ] Update all imports

#### 4.2 Update Module Re-exports
- [ ] Remove backward compatibility re-exports from `src/lib.rs`
- [ ] Force all code to use organized paths
- [ ] Update all internal imports
- [ ] Update example code
- [ ] Update tests
- [ ] Update benchmarks

#### 4.3 CLI Simplification
- [ ] Remove GUI-focused CLI commands
- [ ] Keep automation-focused commands
- [ ] Simplify TUI (optional secondary interface)
- [ ] Focus on developer/scripting use cases

**CLI Commands to Keep**:
- Model management (list, validate, convert)
- Batch operations (for automation)
- Server mode (headless operation)
- Diagnostic commands (health check, metrics export)

**CLI Commands to Remove/Move to GUI**:
- Interactive model selection
- Real-time monitoring dashboards
- Visual configuration editors

### Success Criteria for Phase 4
- ✅ <15 modules in `src/` root
- ✅ Zero duplicate functionality
- ✅ All imports use organized paths
- ✅ CLI focused on automation
- ✅ No deprecated code warnings

---

## Phase 5: Distribution & Deployment 📦

**Goal**: Professional macOS distribution ready for users
**Duration**: 3-4 days
**Status**: 📋 Planned

### Task Breakdown

#### 5.1 macOS App Bundle
- [ ] Create signed `.app` bundle with proper structure
- [ ] Generate Info.plist with all required keys
- [ ] Add proper entitlements (sandboxing, hardened runtime)
- [ ] Code sign with Developer ID certificate
- [ ] Notarize with Apple notary service
- [ ] Test Gatekeeper compatibility

**App Bundle Structure**:
```
Inferno.app/
├── Contents/
│   ├── Info.plist
│   ├── MacOS/
│   │   └── inferno-app (universal binary)
│   ├── Resources/
│   │   ├── icon.icns
│   │   └── models/
│   └── _CodeSignature/
```

#### 5.2 DMG Installer
- [ ] Create professional DMG with custom background
- [ ] Add drag-to-Applications workflow
- [ ] Include README and license
- [ ] Sign DMG image
- [ ] Test installation flow

#### 5.3 Auto-Update System
- [ ] Integrate Tauri updater plugin
- [ ] Set up update server/CDN
- [ ] Implement delta updates
- [ ] Add update notification UI
- [ ] Create rollback mechanism

#### 5.4 Distribution Channels
- [ ] Prepare for Mac App Store submission
- [ ] Create Homebrew cask formula
- [ ] Set up GitHub Releases workflow
- [ ] Add Sparkle appcast for updates

**Homebrew Cask**:
```ruby
cask "inferno" do
  version "0.5.0"
  sha256 "..."

  url "https://github.com/inferno-ai/inferno/releases/download/v#{version}/Inferno-universal.dmg"
  name "Inferno AI Runner"
  desc "Offline AI/ML model runner optimized for Apple Silicon"
  homepage "https://inferno-ai.dev"

  app "Inferno.app"
end
```

#### 5.5 Performance Documentation
- [ ] Create Metal GPU benchmarks (M1/M2/M3/M4)
- [ ] Document memory usage patterns
- [ ] Establish performance baselines
- [ ] Create regression test suite
- [ ] Add telemetry for optimization feedback

### Success Criteria for Phase 5
- ✅ Signed and notarized .app bundle
- ✅ Professional DMG installer
- ✅ Auto-update system working
- ✅ Homebrew cask published
- ✅ Performance benchmarks documented

---

## Phase 6: Documentation & Polish 📝

**Goal**: Comprehensive documentation for macOS users
**Duration**: 2-3 days
**Status**: 📋 Planned

### Task Breakdown

#### 6.1 User Documentation
- [ ] macOS installation guide
- [ ] Quick start tutorial
- [ ] Model installation guide
- [ ] Performance optimization guide
- [ ] Troubleshooting for Apple Silicon
- [ ] FAQ section

#### 6.2 Developer Documentation
- [ ] Metal backend API documentation
- [ ] Desktop interface integration guide
- [ ] macOS-specific development guide
- [ ] Contributing guidelines
- [ ] Build and release process

#### 6.3 Marketing Materials
- [ ] Screenshots showcasing macOS UI
- [ ] Performance comparison charts
- [ ] Video demo of key features
- [ ] Blog post announcing v0.5.0

### Success Criteria for Phase 6
- ✅ Complete user guide
- ✅ Developer documentation
- ✅ Marketing materials ready
- ✅ Release announcement prepared

---

## 📈 Success Metrics

| Metric | Current (v0.4.0) | Target (v0.5.0) | Status |
|--------|------------------|-----------------|--------|
| **Desktop Binaries** | 2 (Tauri v1 + v2) | 1 (unified) | 📋 |
| **Metal Acceleration** | ❌ Not implemented | ✅ 3-5x faster | 📋 |
| **Native macOS UX** | ⚠️ Partial | ✅ Complete | 📋 |
| **Root Modules** | 40+ | <15 | 📋 |
| **Binary Size** | ~80MB | <50MB | 📋 |
| **Cold Start Time** | ~3s | <2s | 📋 |
| **Memory (7B model)** | ~4GB | ~2GB | 📋 |
| **Tokens/sec (M1, 7B)** | ~10 (CPU) | ~30-40 (Metal) | 📋 |

---

## ⚠️ Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Metal API complexity | High | Medium | Gradual rollout, CPU fallback |
| Tauri v2 migration issues | Medium | Low | Comprehensive testing |
| Performance regression | High | Low | Benchmark suite, CI checks |
| Code signing/notarization | Medium | Medium | Early testing, automation |
| Binary size increase | Low | Medium | Aggressive optimization, stripping |

---

## 🔄 Session Notes

### Session 1: 2025-09-29
**Status**: Phase 1 initiated
**Completed**:
- [x] Created comprehensive plan document
- [ ] Auditing current Tauri implementations

**Next Steps**:
1. Complete audit of Tauri v1 vs v2 functionality
2. Create new `src/interfaces/desktop/` structure
3. Begin command migration

**Blockers**: None

**Notes**:
- Dashboard/src-tauri has complete implementation with SQLite, security, events
- Main codebase has 1,811 lines in upgrade system for auto-updates
- Icon assets are professional and complete

---

## 📚 Technical References

### Dependencies to Add
- `metal-rs` - Metal API bindings
- `core-foundation` - macOS system integration
- `cocoa` - macOS UI integration
- `block` - Objective-C block support

### Dependencies to Remove
- Old Tauri v1 dependencies
- Redundant async runtimes
- Unused ML backend features

### Build Tools
- Xcode Command Line Tools (Metal shader compilation)
- `cargo-bundle` (app bundle creation)
- `codesign` (code signing)
- `xcrun notarytool` (notarization)

---

## 🎯 Definition of Done

**v0.5.0 is complete when**:
1. ✅ Single `inferno-app` binary for macOS (universal)
2. ✅ Metal GPU acceleration working on Apple Silicon
3. ✅ Native macOS UI indistinguishable from Apple apps
4. ✅ Zero module duplication in codebase
5. ✅ Signed, notarized, and installable via DMG
6. ✅ Auto-update system functional
7. ✅ Complete documentation published
8. ✅ Performance benchmarks meet targets
9. ✅ All tests passing (unit, integration, e2e)
10. ✅ Released on GitHub with Homebrew cask

---

**Legend**:
- 📋 Planned
- 🔄 In Progress
- ✅ Completed
- ❌ Blocked
- ⚠️ At Risk