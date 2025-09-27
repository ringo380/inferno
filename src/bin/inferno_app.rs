#[cfg(not(feature = "tauri-app"))]
use anyhow::Result;
#[cfg(feature = "tauri-app")]
use inferno::tauri_app::run_tauri_app;

#[cfg(not(feature = "tauri-app"))]
fn main() -> Result<()> {
    eprintln!("Tauri app feature not enabled. Build with --features tauri-app");
    std::process::exit(1);
}

#[cfg(feature = "tauri-app")]
fn main() -> anyhow::Result<()> {
    run_tauri_app()
}
