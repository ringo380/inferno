//! CLI Architecture Example
//!
//! Demonstrates the new CLI command architecture with:
//! - Command trait implementation
//! - Command pipeline with middleware
//! - Context and structured output
//!
//! Run with: cargo run --example cli_architecture

use anyhow::Result;
use async_trait::async_trait;
use inferno::core::config::ConfigBuilder;
use inferno::interfaces::cli::{
    Command, CommandContext, CommandOutput, CommandPipeline, LoggingMiddleware, MetricsMiddleware,
};
use serde_json::json;

/// Example echo command - demonstrates basic command implementation
pub struct EchoCommand {
    /// Message to echo
    message: String,
    /// Number of times to repeat
    repeat: usize,
}

#[async_trait]
impl Command for EchoCommand {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo a message multiple times"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.message.is_empty() {
            anyhow::bail!("Message cannot be empty");
        }
        if self.repeat == 0 {
            anyhow::bail!("Repeat count must be greater than 0");
        }
        if self.repeat > 100 {
            anyhow::bail!("Repeat count too high (max: 100)");
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let mut lines = Vec::new();

        for i in 1..=self.repeat {
            let line = if ctx.is_verbose() {
                format!("[{}/{}] {}", i, self.repeat, self.message)
            } else {
                self.message.clone()
            };
            lines.push(line.clone());

            if !ctx.json_output {
                println!("{}", line);
            }
        }

        Ok(CommandOutput::success_with_data(
            format!("Echoed message {} times", self.repeat),
            json!({
                "message": self.message,
                "repeat": self.repeat,
                "lines": lines,
            }),
        ))
    }
}

/// Example command that demonstrates error handling
pub struct FailingCommand;

#[async_trait]
impl Command for FailingCommand {
    fn name(&self) -> &str {
        "fail"
    }

    fn description(&self) -> &str {
        "Always fails - demonstrates error handling"
    }

    async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
        anyhow::bail!("This command intentionally fails for demonstration purposes")
    }
}

/// Example command with validation
pub struct ValidatingCommand {
    value: i32,
}

#[async_trait]
impl Command for ValidatingCommand {
    fn name(&self) -> &str {
        "validate"
    }

    fn description(&self) -> &str {
        "Validates input before execution"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.value < 0 {
            anyhow::bail!("Value must be non-negative");
        }
        if self.value > 1000 {
            anyhow::bail!("Value must be less than 1000");
        }
        Ok(())
    }

    async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
        Ok(CommandOutput::success_with_data(
            format!("Value {} is valid!", self.value),
            json!({
                "value": self.value,
                "is_even": self.value % 2 == 0,
                "squared": self.value * self.value,
            }),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for log output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔥 Inferno CLI Architecture Examples\n");

    // Create a configuration
    let config = inferno::config::Config::default();
    let core_config = ConfigBuilder::new().build_unchecked();

    // Build command pipeline with middleware
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .with_middleware(Box::new(MetricsMiddleware::new()))
        .build();

    println!("Example 1: Simple Echo Command");
    println!("═══════════════════════════════");

    let echo_cmd = EchoCommand {
        message: "Hello, Inferno!".to_string(),
        repeat: 3,
    };

    let mut ctx = CommandContext::with_configs(config.clone(), core_config.clone());
    let output = pipeline.execute(&echo_cmd, &mut ctx).await?;

    println!("\nOutput:");
    println!("  Success: {}", output.success);
    println!("  Message: {}", output.message.unwrap_or_default());
    println!("  Duration: {:?}", ctx.elapsed());
    println!();

    println!("Example 2: Echo with JSON Output");
    println!("═════════════════════════════════");

    let echo_cmd2 = EchoCommand {
        message: "JSON formatted!".to_string(),
        repeat: 2,
    };

    let mut ctx2 = CommandContext::with_configs(config.clone(), core_config.clone());
    ctx2.set_json_output(true);
    let output2 = pipeline.execute(&echo_cmd2, &mut ctx2).await?;

    println!("\nJSON Output:");
    println!("{}", output2.to_json()?);
    println!();

    println!("Example 3: Verbose Mode");
    println!("═══════════════════════");

    let echo_cmd3 = EchoCommand {
        message: "Verbose output".to_string(),
        repeat: 3,
    };

    let mut ctx3 = CommandContext::with_configs(config.clone(), core_config.clone());
    ctx3.set_verbosity(1); // Enable verbose mode
    let _output3 = pipeline.execute(&echo_cmd3, &mut ctx3).await?;
    println!();

    println!("Example 4: Validation Success");
    println!("══════════════════════════════");

    let valid_cmd = ValidatingCommand { value: 42 };

    let mut ctx4 = CommandContext::with_configs(config.clone(), core_config.clone());
    let output4 = pipeline.execute(&valid_cmd, &mut ctx4).await?;

    println!("\nValidation passed!");
    println!("Message: {}", output4.message.unwrap_or_default());
    println!("Data: {}", serde_json::to_string_pretty(&output4.data)?);
    println!();

    println!("Example 5: Validation Failure");
    println!("══════════════════════════════");

    let invalid_cmd = ValidatingCommand { value: -10 };

    let mut ctx5 = CommandContext::with_configs(config.clone(), core_config.clone());
    let output5 = pipeline.execute(&invalid_cmd, &mut ctx5).await?;

    println!("\nValidation failed (as expected):");
    println!("Success: {}", output5.success);
    println!("Message: {}", output5.message.unwrap_or_default());
    println!("Exit code: {}", output5.exit_code);
    println!();

    println!("Example 6: Command Error Handling");
    println!("══════════════════════════════════");

    let failing_cmd = FailingCommand;

    let mut ctx6 = CommandContext::with_configs(config.clone(), core_config);
    let output6 = pipeline.execute(&failing_cmd, &mut ctx6).await?;

    println!("\nCommand failed (as expected):");
    println!("Success: {}", output6.success);
    println!("Message: {}", output6.message.unwrap_or_default());
    println!("Exit code: {}", output6.exit_code);
    println!();

    println!("✅ All examples completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   1. Command trait implementation");
    println!("   2. Command validation before execution");
    println!("   3. Middleware (logging + metrics)");
    println!("   4. Context with verbosity and JSON output modes");
    println!("   5. Structured output with success/error handling");
    println!("   6. Automatic error handling via pipeline");

    Ok(())
}
