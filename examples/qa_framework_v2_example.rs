//! QA Framework Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::qa_framework_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== QA Framework Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Run all tests
    println!("Example 1: Run all tests");
    let cmd = QATest::new(config.clone(), None, None, false, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Run specific test
    println!("Example 2: Run specific test");
    let cmd = QATest::new(
        config.clone(),
        Some("test_inference".to_string()),
        None,
        false,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Run test suite in parallel
    println!("Example 3: Run test suite in parallel");
    let cmd = QATest::new(
        config.clone(),
        None,
        Some("integration".to_string()),
        true,
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Generate HTML report
    println!("Example 4: Generate HTML report");
    let cmd = QAReport::new(
        config.clone(),
        "html".to_string(),
        Some("reports/test-report.html".to_string()),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Generate JSON report
    println!("Example 5: Generate JSON report");
    let cmd = QAReport::new(config.clone(), "json".to_string(), None, false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Analyze coverage
    println!("Example 6: Analyze coverage");
    let cmd = QACoverage::new(config.clone(), None, Some(80.0), false);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Detailed coverage
    println!("Example 7: Detailed coverage");
    let cmd = QACoverage::new(config.clone(), Some("core".to_string()), None, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Create test suite
    println!("Example 8: Create test suite");
    let cmd = QASuite::new(
        config.clone(),
        "create".to_string(),
        "smoke".to_string(),
        Some(vec!["test_1".to_string(), "test_2".to_string()]),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: List test suites
    println!("Example 9: List test suites");
    let cmd = QASuite::new(config.clone(), "list".to_string(), "".to_string(), None);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Run benchmark
    println!("Example 10: Run benchmark");
    let cmd = QABenchmark::new(config.clone(), None, 100, true);
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    println!("=== All examples completed successfully ===");
    Ok(())
}
