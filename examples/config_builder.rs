//! Configuration Builder Examples
//!
//! This example demonstrates how to use the new configuration builder pattern
//! introduced in Inferno v0.4.0.
//!
//! Run with: cargo run --example config_builder

use inferno::core::config::{ConfigBuilder, LogFormat, LogLevel, Preset};

fn main() -> anyhow::Result<()> {
    println!("🔥 Inferno Configuration Builder Examples\n");

    // Example 1: Simple configuration
    println!("Example 1: Simple Configuration");
    println!("═══════════════════════════════");
    let simple_config = ConfigBuilder::new()
        .models_dir("./models")
        .build_unchecked();

    println!("Models directory: {}", simple_config.models_dir.display());
    println!("Cache directory: {}", simple_config.cache_dir.display());
    println!("Log level: {}", simple_config.log_level);
    println!("Log format: {}\n", simple_config.log_format);

    // Example 2: Using presets
    println!("Example 2: Development Preset");
    println!("═══════════════════════════════");
    let dev_config = ConfigBuilder::new()
        .preset(Preset::Development)
        .models_dir("./dev-models")
        .build_unchecked();

    println!("Preset: Development");
    println!(
        "Log level: {} (verbose for debugging)",
        dev_config.log_level
    );
    println!("Log format: {} (human-readable)", dev_config.log_format);
    println!("Description: {}\n", Preset::Development.description());

    // Example 3: Production preset
    println!("Example 3: Production Preset");
    println!("═══════════════════════════════");
    let prod_config = ConfigBuilder::new()
        .preset(Preset::Production)
        .models_dir("./prod-models")
        .build_unchecked();

    println!("Preset: Production");
    println!("Log level: {} (optimized)", prod_config.log_level);
    println!(
        "Log format: {} (for log aggregation)",
        prod_config.log_format
    );
    println!("Cache enabled: {}", Preset::Production.cache_enabled());
    println!(
        "Monitoring enabled: {}",
        Preset::Production.monitoring_enabled()
    );
    println!(
        "Max concurrent requests: {}",
        Preset::Production.max_concurrent_requests()
    );
    println!("Description: {}\n", Preset::Production.description());

    // Example 4: Testing preset
    println!("Example 4: Testing Preset");
    println!("═══════════════════════════════");
    let test_config = ConfigBuilder::new()
        .preset(Preset::Testing)
        .models_dir("./test-models")
        .build_unchecked();

    println!("Preset: Testing");
    println!("Log level: {} (minimal output)", test_config.log_level);
    println!("Log format: {} (compact)", test_config.log_format);
    println!("Cache enabled: {}", Preset::Testing.cache_enabled());
    println!("Description: {}\n", Preset::Testing.description());

    // Example 5: Benchmark preset
    println!("Example 5: Benchmark Preset");
    println!("═══════════════════════════════");
    let bench_config = ConfigBuilder::new()
        .preset(Preset::Benchmark)
        .models_dir("./benchmark-models")
        .build_unchecked();

    println!("Preset: Benchmark");
    println!("Log level: {} (minimal overhead)", bench_config.log_level);
    println!(
        "Monitoring enabled: {} (no overhead)",
        Preset::Benchmark.monitoring_enabled()
    );
    println!(
        "Request timeout: {}s (long for benchmarks)",
        Preset::Benchmark.request_timeout_seconds()
    );
    println!("Description: {}\n", Preset::Benchmark.description());

    // Example 6: Customizing a preset
    println!("Example 6: Customized Production Config");
    println!("════════════════════════════════════════");
    let custom_config = ConfigBuilder::new()
        .preset(Preset::Production)
        .models_dir("./custom-models")
        .log_level(LogLevel::Warn) // Override preset
        .log_format(LogFormat::Pretty) // Override for local testing
        .build_unchecked();

    println!("Base: Production preset");
    println!("Custom log level: {} (overridden)", custom_config.log_level);
    println!(
        "Custom log format: {} (overridden)",
        custom_config.log_format
    );
    println!("Other settings: Inherited from Production\n");

    // Example 7: Fully custom configuration
    println!("Example 7: Fully Custom Configuration");
    println!("══════════════════════════════════════");
    let fully_custom = ConfigBuilder::new()
        .models_dir("./my-special-models")
        .cache_dir("./my-special-cache")
        .log_level(LogLevel::Trace)
        .log_format(LogFormat::Json)
        .build_unchecked();

    println!("No preset used - all custom settings");
    println!("Models: {}", fully_custom.models_dir.display());
    println!("Cache: {}", fully_custom.cache_dir.display());
    println!("Log level: {}", fully_custom.log_level);
    println!("Log format: {}\n", fully_custom.log_format);

    // Example 8: Comparing presets
    println!("Example 8: Preset Comparison");
    println!("═════════════════════════════");
    println!(
        "{:<15} {:<15} {:<10} {:<10} {:<10}",
        "Preset", "Log Level", "Cache", "Monitor", "Max Req"
    );
    println!("{}", "─".repeat(65));

    for preset in &[
        Preset::Development,
        Preset::Production,
        Preset::Testing,
        Preset::Benchmark,
    ] {
        let config = preset.apply_to_core(inferno::core::config::CoreConfig::default());
        println!(
            "{:<15} {:<15} {:<10} {:<10} {:<10}",
            format!("{:?}", preset),
            format!("{}", config.log_level),
            preset.cache_enabled(),
            preset.monitoring_enabled(),
            preset.max_concurrent_requests()
        );
    }

    println!("\n✅ All examples completed successfully!");
    println!("\n💡 Next steps:");
    println!("   1. Choose a preset that matches your use case");
    println!("   2. Customize specific settings as needed");
    println!("   3. Use .build()? for validation and directory creation");
    println!("   4. Check src/core/config/README.md for more details");

    Ok(())
}
