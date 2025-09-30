# CLI Command Migration Guide

**Version**: 1.0
**Date**: 2025-09-29
**Status**: Active Guide for Phase 2 Refactoring

## Overview

This guide documents the process of migrating existing CLI commands from the old pattern to the new Command trait architecture. It captures lessons learned from the first two successful migrations (`models` and `run`) and provides patterns for future migrations.

## Table of Contents

1. [Why Migrate?](#why-migrate)
2. [Architecture Overview](#architecture-overview)
3. [When to Migrate (and When Not To)](#when-to-migrate)
4. [Migration Patterns](#migration-patterns)
5. [Step-by-Step Process](#step-by-step-process)
6. [Common Scenarios](#common-scenarios)
7. [Testing Strategy](#testing-strategy)
8. [Lessons Learned](#lessons-learned)

---

## Why Migrate?

### Benefits of New Architecture

**Consistency**
- All commands follow the same validate → execute pattern
- Predictable behavior across 46+ commands

**Testability**
- Commands are isolated and unit-testable
- Mock CommandContext for testing
- No dependency on global state

**Middleware Support**
- Automatic logging for all commands
- Metrics collection without code duplication
- Future middleware: rate limiting, audit logging, etc.

**Structured Output**
- Consistent JSON and human-readable modes
- Machine-parseable results
- Easy integration with other tools

**Error Handling**
- Validation before execution
- Structured error messages
- Consistent exit codes

**Developer Experience**
- Clear separation of concerns
- Self-documenting code (name(), description())
- Examples demonstrate patterns

---

## Architecture Overview

### Core Components

```rust
// Command trait - all commands implement this
#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn validate(&self, ctx: &CommandContext) -> Result<()>;
    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput>;
}

// CommandContext - shared state and configuration
pub struct CommandContext {
    pub config: Config,
    pub verbose: bool,
    pub dry_run: bool,
    pub json_output: bool,
    pub start_time: Instant,
}

// CommandOutput - structured results
pub struct CommandOutput {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
    pub exit_code: i32,
}

// CommandPipeline - orchestrates execution with middleware
pub struct CommandPipeline {
    middleware: Vec<Box<dyn Middleware>>,
}
```

### Execution Flow

```
User Input
    ↓
CommandPipeline
    ↓
Middleware Stack (before)
  - LoggingMiddleware
  - MetricsMiddleware
  - [Future middleware]
    ↓
Command::validate()  ← Fail fast with validation errors
    ↓
Command::execute()   ← Actual work happens here
    ↓
Middleware Stack (after)
    ↓
CommandOutput → User
```

---

## When to Migrate (and When Not To)

### ✅ Good Candidates for Migration

**Simple Commands** (like `models`)
- CRUD operations
- Query commands (list, info, show)
- One-shot operations with clear completion

**Complex One-Shot Commands** (like `run`)
- Inference execution
- File processing
- Data transformation
- Batch operations with defined completion

**Validation-Heavy Commands**
- Commands with many parameters
- Complex validation rules
- Multiple input formats

### ⚠️ Difficult Candidates

**Long-Running Services** (like `serve`)
- HTTP servers
- WebSocket servers
- Background daemons
- Process managers

**Recommendation**: Leave server commands in original pattern. They need specialized lifecycle management that doesn't fit the Command trait pattern.

**Interactive Commands**
- TUI applications
- REPL loops
- Watch modes

**Recommendation**: Consider a specialized `InteractiveCommand` trait if needed.

---

## Migration Patterns

### Pattern 1: Simple Command (like models list)

**Characteristics:**
- Single responsibility
- Clear input/output
- No streaming
- Quick execution

**Template:**
```rust
pub struct SimpleCommand {
    config: Config,
    // Command-specific fields
}

#[async_trait]
impl Command for SimpleCommand {
    fn name(&self) -> &str { "command name" }

    fn description(&self) -> &str { "what it does" }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        // Validate inputs
        if self.field.is_empty() {
            anyhow::bail!("field cannot be empty");
        }
        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Do the work
        let result = do_something().await?;

        // Human-readable output
        if !ctx.json_output {
            println!("Result: {}", result);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Success message",
            json!({ "result": result })
        ))
    }
}
```

### Pattern 2: Command with Subcommands (like models)

**Characteristics:**
- Multiple related operations
- Shared context
- Different validation per subcommand

**Template:**
```rust
// One command struct per subcommand
pub struct CommandList { config: Config }
pub struct CommandInfo { config: Config, id: String }
pub struct CommandDelete { config: Config, id: String }

// Each implements Command trait separately
#[async_trait]
impl Command for CommandList {
    // ... implementation
}

#[async_trait]
impl Command for CommandInfo {
    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.id.is_empty() {
            anyhow::bail!("ID required");
        }
        Ok(())
    }
    // ... rest
}
```

**Advantages:**
- Each subcommand is independently testable
- Clear separation of validation logic
- Easy to add new subcommands

### Pattern 3: Complex Command with Modes (like run)

**Characteristics:**
- Multiple execution modes (single/batch/streaming)
- Conditional logic based on flags
- Shared setup, different execution paths

**Template:**
```rust
pub struct ComplexCommand {
    config: Config,
    mode: ExecutionMode,
    // ... other fields
}

impl ComplexCommand {
    // Private helper methods for each mode
    async fn execute_mode_a(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Mode A implementation
    }

    async fn execute_mode_b(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Mode B implementation
    }
}

#[async_trait]
impl Command for ComplexCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Dispatch to appropriate mode
        match self.mode {
            ExecutionMode::A => self.execute_mode_a(ctx).await,
            ExecutionMode::B => self.execute_mode_b(ctx).await,
        }
    }
}
```

**Key Points:**
- Keep mode-specific logic in private methods
- Share validation and setup code
- Return consistent CommandOutput structure

### Pattern 4: Streaming Output

**Characteristics:**
- Real-time token/chunk output
- Progress indication
- Still returns structured result at end

**Template:**
```rust
async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
    let mut stream = create_stream().await?;
    let mut accumulated = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        // Stream to stdout if not JSON mode
        if !ctx.json_output {
            print!("{}", chunk);
            std::io::stdout().flush()?;
        }

        accumulated.push_str(&chunk);
    }

    // Return complete result
    Ok(CommandOutput::success_with_data(
        "Streaming complete",
        json!({ "output": accumulated })
    ))
}
```

---

## Step-by-Step Process

### Phase 1: Analysis (15-30 minutes)

1. **Read the original command**
   - Understand what it does
   - Identify subcommands if any
   - Note dependencies and complexity

2. **Identify the pattern**
   - Simple command?
   - Multiple subcommands?
   - Multiple modes?
   - Streaming output?

3. **Check for blockers**
   - Is it a long-running server? (consider skipping)
   - Does it have complex interactive elements?
   - Are there circular dependencies?

### Phase 2: Create v2 File (1-2 hours)

1. **Create `{command}_v2.rs`**
   ```bash
   touch src/cli/{command}_v2.rs
   ```

2. **Add boilerplate**
   ```rust
   //! {Command} Command - New Architecture
   //!
   //! This module demonstrates the migration of the {command} command
   //! to the new CLI architecture.

   use crate::config::Config;
   use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
   use anyhow::Result;
   use async_trait::async_trait;
   use serde_json::json;
   ```

3. **Define command struct(s)**
   - One struct per subcommand OR
   - One struct with mode enum

4. **Implement Command trait**
   - `name()` and `description()`
   - `validate()` with all input checks
   - `execute()` with actual logic

5. **Add helper functions**
   - Keep execute() clean
   - Extract complex logic to private methods

### Phase 3: Integration (30 minutes)

1. **Add to mod.rs**
   ```rust
   pub mod {command};
   pub mod {command}_v2;  // New architecture implementation
   ```

2. **Test compilation**
   ```bash
   cargo check --lib
   ```

3. **Fix any errors**
   - Usually related to missing imports
   - Or incorrect method signatures

### Phase 4: Example (1 hour)

1. **Create example file**
   ```bash
   touch examples/{command}_v2_example.rs
   ```

2. **Demonstrate key features**
   - Basic usage
   - JSON output mode
   - Error handling
   - All major subcommands/modes

3. **Test the example**
   ```bash
   cargo run --example {command}_v2_example
   ```

### Phase 5: Documentation (30 minutes)

1. **Update refactoring plan**
   - Mark command as migrated
   - Add any lessons learned

2. **Add inline documentation**
   - Document complex logic
   - Explain validation rules
   - Note any caveats

3. **Commit with descriptive message**
   ```bash
   git add src/cli/{command}_v2.rs examples/{command}_v2_example.rs
   git commit -m "refactor(cli): migrate {command} to new architecture"
   ```

---

## Common Scenarios

### Scenario 1: Command with File I/O

**Old Pattern:**
```rust
let content = tokio::fs::read_to_string(&path).await?;
println!("{}", content);
```

**New Pattern:**
```rust
async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
    let content = tokio::fs::read_to_string(&self.path).await?;

    if !ctx.json_output {
        println!("{}", content);
    }

    Ok(CommandOutput::success_with_data(
        format!("Read {} bytes", content.len()),
        json!({ "content": content, "size": content.len() })
    ))
}
```

### Scenario 2: Command with Progress

**Use structured output data:**
```rust
let mut progress = 0;
for item in items {
    process(item).await?;
    progress += 1;

    if !ctx.json_output {
        println!("Progress: {}/{}", progress, total);
    }
}

Ok(CommandOutput::success_with_data(
    "Processing complete",
    json!({
        "total": total,
        "processed": progress,
        "success_rate": (progress as f64 / total as f64) * 100.0
    })
))
```

### Scenario 3: Command with Optional Output File

**Pattern:**
```rust
let result = compute_result().await?;

if let Some(output_path) = &self.output {
    tokio::fs::write(output_path, &result).await?;
    info!("Output written to: {}", output_path.display());
} else if !ctx.json_output {
    println!("{}", result);
}

Ok(CommandOutput::success_with_data(
    "Computation complete",
    json!({
        "result": result,
        "output_file": self.output.as_ref().map(|p| p.display().to_string())
    })
))
```

### Scenario 4: Command with Metrics

**Use backend metrics:**
```rust
let start = Instant::now();
let response = backend.infer(input, &params).await?;
let elapsed = start.elapsed();

let metrics = backend.get_metrics();

Ok(CommandOutput::success_with_data(
    format!("Inference completed in {:.2}s", elapsed.as_secs_f64()),
    json!({
        "response": response,
        "elapsed_ms": elapsed.as_millis(),
        "metrics": metrics.map(|m| json!({
            "tokens_per_second": m.tokens_per_second,
            "total_tokens": m.total_tokens,
        }))
    })
))
```

---

## Testing Strategy

### Unit Tests

**Test command validation:**
```rust
#[tokio::test]
async fn test_validation_empty_field() {
    let cmd = MyCommand::new("".to_string());
    let ctx = CommandContext::new(Config::default());

    let result = cmd.validate(&ctx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}
```

**Test command execution:**
```rust
#[tokio::test]
async fn test_execute_success() {
    let cmd = MyCommand::new("test".to_string());
    let mut ctx = CommandContext::new(Config::default());

    let output = cmd.execute(&mut ctx).await.unwrap();
    assert!(output.success);
    assert!(output.data.is_some());
}
```

### Integration Tests

**Test with pipeline:**
```rust
#[tokio::test]
async fn test_command_with_pipeline() {
    let pipeline = CommandPipeline::builder()
        .with_middleware(Box::new(LoggingMiddleware::new()))
        .build();

    let cmd = MyCommand::new("test".to_string());
    let mut ctx = CommandContext::new(Config::default());

    let output = pipeline.execute(Box::new(cmd), &mut ctx).await.unwrap();
    assert!(output.success);
}
```

### Example-Based Testing

**Run examples as integration tests:**
```bash
cargo run --example {command}_v2_example
```

---

## Lessons Learned

### From models Migration

**What Went Well:**
- Simple commands are perfect for this pattern
- Subcommand separation makes code very clean
- Validation catches errors early

**Challenges:**
- Need to handle both JSON and human-readable output
- Helper functions for formatting (format_size, format_params)

**Key Insight:** Keep subcommands as separate Command implementations rather than one big enum-based command.

### From run Migration

**What Went Well:**
- Complex commands can be broken into modes
- Private helper methods keep execute() clean
- Streaming fits well with CommandOutput

**Challenges:**
- Batch processing integration took careful thought
- Needed comprehensive validation for many parameters
- Streaming while maintaining structured output

**Key Insight:** Use private methods for mode-specific logic. The execute() method should be a clean dispatcher.

### What NOT to Migrate (Yet)

**Server Commands:**
- serve.rs is 502 lines of HTTP server logic
- Doesn't fit the "execute and return" pattern
- Needs specialized lifecycle management
- **Decision**: Skip for now, revisit with specialized pattern

**Recommendation:** Focus on commands that benefit most from the pattern. Don't force-fit everything.

---

## Next Commands to Migrate

### Recommended Priority Order

**High Priority (Easy Wins):**
1. `validate` - Simple validation command
2. `bench` - One-shot benchmark execution
3. `config` - Configuration management
4. `gpu` - GPU information queries

**Medium Priority (Good Examples):**
5. `convert` - Model format conversion
6. `cache` - Cache management operations
7. `metrics` - Metrics querying
8. `repo` - Repository operations

**Lower Priority (Complex, but valuable):**
9. `batch` - Already has infrastructure, but coordinate with batch-queue
10. `deployment` - Deployment automation
11. `backup_recovery` - Backup operations (large file, split first)

**Skip for Now:**
12. `serve` - Server lifecycle doesn't fit pattern
13. `tui` - Interactive, needs specialized pattern
14. `dashboard` - Large file (3,608 lines), split first

---

## Migration Checklist

Use this checklist for each command migration:

- [ ] Analyzed original command structure
- [ ] Identified appropriate pattern
- [ ] Created `{command}_v2.rs` file
- [ ] Implemented all subcommands/modes
- [ ] Added comprehensive validation
- [ ] Handled JSON and human-readable output
- [ ] Added helper functions as needed
- [ ] Tested compilation (`cargo check --lib`)
- [ ] Created example file
- [ ] Tested example execution
- [ ] Added unit tests
- [ ] Updated mod.rs
- [ ] Updated refactoring plan
- [ ] Committed with descriptive message

---

## Questions and Support

**Questions about patterns?** Check the examples:
- `examples/cli_architecture.rs` - Basic patterns
- `examples/models_v2_example.rs` - Subcommand pattern
- `examples/run_v2_example.rs` - Complex multi-mode pattern

**Unsure if a command fits?** Ask yourself:
1. Does it have clear start and end? → Migrate
2. Is it a long-running service? → Skip for now
3. Is it highly interactive? → Consider specialized pattern

**Need help?** Review this guide and the completed migrations for patterns.

---

## Version History

- **v1.0** (2025-09-29): Initial guide based on models and run migrations