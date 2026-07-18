#![allow(dead_code)]
// User-friendly error handling and setup guidance module

use crate::config::Config;
use crate::interfaces::cli::{Command, CommandContext, CommandOutput};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::info;

/// User-friendly error handling and setup guidance
pub struct HelpSystem;

impl HelpSystem {
    /// Provide helpful error messages and setup guidance
    pub fn handle_error(error: &anyhow::Error) -> String {
        let error_msg = error.to_string().to_lowercase();

        // Check for common error patterns and provide helpful guidance
        if error_msg.contains("no such file or directory") {
            Self::handle_file_not_found_error(&error_msg)
        } else if error_msg.contains("permission denied") {
            Self::handle_permission_error(&error_msg)
        } else if error_msg.contains("network") || error_msg.contains("connection") {
            Self::handle_network_error(&error_msg)
        } else if error_msg.contains("config") || error_msg.contains("configuration") {
            Self::handle_config_error(&error_msg)
        } else if error_msg.contains("model") && error_msg.contains("not found") {
            Self::handle_model_not_found_error(&error_msg)
        } else if error_msg.contains("authentication") || error_msg.contains("unauthorized") {
            Self::handle_auth_error(&error_msg)
        } else if error_msg.contains("disk") || error_msg.contains("space") {
            Self::handle_disk_space_error(&error_msg)
        } else {
            Self::handle_generic_error(error)
        }
    }

    fn handle_file_not_found_error(error_msg: &str) -> String {
        let mut message = String::from("❌ File or directory not found.\n\n");

        if error_msg.contains("models") {
            message.push_str("💡 This usually means:\n");
            message.push_str("   • No models directory has been configured\n");
            message.push_str("   • The specified model file doesn't exist\n\n");
            message.push_str("🔧 Try these solutions:\n");
            message.push_str("   1. Check your models directory:\n");
            message.push_str("      inferno models list\n\n");
            message.push_str("   2. Install a model first:\n");
            message.push_str("      inferno models install TheBloke/Llama-2-7B-GGUF\n\n");
            message.push_str("   3. Configure your models directory:\n");
            message.push_str("      inferno config set models_dir /path/to/models\n");
        } else if error_msg.contains("config") {
            message.push_str("💡 Configuration file not found.\n\n");
            message.push_str("🔧 Initialize configuration:\n");
            message.push_str("   inferno config init\n\n");
            message.push_str("   Or create a basic config:\n");
            message.push_str("   mkdir -p ~/.config/inferno\n");
            message.push_str("   inferno config show > ~/.config/inferno/config.toml\n");
        } else {
            message.push_str("💡 Check that the file path is correct and the file exists.\n");
        }

        message
    }

    fn handle_permission_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Permission denied.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • You don't have permission to access the file/directory\n");
        message.push_str("   • The file is owned by another user\n");
        message.push_str("   • SELinux or similar security policies are blocking access\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Check file permissions:\n");
        message.push_str("      ls -la [file-path]\n\n");
        message.push_str("   2. Fix permissions if you own the file:\n");
        message.push_str("      chmod 644 [file-path]  # for files\n");
        message.push_str("      chmod 755 [directory]  # for directories\n\n");
        message.push_str("   3. Use your home directory instead:\n");
        message.push_str("      inferno config set models_dir ~/models\n\n");
        message.push_str("   4. Run with appropriate permissions (be careful!):\n");
        message.push_str("      sudo inferno [command]  # only if necessary\n");

        message
    }

    fn handle_network_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Network connection error.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • No internet connection\n");
        message.push_str("   • HuggingFace (or the model host) is unreachable\n");
        message.push_str("   • Firewall is blocking the connection\n");
        message.push_str("   • Proxy configuration issues\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Check your internet connection:\n");
        message.push_str("      ping huggingface.co\n\n");
        message.push_str("   2. Test model discovery (exercises the network path):\n");
        message.push_str("      inferno models search llama\n\n");
        message.push_str("   3. Check proxy settings if behind a corporate firewall:\n");
        message.push_str("      export HTTP_PROXY=http://proxy.company.com:8080\n");
        message.push_str("      export HTTPS_PROXY=http://proxy.company.com:8080\n\n");
        message.push_str("   4. Try again later (the host might be temporarily down)\n");

        message
    }

    fn handle_config_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Configuration error.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • Configuration file is missing or corrupted\n");
        message.push_str("   • Invalid configuration values\n");
        message.push_str("   • Missing required configuration\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Initialize configuration:\n");
        message.push_str("      inferno config init\n\n");
        message.push_str("   2. Check current configuration:\n");
        message.push_str("      inferno config show\n\n");
        message.push_str("   3. Reset to default configuration:\n");
        message.push_str("      inferno config reset --confirm\n\n");
        message.push_str("   4. Set specific configuration values:\n");
        message.push_str("      inferno config set models_dir ~/models\n");
        message.push_str("      inferno config set log_level info\n");

        message
    }

    fn handle_model_not_found_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Model not found.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • The model name or path is incorrect\n");
        message.push_str("   • The model file isn't in your models directory\n");
        message.push_str("   • The HuggingFace repo ID doesn't exist\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. List the models you already have:\n");
        message.push_str("      inferno models list\n\n");
        message.push_str("   2. Search HuggingFace for a model:\n");
        message.push_str("      inferno models search llama\n");
        message.push_str("      inferno models search \"mistral instruct\" --task text-generation\n\n");
        message.push_str("   3. Install a model by HuggingFace repo ID or direct URL:\n");
        message.push_str("      inferno models install TheBloke/Llama-2-7B-GGUF\n");
        message.push_str("      inferno models install TheBloke/Mistral-7B-Instruct-v0.2-GGUF\n\n");
        message.push_str("   4. Inspect a specific model:\n");
        message.push_str("      inferno models info [model-name]\n");

        message
    }

    fn handle_auth_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Authentication error.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • Missing API key or token\n");
        message.push_str("   • Invalid or expired credentials\n");
        message.push_str("   • Insufficient permissions for private models\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Set up Hugging Face authentication:\n");
        message.push_str("      export HUGGINGFACE_TOKEN=hf_your_token_here\n\n");
        message.push_str("   2. Try a public model instead of a gated one:\n");
        message.push_str("      inferno models search llama\n\n");
        message.push_str("   3. Check your API tokens:\n");
        message.push_str("      • Hugging Face: https://huggingface.co/settings/tokens\n");
        message.push_str("      • Make sure tokens have appropriate permissions\n");

        message
    }

    fn handle_disk_space_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Disk space error.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • Not enough disk space for model download\n");
        message.push_str("   • Cache directory is full\n");
        message.push_str("   • Temporary space exhausted\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Clear the model cache:\n");
        message.push_str("      inferno cache clear\n\n");
        message.push_str("   2. Remove unused models (delete the files directly):\n");
        message.push_str("      inferno models list        # find the file paths\n");
        message.push_str("      rm [models_dir]/[unused-model].gguf\n\n");
        message.push_str("   3. Check disk space:\n");
        message.push_str("      df -h\n\n");
        message.push_str("   4. Move models directory to a larger disk:\n");
        message.push_str("      inferno config set models_dir /path/to/larger/disk/models\n\n");
        message.push_str("   5. Check cache usage:\n");
        message.push_str("      inferno cache stats\n");
        message.push_str("      du -sh ~/.cache/inferno\n");

        message
    }

    fn handle_generic_error(error: &anyhow::Error) -> String {
        let mut message = String::from("❌ An error occurred.\n\n");

        message.push_str(&format!("Error: {}\n\n", error));

        message.push_str("🔧 General troubleshooting steps:\n");
        message.push_str("   1. Check system status:\n");
        message.push_str("      inferno config show\n");
        message.push_str("      inferno models list\n\n");
        message.push_str("   2. Clear the model cache:\n");
        message.push_str("      inferno cache clear\n\n");
        message.push_str("   3. Check logs for more details:\n");
        message.push_str("      INFERNO_LOG_LEVEL=debug inferno [your-command]\n\n");
        message.push_str("   4. Reset configuration if needed:\n");
        message.push_str("      inferno config reset --confirm\n\n");
        message.push_str("   5. Get help:\n");
        message.push_str("      inferno --help\n");
        message.push_str("      inferno [command] --help\n");

        message
    }

    /// Check prerequisites for common commands
    pub fn check_prerequisites(command: &str) -> Option<String> {
        match command {
            "serve" => Self::check_serve_prerequisites(),
            "run" => Self::check_run_prerequisites(),
            _ => None,
        }
    }

    fn check_serve_prerequisites() -> Option<String> {
        // Check if models are available
        if !Self::has_models_available() {
            return Some(
                "⚠️  No models available to serve.\n\n\
                🔧 Setup required:\n\
                   1. Install a model first:\n\
                      inferno models install TheBloke/Llama-2-7B-GGUF\n\n\
                   2. Or check existing models:\n\
                      inferno models list\n\n\
                   3. Verify models directory:\n\
                      inferno config get models_dir\n"
                    .to_string(),
            );
        }

        None
    }

    fn check_run_prerequisites() -> Option<String> {
        // Check if models and configuration are ready
        if !Self::has_models_available() {
            return Some(
                "⚠️  No models available for inference.\n\n\
                🔧 Setup required:\n\
                   1. Install a model:\n\
                      inferno models install TheBloke/Llama-2-7B-GGUF\n\n\
                   2. Or specify model path:\n\
                      inferno run --model /path/to/model.gguf --prompt \"Hello\"\n\n\
                   3. List available models:\n\
                      inferno models list\n"
                    .to_string(),
            );
        }

        None
    }

    // Helper functions (these would check actual system state in real implementation)
    fn has_network_connectivity() -> bool {
        // In real implementation: try to ping a reliable host
        true // Assume network is available for now
    }

    fn has_models_available() -> bool {
        // In real implementation: check models directory and package database
        true // Assume models might be available
    }

    /// Provide command usage examples
    pub fn get_usage_examples(command: &str) -> String {
        match command {
            "install" => Self::install_examples(),
            "search" => Self::search_examples(),
            "list" => Self::list_examples(),
            _ => Self::general_examples(),
        }
    }

    fn install_examples() -> String {
        "📚 Model Install Examples:\n\n\
        # Install a GGUF model by HuggingFace repo ID\n\
        inferno models install TheBloke/Llama-2-7B-GGUF\n\
        inferno models install TheBloke/Mistral-7B-Instruct-v0.2-GGUF\n\n\
        # Pick a specific file from a repo, or rename it locally\n\
        inferno models install TheBloke/Llama-2-7B-GGUF --file llama-2-7b.Q4_K_M.gguf\n\
        inferno models install TheBloke/Llama-2-7B-GGUF --name llama2-7b\n\n\
        # Install from a direct HTTPS URL\n\
        inferno models install https://example.com/models/my-model.gguf\n"
            .to_string()
    }

    fn search_examples() -> String {
        "📚 Model Search Examples:\n\n\
        # Basic search on HuggingFace\n\
        inferno models search llama\n\
        inferno models search \"mistral instruct\"\n\n\
        # Filter by task or limit results\n\
        inferno models search llama --task text-generation\n\
        inferno models search gemma --limit 25\n"
            .to_string()
    }

    fn list_examples() -> String {
        "📚 Model Listing Examples:\n\n\
        # List local models\n\
        inferno models list\n\n\
        # Inspect a specific model\n\
        inferno models info [model-name]\n\
        inferno models validate [model-file]\n\
        inferno models stats\n"
            .to_string()
    }

    fn general_examples() -> String {
        "📚 General Examples:\n\n\
        # Quick start\n\
        inferno models install TheBloke/Llama-2-7B-GGUF\n\
        inferno models list\n\
        inferno run --model llama-2-7b --prompt \"Hello!\"\n\n\
        # Serve an OpenAI-compatible API\n\
        inferno serve\n\n\
        # Configuration\n\
        inferno config show\n\
        inferno config set models_dir ~/models\n\n\
        # Help\n\
        inferno --help\n\
        inferno [command] --help\n"
            .to_string()
    }
}

// ============================================================================
// HandleError - Provide helpful error guidance
// ============================================================================

/// Provide helpful error messages and setup guidance
pub struct HandleError {
    config: Config,
    error_message: String,
}

impl HandleError {
    pub fn new(config: Config, error_message: String) -> Self {
        Self {
            config,
            error_message,
        }
    }
}

#[async_trait]
impl Command for HandleError {
    fn name(&self) -> &str {
        "help error"
    }

    fn description(&self) -> &str {
        "Provide helpful error messages and setup guidance"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.error_message.is_empty() {
            anyhow::bail!("Error message cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Providing help for error: {}", self.error_message);

        // Create a mock error to pass to HelpSystem
        let error = anyhow::anyhow!("{}", self.error_message);
        let guidance = HelpSystem::handle_error(&error);

        // Human-readable output
        if !ctx.json_output {
            print!("{}", guidance);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            "Error guidance provided",
            json!({
                "error": self.error_message,
                "guidance": guidance,
            }),
        ))
    }
}

// ============================================================================
// CheckPrerequisites - Check command prerequisites
// ============================================================================

/// Check prerequisites for a command
pub struct CheckPrerequisites {
    config: Config,
    command_name: String,
}

impl CheckPrerequisites {
    pub fn new(config: Config, command_name: String) -> Self {
        Self {
            config,
            command_name,
        }
    }
}

#[async_trait]
impl Command for CheckPrerequisites {
    fn name(&self) -> &str {
        "help prerequisites"
    }

    fn description(&self) -> &str {
        "Check prerequisites for a command"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.command_name.is_empty() {
            anyhow::bail!("Command name cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Checking prerequisites for: {}", self.command_name);

        let prereq_check = HelpSystem::check_prerequisites(&self.command_name);

        let (has_prereqs, message) = match prereq_check {
            Some(msg) => (true, msg),
            None => (
                false,
                format!("✓ No prerequisites for '{}'", self.command_name),
            ),
        };

        // Human-readable output
        if !ctx.json_output {
            if has_prereqs {
                println!("⚠️  Prerequisites for '{}':", self.command_name);
                print!("{}", message);
            } else {
                println!("{}", message);
            }
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            if has_prereqs {
                "Prerequisites found"
            } else {
                "No prerequisites"
            },
            json!({
                "command": self.command_name,
                "has_prerequisites": has_prereqs,
                "message": message,
            }),
        ))
    }
}

// ============================================================================
// GetExamples - Get usage examples for a command
// ============================================================================

/// Get usage examples for a command
pub struct GetExamples {
    config: Config,
    command_name: String,
}

impl GetExamples {
    pub fn new(config: Config, command_name: String) -> Self {
        Self {
            config,
            command_name,
        }
    }
}

#[async_trait]
impl Command for GetExamples {
    fn name(&self) -> &str {
        "help examples"
    }

    fn description(&self) -> &str {
        "Get usage examples for a command"
    }

    async fn validate(&self, _ctx: &CommandContext) -> Result<()> {
        if self.command_name.is_empty() {
            anyhow::bail!("Command name cannot be empty");
        }

        Ok(())
    }

    async fn execute(&self, ctx: &mut CommandContext) -> Result<CommandOutput> {
        info!("Getting examples for: {}", self.command_name);

        let examples = HelpSystem::get_usage_examples(&self.command_name);

        // Human-readable output
        if !ctx.json_output {
            print!("{}", examples);
        }

        // Structured output
        Ok(CommandOutput::success_with_data(
            format!("Examples for '{}'", self.command_name),
            json!({
                "command": self.command_name,
                "examples": examples,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_error_validation_empty() {
        let config = Config::default();
        let cmd = HandleError::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Error message cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_handle_error_validation_valid() {
        let config = Config::default();
        let cmd = HandleError::new(config.clone(), "no such file or directory".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_prerequisites_validation_empty() {
        let config = Config::default();
        let cmd = CheckPrerequisites::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Command name cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_check_prerequisites_validation_valid() {
        let config = Config::default();
        let cmd = CheckPrerequisites::new(config.clone(), "install".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_examples_validation_empty() {
        let config = Config::default();
        let cmd = GetExamples::new(config.clone(), String::new());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Command name cannot be empty")
        );
    }

    #[tokio::test]
    async fn test_get_examples_validation_valid() {
        let config = Config::default();
        let cmd = GetExamples::new(config.clone(), "install".to_string());
        let ctx = CommandContext::new(config);

        let result = cmd.validate(&ctx).await;
        assert!(result.is_ok());
    }
}
