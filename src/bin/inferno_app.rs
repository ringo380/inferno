//! # DEPRECATED: Tauri v1 Desktop Binary (v0.5.0+)
//!
//! **⚠️ DEPRECATION NOTICE ⚠️**
//!
//! This binary is deprecated as of v0.5.0 and will be removed in v0.6.0.
//!
//! ## Migration Path
//!
//! **Old (Tauri v1)**: `inferno_app` binary with `--features tauri-app`
//! **New (Tauri v2)**: `inferno-desktop` binary in `dashboard/src-tauri/`
//!
//! ### Why Deprecated?
//!
//! 1. **Tauri v2 Superior**: 51 commands vs 14 (3.6x more functionality)
//! 2. **Better Architecture**: Unified with dashboard implementation
//! 3. **Apple Silicon Native**: M1/M2/M3/M4 optimizations
//! 4. **Dependency Conflicts**: Tauri v1 and v2 cannot coexist
//! 5. **Modern Features**: Auto-updates, plugins, better performance
//!
//! ### How to Migrate
//!
//! Instead of building this binary, use the new desktop app:
//!
//! ```bash
//! # OLD (deprecated):
//! cargo run --bin inferno_app --features tauri-app
//!
//! # NEW (recommended):
//! cd dashboard && npm run tauri dev              # Development
//! ./scripts/build-desktop.sh --release           # Release build
//! ./scripts/build-desktop.sh --release --universal  # Universal binary
//! ```
//!
//! ### Distribution
//!
//! The new desktop app creates proper macOS installers:
//! - **DMG**: `dashboard/src-tauri/target/release/bundle/dmg/Inferno.dmg`
//! - **App Bundle**: `dashboard/src-tauri/target/release/bundle/macos/Inferno.app`
//!
//! See: `dashboard/src-tauri/` and `scripts/build-desktop.sh` for details.

#[cfg(not(feature = "desktop"))]
use anyhow::Result;
#[cfg(feature = "desktop")]
use inferno_ai::tauri_app::run_tauri_app;

#[cfg(not(feature = "desktop"))]
fn main() -> Result<()> {
    eprintln!("⚠️  DEPRECATED: This binary is deprecated as of v0.5.0");
    eprintln!("   Use the new Tauri v2 desktop app instead:");
    eprintln!();
    eprintln!("   cd dashboard && npm run tauri dev");
    eprintln!("   OR");
    eprintln!("   ./scripts/build-desktop.sh --release");
    eprintln!();
    eprintln!("   See src/bin/inferno_app.rs for migration details.");
    std::process::exit(1);
}

#[cfg(feature = "desktop")]
fn main() -> anyhow::Result<()> {
    eprintln!("⚠️  DEPRECATED: This binary is deprecated as of v0.5.0");
    eprintln!("   Use the new Tauri v2 desktop app instead:");
    eprintln!("   cd dashboard && npm run tauri dev");
    eprintln!();
    run_tauri_app()
}
