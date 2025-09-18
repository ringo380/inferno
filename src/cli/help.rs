// User-friendly error handling and setup guidance module
// No external dependencies needed for static help content

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
        } else if error_msg.contains("repository") || error_msg.contains("repo") {
            Self::handle_repository_error(&error_msg)
        } else if error_msg.contains("model") && error_msg.contains("not found") {
            Self::handle_model_not_found_error(&error_msg)
        } else if error_msg.contains("authentication") || error_msg.contains("unauthorized") {
            Self::handle_auth_error(&error_msg)
        } else if error_msg.contains("disk") || error_msg.contains("space") {
            Self::handle_disk_space_error(&error_msg)
        } else if error_msg.contains("dependency") || error_msg.contains("dependencies") {
            Self::handle_dependency_error(&error_msg)
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
            message.push_str("      inferno install microsoft/DialoGPT-medium\n\n");
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
        message.push_str("   • Repository server is down\n");
        message.push_str("   • Firewall is blocking the connection\n");
        message.push_str("   • Proxy configuration issues\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Check your internet connection:\n");
        message.push_str("      ping google.com\n\n");
        message.push_str("   2. Test repository connectivity:\n");
        message.push_str("      inferno repo test huggingface\n\n");
        message.push_str("   3. Update repository metadata:\n");
        message.push_str("      inferno repo update --force\n\n");
        message.push_str("   4. Check proxy settings if behind corporate firewall:\n");
        message.push_str("      export HTTP_PROXY=http://proxy.company.com:8080\n");
        message.push_str("      export HTTPS_PROXY=http://proxy.company.com:8080\n\n");
        message.push_str("   5. Try again later (server might be temporarily down)\n");

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

    fn handle_repository_error(error_msg: &str) -> String {
        let mut message = String::from("❌ Repository error.\n\n");

        if error_msg.contains("not found") {
            message.push_str("💡 Repository not found or not configured.\n\n");
            message.push_str("🔧 Try these solutions:\n");
            message.push_str("   1. List available repositories:\n");
            message.push_str("      inferno repo list\n\n");
            message.push_str("   2. Add a repository:\n");
            message.push_str("      inferno repo add myrepo https://models.example.com\n\n");
            message.push_str("   3. Use a default repository:\n");
            message.push_str("      inferno search [model] --repo huggingface\n");
        } else {
            message.push_str("💡 Repository access or configuration issue.\n\n");
            message.push_str("🔧 Try these solutions:\n");
            message.push_str("   1. Test repository connectivity:\n");
            message.push_str("      inferno repo test [repository-name]\n\n");
            message.push_str("   2. Update repository metadata:\n");
            message.push_str("      inferno repo update [repository-name]\n\n");
            message.push_str("   3. Check repository status:\n");
            message.push_str("      inferno repo info [repository-name]\n\n");
            message.push_str("   4. Re-add repository if corrupted:\n");
            message.push_str("      inferno repo remove [repository-name]\n");
            message.push_str("      inferno repo add [repository-name] [url]\n");
        }

        message
    }

    fn handle_model_not_found_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Model not found.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • The model name is incorrect\n");
        message.push_str("   • The model doesn't exist in configured repositories\n");
        message.push_str("   • Repository metadata is outdated\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Search for similar models:\n");
        message.push_str("      inferno search [partial-model-name]\n\n");
        message.push_str("   2. Search in specific repositories:\n");
        message.push_str("      inferno search [model] --repo huggingface\n");
        message.push_str("      inferno search [model] --repo ollama\n\n");
        message.push_str("   3. Update repository metadata:\n");
        message.push_str("      inferno repo update --force\n\n");
        message.push_str("   4. List available models:\n");
        message.push_str("      inferno models list\n\n");
        message.push_str("   5. Try popular models:\n");
        message.push_str("      inferno install microsoft/DialoGPT-medium\n");
        message.push_str("      inferno install google/flan-t5-base\n");

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
        message.push_str("   2. Configure repository authentication:\n");
        message.push_str("      inferno repo add private-repo https://example.com --verify\n\n");
        message.push_str("   3. Try public models instead:\n");
        message.push_str("      inferno search [model-type] --repo huggingface\n\n");
        message.push_str("   4. Check your API tokens:\n");
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
        message.push_str("   1. Clean up package cache:\n");
        message.push_str("      inferno package clean --all\n\n");
        message.push_str("   2. Remove unused models:\n");
        message.push_str("      inferno package autoremove\n\n");
        message.push_str("   3. Check disk space:\n");
        message.push_str("      df -h\n\n");
        message.push_str("   4. Move models directory to larger disk:\n");
        message.push_str("      inferno config set models_dir /path/to/larger/disk/models\n\n");
        message.push_str("   5. Check cache directories:\n");
        message.push_str("      du -sh ~/.cache/inferno\n");
        message.push_str("      du -sh ~/.inferno\n");

        message
    }

    fn handle_dependency_error(_error_msg: &str) -> String {
        let mut message = String::from("❌ Dependency error.\n\n");

        message.push_str("💡 This usually means:\n");
        message.push_str("   • Missing model dependencies (tokenizers, configs, etc.)\n");
        message.push_str("   • Conflicting model versions\n");
        message.push_str("   • Broken dependency chain\n\n");

        message.push_str("🔧 Try these solutions:\n");
        message.push_str("   1. Install with dependency resolution:\n");
        message.push_str("      inferno install [model-name]  # (dependencies auto-resolved)\n\n");
        message.push_str("   2. Check dependency tree:\n");
        message.push_str("      inferno package depends [model-name] --tree\n\n");
        message.push_str("   3. Fix broken dependencies:\n");
        message.push_str("      inferno package check [model-name] --fix\n\n");
        message.push_str("   4. Remove conflicting models:\n");
        message.push_str("      inferno remove [conflicting-model]\n\n");
        message.push_str("   5. Clean install:\n");
        message.push_str("      inferno remove [model-name]\n");
        message.push_str("      inferno package clean --all\n");
        message.push_str("      inferno install [model-name]\n");

        message
    }

    fn handle_generic_error(error: &anyhow::Error) -> String {
        let mut message = String::from("❌ An error occurred.\n\n");

        message.push_str(&format!("Error: {}\n\n", error));

        message.push_str("🔧 General troubleshooting steps:\n");
        message.push_str("   1. Check system status:\n");
        message.push_str("      inferno config show\n");
        message.push_str("      inferno repo list\n\n");
        message.push_str("   2. Update and clean:\n");
        message.push_str("      inferno repo update --force\n");
        message.push_str("      inferno package clean --all\n\n");
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
            "install" | "package install" => {
                Self::check_install_prerequisites()
            }
            "search" | "package search" => {
                Self::check_search_prerequisites()
            }
            "serve" => {
                Self::check_serve_prerequisites()
            }
            "run" => {
                Self::check_run_prerequisites()
            }
            _ => None,
        }
    }

    fn check_install_prerequisites() -> Option<String> {
        // Check if any repositories are configured
        // In a real implementation, this would check the actual config

        // Check network connectivity
        if !Self::has_network_connectivity() {
            return Some(
                "⚠️  Network connectivity required for model installation.\n\n\
                🔧 Setup required:\n\
                   • Ensure you have an internet connection\n\
                   • Check firewall settings\n\
                   • Configure proxy if needed:\n\
                     export HTTP_PROXY=http://proxy.company.com:8080\n\n\
                💡 Alternatively, install models manually:\n\
                   • Download models to your models directory\n\
                   • Use: inferno models list\n".to_string()
            );
        }

        None
    }

    fn check_search_prerequisites() -> Option<String> {
        // Check repositories
        if !Self::has_repositories_configured() {
            return Some(
                "⚠️  No repositories configured for searching.\n\n\
                🔧 Setup required:\n\
                   1. Add a repository:\n\
                      inferno repo add huggingface https://huggingface.co\n\n\
                   2. Or update existing repositories:\n\
                      inferno repo update\n\n\
                   3. List available repositories:\n\
                      inferno repo list\n".to_string()
            );
        }

        None
    }

    fn check_serve_prerequisites() -> Option<String> {
        // Check if models are available
        if !Self::has_models_available() {
            return Some(
                "⚠️  No models available to serve.\n\n\
                🔧 Setup required:\n\
                   1. Install a model first:\n\
                      inferno install microsoft/DialoGPT-medium\n\n\
                   2. Or check existing models:\n\
                      inferno models list\n\
                      inferno list\n\n\
                   3. Verify models directory:\n\
                      inferno config get models_dir\n".to_string()
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
                      inferno install microsoft/DialoGPT-medium\n\n\
                   2. Or specify model path:\n\
                      inferno run --model /path/to/model.gguf --prompt \"Hello\"\n\n\
                   3. List available models:\n\
                      inferno models list\n".to_string()
            );
        }

        None
    }

    // Helper functions (these would check actual system state in real implementation)
    fn has_network_connectivity() -> bool {
        // In real implementation: try to ping a reliable host
        true // Assume network is available for now
    }

    fn has_repositories_configured() -> bool {
        // In real implementation: check config for repositories
        true // Default repositories are always configured
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
            "remove" => Self::remove_examples(),
            "list" => Self::list_examples(),
            "repo" => Self::repo_examples(),
            "package" => Self::package_examples(),
            _ => Self::general_examples(),
        }
    }

    fn install_examples() -> String {
        "📚 Installation Examples:\n\n\
        # Install popular models\n\
        inferno install microsoft/DialoGPT-medium\n\
        inferno install google/flan-t5-base\n\
        inferno install meta-llama/Llama-2-7b-chat-hf\n\n\
        # Install with options\n\
        inferno install gpt2 --auto-update\n\
        inferno install bert-base --yes --no-deps\n\n\
        # Install from specific repository\n\
        inferno search llama --repo ollama\n\
        inferno install ollama/llama2:7b\n".to_string()
    }

    fn search_examples() -> String {
        "📚 Search Examples:\n\n\
        # Basic search\n\
        inferno search \"language model\"\n\
        inferno search \"code generation\"\n\
        inferno search \"image classification\"\n\n\
        # Repository-specific search\n\
        inferno search \"llama\" --repo huggingface\n\
        inferno search \"resnet\" --repo onnx-models\n\n\
        # Detailed search\n\
        inferno search \"transformer\" --limit 10 --detailed\n".to_string()
    }

    fn remove_examples() -> String {
        "📚 Removal Examples:\n\n\
        # Remove models\n\
        inferno remove gpt2\n\
        inferno remove microsoft/DialoGPT-medium --yes\n\n\
        # Remove with dependencies\n\
        inferno remove old-model --yes\n\
        inferno package autoremove\n\n\
        # Clean up\n\
        inferno package clean --all\n".to_string()
    }

    fn list_examples() -> String {
        "📚 Listing Examples:\n\n\
        # List installed models\n\
        inferno list\n\
        inferno list --detailed\n\
        inferno list --filter \"llama\"\n\n\
        # List all models (including unmanaged)\n\
        inferno models list\n\n\
        # Package-specific listing\n\
        inferno package list --auto-only\n".to_string()
    }

    fn repo_examples() -> String {
        "📚 Repository Examples:\n\n\
        # List repositories\n\
        inferno repo list\n\
        inferno repo list --detailed\n\n\
        # Add custom repository\n\
        inferno repo add company https://models.company.com\n\
        inferno repo add private https://private.ai.com --verify\n\n\
        # Manage repositories\n\
        inferno repo update huggingface\n\
        inferno repo test ollama\n\
        inferno repo priority huggingface 1\n".to_string()
    }

    fn package_examples() -> String {
        "📚 Package Management Examples:\n\n\
        # Package operations\n\
        inferno package install llama-7b\n\
        inferno package update --check-only\n\
        inferno package upgrade --dry-run\n\n\
        # Maintenance\n\
        inferno package clean --all\n\
        inferno package check --deep\n\
        inferno package depends gpt2 --tree\n".to_string()
    }

    fn general_examples() -> String {
        "📚 General Examples:\n\n\
        # Quick start\n\
        inferno install microsoft/DialoGPT-medium\n\
        inferno run --model DialoGPT-medium --prompt \"Hello!\"\n\n\
        # Package management\n\
        inferno search \"transformer\"\n\
        inferno list\n\
        inferno package upgrade\n\n\
        # Configuration\n\
        inferno config show\n\
        inferno repo list\n\n\
        # Help\n\
        inferno --help\n\
        inferno [command] --help\n".to_string()
    }
}