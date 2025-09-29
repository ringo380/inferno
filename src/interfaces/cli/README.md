# Inferno CLI Architecture

**Version**: 0.4.0+
**Status**: Stable (Phase 2 Foundation)
**Migration**: Fully backward compatible

## Overview

The new CLI architecture provides a unified, extensible command system with:

- **Command Trait**: Standard interface for all commands
- **Command Pipeline**: Orchestrates execution with middleware
- **Middleware System**: Cross-cutting concerns (logging, metrics, validation)
- **Command Context**: Shared state and configuration
- **Structured Output**: Type-safe command results

## Quick Start

### Creating a Command

```rust
use inferno::interfaces::cli::{Command, CommandContext, CommandOutput};
use async_trait::async_trait;
use anyhow::Result;

pub struct MyCommand {
    arg1: String,
    arg2: i32,
}

#[async_trait]
impl Command for MyCommand {
    fn name(&self) -> &str {
        "my-command"
    }

    fn description(&self) -> &str {
        "Does something useful"
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Access configuration
        let models_dir = &ctx.config.models_dir;

        // Do work...

        // Return structured output
        Ok(CommandOutput::success("Command completed successfully"))
    }
}
```

### Using the Pipeline

```rust
use inferno::interfaces::cli::{CommandPipeline, LoggingMiddleware, MetricsMiddleware};

let pipeline = CommandPipeline::builder()
    .with_middleware(Box::new(LoggingMiddleware::new()))
    .with_middleware(Box::new(MetricsMiddleware::new()))
    .build();

let command = MyCommand {
    arg1: "value".to_string(),
    arg2: 42,
};

let mut ctx = CommandContext::new(config);
let output = pipeline.execute(&command, &mut ctx).await?;

if output.success {
    println!("{}", output.to_display());
} else {
    std::process::exit(output.exit_code);
}
```

## Architecture

### Components

1. **Command Trait** (`command.rs`)
   - Defines standard interface for all commands
   - Includes validation, execution, and metadata
   - Async-first with `#[async_trait]`

2. **CommandContext** (`context.rs`)
   - Shared context for command execution
   - Contains configuration, args, state
   - Execution metadata (ID, timing, metrics)

3. **CommandOutput** (`output.rs`)
   - Structured output from commands
   - Supports JSON and human-readable formats
   - Exit codes and severity levels

4. **Middleware** (`middleware/`)
   - Trait for pre/post execution hooks
   - Built-in: Logging, Metrics
   - Stackable and conditional

5. **CommandPipeline** (`pipeline.rs`)
   - Orchestrates command execution
   - Manages middleware lifecycle
   - Handles errors uniformly

### Execution Flow

```text
┌─────────────────────────────────────────────────────────┐
│                    CommandPipeline                      │
├─────────────────────────────────────────────────────────┤
│ 1. Run pre-execution middleware (logging, setup)        │
│ 2. Validate command arguments and context               │
│ 3. Execute command with shared context                  │
│ 4. Run post-execution middleware (metrics, cleanup)     │
│ 5. Handle errors and format output                      │
└─────────────────────────────────────────────────────────┘
```

## Key Features

### 1. Validation Before Execution

```rust
#[async_trait]
impl Command for MyCommand {
    async fn validate(&self, ctx: &CommandContext) -> Result<()> {
        if ctx.get_arg("required").is_none() {
            anyhow::bail!("Missing required argument");
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Validation passed, safe to execute
        Ok(CommandOutput::success("Done"))
    }
}
```

### 2. Structured Output

```rust
// Success with data
CommandOutput::success_with_data(
    "Found 5 models",
    json!({ "count": 5, "models": models })
)

// Warning (non-fatal)
CommandOutput::warning("No cache entries found", None)

// Error with exit code
CommandOutput::error("File not found", 1)
```

### 3. Command Context

```rust
async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
    // Access configuration
    let models_dir = &ctx.config.models_dir;

    // Check flags
    if ctx.is_verbose() {
        println!("Verbose mode enabled");
    }

    // Store state for middleware
    ctx.set_state("model_count", 42);

    // JSON output mode
    if ctx.json_output {
        return Ok(CommandOutput::data(json!({ "result": "..." })));
    }

    Ok(CommandOutput::success("Done"))
}
```

### 4. Middleware System

```rust
pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
        tracing::info!("Starting command: {}", ctx.execution_id);
        Ok(())
    }

    async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()> {
        tracing::info!("Finished in {:?}", ctx.elapsed());
        Ok(())
    }
}
```

## Migration Guide

### From Old Style

**Old way (still works)**:
```rust
// src/cli/old_command.rs
pub async fn execute(args: OldCommandArgs, config: &Config) -> Result<()> {
    // Command logic
    Ok(())
}

// main.rs
Commands::OldCommand(args) => old_command::execute(args, &config).await?,
```

**New way (recommended)**:
```rust
// src/interfaces/cli/commands/new_command.rs
pub struct NewCommand {
    args: NewCommandArgs,
}

#[async_trait]
impl Command for NewCommand {
    fn name(&self) -> &str { "new-command" }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Command logic with context
        Ok(CommandOutput::success("Done"))
    }
}

// main.rs
Commands::NewCommand(args) => {
    let cmd = NewCommand { args };
    let mut ctx = CommandContext::new(config);
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .build();
    let output = pipeline.execute(&cmd, &mut ctx).await?;
    if !output.success {
        std::process::exit(output.exit_code);
    }
}
```

### Migration Steps

1. **Keep old command working** - No need to change immediately
2. **Add new command** - Implement Command trait
3. **Test both** - Ensure backward compatibility
4. **Gradual replacement** - Replace old commands incrementally
5. **Remove old** - Once all commands migrated

## Examples

### Simple Command

```rust
pub struct EchoCommand {
    message: String,
}

#[async_trait]
impl Command for EchoCommand {
    fn name(&self) -> &str { "echo" }
    fn description(&self) -> &str { "Echo a message" }

    async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
        println!("{}", self.message);
        Ok(CommandOutput::success("Echoed message"))
    }
}
```

### Command with Validation

```rust
pub struct ValidatingCommand {
    value: i32,
}

#[async_trait]
impl Command for ValidatingCommand {
    fn name(&self) -> &str { "validate" }
    fn description(&self) -> &str { "Validates input" }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.value < 0 || self.value > 100 {
            anyhow::bail!("Value must be between 0 and 100");
        }
        Ok(())
    }

    async fn execute(&self, _ctx: &mut CommandContext) -> Result<CommandOutput> {
        Ok(CommandOutput::success_with_data(
            "Valid!",
            json!({ "value": self.value })
        ))
    }
}
```

### Command with Complex Logic

```rust
pub struct ModelListCommand;

#[async_trait]
impl Command for ModelListCommand {
    fn name(&self) -> &str { "models-list" }
    fn description(&self) -> &str { "List available models" }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        let model_manager = ModelManager::new(&ctx.config.models_dir);
        let models = model_manager.list_models().await?;

        if ctx.json_output {
            return Ok(CommandOutput::data(
                json!({ "models": models, "count": models.len() })
            ));
        }

        if models.is_empty() {
            return Ok(CommandOutput::warning(
                "No models found",
                Some(json!({ "models_dir": ctx.config.models_dir.display().to_string() }))
            ));
        }

        // Pretty print for human consumption
        println!("Found {} models:", models.len());
        for model in &models {
            println!("  - {}", model.name);
        }

        Ok(CommandOutput::success_with_data(
            format!("Found {} models", models.len()),
            json!({ "models": models, "count": models.len() })
        ))
    }
}
```

## Built-in Middleware

### LoggingMiddleware

Automatically logs command execution:
- Start time and execution ID
- Success/failure with duration
- Error details when commands fail

```rust
let pipeline = CommandPipeline::builder()
    .with_middleware(Box::new(LoggingMiddleware::new()))
    .build();
```

### MetricsMiddleware

Records command metrics:
- Execution duration
- Success/failure counts
- Exit codes

```rust
let pipeline = CommandPipeline::builder()
    .with_middleware(Box::new(MetricsMiddleware::new()))
    .build();
```

### Creating Custom Middleware

```rust
pub struct TimingMiddleware;

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn before(&self, ctx: &mut CommandContext) -> Result<()> {
        ctx.set_state("timing_start", Instant::now());
        Ok(())
    }

    async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()> {
        if let Some(start) = ctx.get_state::<Instant>("timing_start") {
            let duration = start.elapsed();
            println!("Command took: {:?}", duration);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "timing"
    }
}
```

## Testing

### Testing Commands

```rust
#[tokio::test]
async fn test_echo_command() {
    let cmd = EchoCommand {
        message: "test".to_string(),
    };

    let mut ctx = CommandContext::mock();
    let output = cmd.execute(&mut ctx).await.unwrap();

    assert!(output.success);
    assert_eq!(output.message, Some("Echoed message".to_string()));
}
```

### Testing with Pipeline

```rust
#[tokio::test]
async fn test_command_with_pipeline() {
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .build();

    let cmd = EchoCommand {
        message: "test".to_string(),
    };

    let mut ctx = CommandContext::mock();
    let output = pipeline.execute(&cmd, &mut ctx).await.unwrap();

    assert!(output.success);
}
```

### Testing Middleware

```rust
#[tokio::test]
async fn test_logging_middleware() {
    let middleware = LoggingMiddleware::new();
    let mut ctx = CommandContext::mock();

    middleware.before(&mut ctx).await.unwrap();
    let output = CommandOutput::success("test");
    middleware.after(&mut ctx, &Ok(output)).await.unwrap();
}
```

## Benefits

1. **Reduced Code Duplication**
   - Cross-cutting concerns handled once in middleware
   - Common validation patterns shared across commands
   - Consistent error handling

2. **Better Testing**
   - Commands testable in isolation
   - Mock context for unit tests
   - Middleware tested independently

3. **Enhanced Observability**
   - Automatic logging of all commands
   - Consistent metrics collection
   - Built-in audit trails

4. **Improved Error Handling**
   - Consistent error formatting
   - Better error messages with context
   - Centralized error recovery

5. **Better Developer Experience**
   - Clear patterns for new commands
   - Self-documenting via traits
   - Easier to understand command flow

## Performance

The new architecture adds minimal overhead:
- ~1-2ms per command execution (middleware)
- Zero-cost abstractions via traits
- Async-first for efficient I/O

Benchmark comparison (Phase 2):
- Old: ~12ms (command execution only)
- New: ~13-14ms (with logging + metrics middleware)
- Overhead: ~1-2ms (~8-16%)

## Future Enhancements

Planned for future phases:
- **Command Decorators**: @cache, @audit, @retry
- **Plugin System**: Dynamic command loading
- **Command Composition**: Pipelines of commands
- **Smart Validation**: Schema-based validation
- **Auto-completion**: Shell completion generation

## Troubleshooting

### Command Not Found

Ensure command is registered in `Commands` enum:
```rust
#[derive(Subcommand)]
pub enum Commands {
    MyCommand(my_command::MyCommandArgs),
}
```

### Middleware Not Running

Check middleware is added to pipeline:
```rust
let pipeline = CommandPipeline::builder()
    .with_middleware(Box::new(MyMiddleware))  // ← Add here
    .build();
```

### Context State Not Available

Ensure state is set before accessing:
```rust
ctx.set_state("key", value);  // Set
let value = ctx.get_state::<Type>("key");  // Get
```

## See Also

- **Examples**: `examples/cli_architecture.rs`
- **Planning Doc**: `.claude/plans/phase2-cli-architecture.md`
- **Core Config**: `src/core/config/README.md`
- **Legacy CLI**: `src/cli/` (old command files)