//! Optimization Command Examples - New Architecture
//!
//! This example demonstrates the usage of optimization commands in the v2 architecture.

use anyhow::Result;
use inferno::{
    cli::optimization_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Optimization Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Quantize to int8
    println!("Example 1: Quantize to int8");
    let cmd = OptimizeQuantize::new(
        config.clone(),
        "models/llama-7b.gguf".to_string(),
        "models/llama-7b-int8.gguf".to_string(),
        "int8".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Quantize to fp16
    println!("Example 2: Quantize to fp16");
    let cmd = OptimizeQuantize::new(
        config.clone(),
        "models/model.onnx".to_string(),
        "models/model-fp16.onnx".to_string(),
        "fp16".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Quantize to int4 (aggressive)
    println!("Example 3: Quantize to int4 (aggressive)");
    let cmd = OptimizeQuantize::new(
        config.clone(),
        "models/large-model.gguf".to_string(),
        "models/large-model-int4.gguf".to_string(),
        "int4".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Prune with 30% sparsity
    println!("Example 4: Prune with 30% sparsity");
    let cmd = OptimizePrune::new(
        config.clone(),
        "models/model.gguf".to_string(),
        "models/model-pruned.gguf".to_string(),
        0.3,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Aggressive pruning (50%)
    println!("Example 5: Aggressive pruning (50%)");
    let cmd = OptimizePrune::new(
        config.clone(),
        "models/model.gguf".to_string(),
        "models/model-sparse.gguf".to_string(),
        0.5,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Conservative pruning (10%)
    println!("Example 6: Conservative pruning (10%)");
    let cmd = OptimizePrune::new(
        config.clone(),
        "models/model.gguf".to_string(),
        "models/model-light-prune.gguf".to_string(),
        0.1,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 7: Distill with temperature 2.0
    println!("Example 7: Distill with temperature 2.0");
    let cmd = OptimizeDistill::new(
        config.clone(),
        "models/teacher-large.gguf".to_string(),
        "models/student-small.gguf".to_string(),
        "models/student-distilled.gguf".to_string(),
        2.0,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 8: Distill with temperature 4.0 (softer)
    println!("Example 8: Distill with temperature 4.0 (softer)");
    let cmd = OptimizeDistill::new(
        config.clone(),
        "models/teacher.gguf".to_string(),
        "models/student.gguf".to_string(),
        "models/distilled.gguf".to_string(),
        4.0,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 9: Benchmark single technique
    println!("Example 9: Benchmark single technique");
    let cmd = OptimizeBenchmark::new(
        config.clone(),
        "models/model.gguf".to_string(),
        vec!["quantize".to_string()],
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 10: Benchmark multiple techniques
    println!("Example 10: Benchmark multiple techniques");
    let cmd = OptimizeBenchmark::new(
        config.clone(),
        "models/model.gguf".to_string(),
        vec!["quantize".to_string(), "prune".to_string()],
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 11: Benchmark all techniques
    println!("Example 11: Benchmark all techniques");
    let cmd = OptimizeBenchmark::new(
        config.clone(),
        "models/model.gguf".to_string(),
        vec!["all".to_string()],
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 12: Basic profiling
    println!("Example 12: Basic profiling");
    let cmd = OptimizeProfile::new(
        config.clone(),
        "models/model.gguf".to_string(),
        false,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 13: Detailed profiling
    println!("Example 13: Detailed profiling");
    let cmd = OptimizeProfile::new(
        config.clone(),
        "models/model.gguf".to_string(),
        true,
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 14: Validation - Invalid precision
    println!("Example 14: Validation - Invalid precision");
    let cmd = OptimizeQuantize::new(
        config.clone(),
        "input.gguf".to_string(),
        "output.gguf".to_string(),
        "invalid".to_string(),
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 15: Validation - Invalid sparsity (too high)
    println!("Example 15: Validation - Invalid sparsity (too high)");
    let cmd = OptimizePrune::new(
        config.clone(),
        "input.gguf".to_string(),
        "output.gguf".to_string(),
        1.5,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 16: Validation - Invalid sparsity (negative)
    println!("Example 16: Validation - Invalid sparsity (negative)");
    let cmd = OptimizePrune::new(
        config.clone(),
        "input.gguf".to_string(),
        "output.gguf".to_string(),
        -0.1,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 17: Validation - Zero temperature
    println!("Example 17: Validation - Zero temperature");
    let cmd = OptimizeDistill::new(
        config.clone(),
        "teacher.gguf".to_string(),
        "student.gguf".to_string(),
        "output.gguf".to_string(),
        0.0,
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    // Example 18: Validation - Invalid benchmark technique
    println!("Example 18: Validation - Invalid benchmark technique");
    let cmd = OptimizeBenchmark::new(
        config.clone(),
        "model.gguf".to_string(),
        vec!["invalid".to_string()],
    );
    let ctx = CommandContext::new(config.clone());
    match cmd.validate(&ctx).await {
        Ok(_) => println!("Validation passed unexpectedly"),
        Err(e) => println!("Validation failed as expected: {}\n", e),
    }

    println!("=== All examples completed successfully ===");
    Ok(())
}