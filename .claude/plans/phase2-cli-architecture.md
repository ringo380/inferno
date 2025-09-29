# Phase 2: CLI Command Architecture

**Date**: 2025-09-29
**Status**: In Progress
**Complexity**: Complex

## Current State Analysis

### Statistics
- **Total CLI command files**: 46
- **Args structs**: 40+ (one per command)
- **Execute functions**: 40+ with signature `pub async fn execute(args: Args, config: &Config) -> Result<()>`
- **Config imports**: 19 commands explicitly use `Config`
- **Pattern repetition**: Very high - every command follows same structure

### Common Patterns Identified

1. **Args Structure**:
   ```rust
   #[derive(Args)]
   pub struct CommandArgs {
       #[command(subcommand)]
       pub command: CommandSubcommand,  // or direct fields
   }
   ```

2. **Execute Function**:
   ```rust
   pub async fn execute(args: CommandArgs, config: &Config) -> Result<()> {
       // Command logic
   }
   ```

3. **Imports**:
   ```rust
   use crate::config::Config;
   use anyhow::Result;
   use clap::Args;
   ```

4. **Main.rs Integration**:
   ```rust
   Commands::CommandName(args) => command_name::execute(args, &config).await?,
   ```

### Problems with Current Approach

1. **No Middleware**: Each command handles logging, metrics, validation independently
2. **Inconsistent Error Handling**: Different error messages and formats
3. **No Shared Context**: Config passed separately, no way to inject dependencies
4. **Testing Difficulty**: Hard to test commands in isolation
5. **Code Duplication**: Same setup/teardown code in every command
6. **No Command Hooks**: Can't run pre/post execution logic
7. **Limited Composability**: Commands can't easily call other commands

## Proposed Architecture

### Core Trait: Command

```rust
#[async_trait::async_trait]
pub trait Command: Send + Sync {
    /// Command name for identification
    fn name(&self) -> &str;

    /// Command description for help text
    fn description(&self) -> &str;

    /// Validate command arguments before execution
    async fn validate(&self, ctx: &CommandContext) -> Result<()> {
        Ok(())  // Default: no validation
    }

    /// Execute the command
    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput>;

    /// Command metadata (for help, completion, etc.)
    fn metadata(&self) -> CommandMetadata {
        CommandMetadata::default()
    }
}
```

### Command Context

```rust
pub struct CommandContext {
    /// Application configuration
    pub config: Arc<Config>,

    /// Core configuration (new builder-based)
    pub core_config: Arc<CoreConfig>,

    /// Command-specific arguments (dynamic)
    pub args: HashMap<String, serde_json::Value>,

    /// Shared state across middleware
    pub state: HashMap<String, Box<dyn Any + Send + Sync>>,

    /// Execution metadata
    pub execution_id: Uuid,
    pub start_time: Instant,

    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,

    /// Logger
    pub logger: Arc<dyn Logger>,
}
```

### Command Output

```rust
pub struct CommandOutput {
    /// Success indicator
    pub success: bool,

    /// Output data (for piping, JSON output, etc.)
    pub data: Option<serde_json::Value>,

    /// Human-readable message
    pub message: Option<String>,

    /// Exit code
    pub exit_code: i32,
}
```

### Middleware System

```rust
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Run before command execution
    async fn before(&self, ctx: &mut CommandContext) -> Result<()>;

    /// Run after command execution (even on error)
    async fn after(&self, ctx: &mut CommandContext, result: &Result<CommandOutput>) -> Result<()>;
}
```

### Built-in Middleware

1. **LoggingMiddleware**: Logs command execution start/end
2. **MetricsMiddleware**: Records command metrics
3. **ValidationMiddleware**: Runs command validation
4. **ErrorMiddleware**: Formats and logs errors consistently
5. **TimingMiddleware**: Measures command execution time
6. **AuditMiddleware**: Records command execution for compliance

### Command Pipeline

```rust
pub struct CommandPipeline {
    middleware: Vec<Box<dyn Middleware>>,
    error_handler: Box<dyn ErrorHandler>,
}

impl CommandPipeline {
    pub async fn execute(&self, command: &dyn Command, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Run pre-execution middleware
        for mw in &self.middleware {
            mw.before(ctx).await?;
        }

        // Validate command
        command.validate(ctx).await?;

        // Execute command
        let result = command.execute(ctx).await;

        // Run post-execution middleware (always)
        for mw in &self.middleware {
            mw.after(ctx, &result).await?;
        }

        // Handle errors
        match result {
            Ok(output) => Ok(output),
            Err(e) => self.error_handler.handle(e, ctx).await,
        }
    }
}
```

## Implementation Plan

### Step 1: Create Core Abstractions ✅
- [x] Create `src/interfaces/cli/command.rs` with Command trait
- [x] Create `src/interfaces/cli/context.rs` with CommandContext
- [x] Create `src/interfaces/cli/middleware.rs` with Middleware trait
- [x] Create `src/interfaces/cli/pipeline.rs` with CommandPipeline
- [x] Create `src/interfaces/cli/output.rs` with CommandOutput

### Step 2: Implement Built-in Middleware
- [ ] Create `src/interfaces/cli/middleware/logging.rs`
- [ ] Create `src/interfaces/cli/middleware/metrics.rs`
- [ ] Create `src/interfaces/cli/middleware/validation.rs`
- [ ] Create `src/interfaces/cli/middleware/error.rs`
- [ ] Create `src/interfaces/cli/middleware/timing.rs`
- [ ] Create `src/interfaces/cli/middleware/audit.rs`

### Step 3: Refactor Example Commands
- [ ] Refactor `src/cli/models.rs` to use new architecture
- [ ] Refactor `src/cli/run.rs` to use new architecture
- [ ] Refactor `src/cli/serve.rs` to use new architecture
- [ ] Document refactoring pattern in README

### Step 4: Create Migration Guide
- [ ] Document how to convert old commands to new architecture
- [ ] Create examples of common patterns
- [ ] Add testing guide for new command architecture

### Step 5: Gradual Migration
- [ ] Maintain backward compatibility with old command style
- [ ] Migrate high-traffic commands first (run, serve, models)
- [ ] Migrate remaining commands incrementally

## Benefits

### 1. Reduced Code Duplication
- Middleware handles cross-cutting concerns once
- Common validation logic in one place
- Consistent error handling

### 2. Better Testing
- Commands can be tested in isolation
- Mock CommandContext for unit tests
- Middleware can be tested independently

### 3. Enhanced Observability
- Automatic logging of all commands
- Consistent metrics collection
- Built-in audit trails

### 4. Improved Error Handling
- Consistent error formatting
- Better error messages with context
- Centralized error recovery

### 5. Better Developer Experience
- Clear patterns for adding new commands
- Self-documenting via traits
- Easier to understand command flow

### 6. Command Composition
- Commands can call other commands
- Build complex workflows from simple commands
- Reusable command logic

## Backward Compatibility

### Old Style (still works)
```rust
// src/cli/old_command.rs
pub async fn execute(args: OldCommandArgs, config: &Config) -> Result<()> {
    // Old logic
}

// main.rs
Commands::OldCommand(args) => old_command::execute(args, &config).await?,
```

### New Style (recommended)
```rust
// src/cli/new_command.rs
pub struct NewCommand {
    args: NewCommandArgs,
}

#[async_trait::async_trait]
impl Command for NewCommand {
    fn name(&self) -> &str { "new-command" }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // New logic with context
        Ok(CommandOutput::success("Command completed"))
    }
}

// main.rs
Commands::NewCommand(args) => {
    let cmd = NewCommand { args };
    let mut ctx = CommandContext::new(config);
    pipeline.execute(&cmd, &mut ctx).await?;
}
```

## Example: Models Command Refactored

### Before (current)
```rust
// src/cli/models.rs
pub async fn execute(args: ModelsArgs, config: &Config) -> Result<()> {
    let model_manager = ModelManager::new(&config.models_dir);

    match args.command {
        ModelsCommand::List => {
            info!("Scanning for models...");
            let models = model_manager.list_models().await?;
            // Display logic
        }
        // ...
    }
    Ok(())
}
```

### After (new architecture)
```rust
// src/interfaces/cli/commands/models/list.rs
pub struct ListModelsCommand;

#[async_trait::async_trait]
impl Command for ListModelsCommand {
    fn name(&self) -> &str { "models-list" }
    fn description(&self) -> &str { "List all available models" }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        // Logging middleware handles logging automatically
        // Metrics middleware records execution time

        let model_manager = ModelManager::new(&ctx.config.models_dir);
        let models = model_manager.list_models().await?;

        if models.is_empty() {
            return Ok(CommandOutput::warning(
                "No models found",
                json!({ "count": 0 })
            ));
        }

        Ok(CommandOutput::success_with_data(
            format!("Found {} models", models.len()),
            json!({ "models": models, "count": models.len() })
        ))
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_list_models_command() {
    let cmd = ListModelsCommand;
    let mut ctx = CommandContext::mock();

    let result = cmd.execute(&mut ctx).await;
    assert!(result.is_ok());
}
```

### Middleware Tests
```rust
#[tokio::test]
async fn test_logging_middleware() {
    let middleware = LoggingMiddleware::new();
    let mut ctx = CommandContext::mock();

    middleware.before(&mut ctx).await.unwrap();
    // Assert logs were written
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_command_pipeline() {
    let pipeline = CommandPipeline::builder()
        .with_logging()
        .with_metrics()
        .build();

    let cmd = ListModelsCommand;
    let mut ctx = CommandContext::mock();

    let output = pipeline.execute(&cmd, &mut ctx).await.unwrap();
    assert!(output.success);
}
```

## Next Steps

1. Implement core abstractions in `src/interfaces/cli/`
2. Build essential middleware (logging, metrics, error handling)
3. Refactor 3 example commands to validate approach
4. Create migration guide and documentation
5. Begin gradual migration of remaining 43 commands

## Success Metrics

- [ ] Command trait implemented and tested
- [ ] 6 middleware components functional
- [ ] 3 commands successfully refactored
- [ ] Documentation complete
- [ ] Zero regressions in existing commands
- [ ] Build time unchanged or improved
- [ ] Test coverage >60% for new code