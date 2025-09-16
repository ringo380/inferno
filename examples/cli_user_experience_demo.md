# Inferno CLI User Experience Demo

This document demonstrates the enhanced user-friendly CLI experience with typo detection, helpful error messages, and setup guidance.

## 🔍 Typo Detection and Command Suggestions

### Common Typos Get Automatic Suggestions

```bash
# User types a common typo
$ inferno instal gpt2
💡 Did you mean 'install'?
   You typed: instal
💡 Try: inferno install

# The command provides a helpful suggestion and continues
```

```bash
# User tries to remove with wrong command
$ inferno rm gpt2
💡 Note: 'rm' is an alias for 'remove'

# Automatically translates to the correct command
```

```bash
# User makes a spelling mistake
$ inferno serch "language model"
❓ Did you mean 'search'?
   You typed: serch
💡 Try: inferno search
```

### Invalid Commands Get Helpful Guidance

```bash
# User types completely wrong command
$ inferno xyz123
❌ Unknown command: 'xyz123'

💡 Did you mean one of these?
   • install
   • search
   • list

🔧 Common commands:
   • inferno install <model>     # Install a model
   • inferno search <query>      # Search for models
   • inferno list                # List installed models
   • inferno run <model>         # Run inference
   • inferno --help              # Show all commands

📚 General Examples:

# Quick start
inferno install microsoft/DialoGPT-medium
inferno run --model DialoGPT-medium --prompt "Hello!"

# Package management
inferno search "transformer"
inferno list
inferno package upgrade

# Configuration
inferno config show
inferno repo list

# Help
inferno --help
inferno [command] --help
```

## 🛠️ Setup Guidance and Prerequisites

### Automatic Prerequisites Checking

```bash
# User tries to install without network
$ inferno install gpt2
⚠️  Network connectivity required for model installation.

🔧 Setup required:
   • Ensure you have an internet connection
   • Check firewall settings
   • Configure proxy if needed:
     export HTTP_PROXY=http://proxy.company.com:8080

💡 Alternatively, install models manually:
   • Download models to your models directory
   • Use: inferno models list

❓ Continue anyway? (y/N): n
Operation cancelled.
```

```bash
# User tries to serve without models
$ inferno serve
⚠️  No models available to serve.

🔧 Setup required:
   1. Install a model first:
      inferno install microsoft/DialoGPT-medium

   2. Or check existing models:
      inferno models list
      inferno list

   3. Verify models directory:
      inferno config get models_dir
```

## 📝 Enhanced Error Messages

### Model Not Found Errors

```bash
$ inferno install nonexistent-model
❌ Package 'nonexistent-model' not found

💡 Try these alternatives:
   • Search for similar packages:
     inferno search nonexistent-model
   • Check available repositories:
     inferno repo list
   • Search in specific repository:
     inferno search nonexistent-model --repo huggingface

📚 Popular models to try:
   • inferno install microsoft/DialoGPT-medium
   • inferno install google/flan-t5-base
   • inferno install facebook/bart-large-cnn
```

### Network Errors

```bash
$ inferno install gpt2
❌ Network connection error.

💡 This usually means:
   • No internet connection
   • Repository server is down
   • Firewall is blocking the connection
   • Proxy configuration issues

🔧 Try these solutions:
   1. Check your internet connection:
      ping google.com

   2. Test repository connectivity:
      inferno repo test huggingface

   3. Update repository metadata:
      inferno repo update --force

   4. Check proxy settings if behind corporate firewall:
      export HTTP_PROXY=http://proxy.company.com:8080
      export HTTPS_PROXY=http://proxy.company.com:8080

   5. Try again later (server might be temporarily down)
```

### Search with No Results

```bash
$ inferno search "xyz-nonexistent"
❌ No packages found matching: 'xyz-nonexistent'

💡 Suggestions:
   • Try a broader search term
   • Check spelling: xyz-nonexistent
   • Update repository metadata:
     inferno repo update --force
   • List available repositories:
     inferno repo list

📚 Popular models to try:
   • inferno install microsoft/DialoGPT-medium
   • inferno install google/flan-t5-base
   • inferno install facebook/bart-large-cnn
```

### Permission Errors

```bash
$ inferno config set models_dir /root/models
❌ Permission denied.

💡 This usually means:
   • You don't have permission to access the file/directory
   • The file is owned by another user
   • SELinux or similar security policies are blocking access

🔧 Try these solutions:
   1. Check file permissions:
      ls -la [file-path]

   2. Fix permissions if you own the file:
      chmod 644 [file-path]  # for files
      chmod 755 [directory]  # for directories

   3. Use your home directory instead:
      inferno config set models_dir ~/models

   4. Run with appropriate permissions (be careful!):
      sudo inferno [command]  # only if necessary
```

### Dependency Errors

```bash
$ inferno install complex-model --no-deps
❌ Dependency error.

💡 This usually means:
   • Missing model dependencies (tokenizers, configs, etc.)
   • Conflicting model versions
   • Broken dependency chain

🔧 Try these solutions:
   1. Install with dependency resolution:
      inferno install complex-model  # (dependencies auto-resolved)

   2. Check dependency tree:
      inferno package depends complex-model --tree

   3. Fix broken dependencies:
      inferno package check complex-model --fix

   4. Remove conflicting models:
      inferno remove conflicting-model

   5. Clean install:
      inferno remove complex-model
      inferno package clean --all
      inferno install complex-model
```

## 🎯 Context-Aware Help

### Command-Specific Examples

```bash
# User asks for help with install
$ inferno install --help
# ... standard clap help output ...

# Then automatically shows examples:
📚 Installation Examples:

# Install popular models
inferno install microsoft/DialoGPT-medium
inferno install google/flan-t5-base
inferno install meta-llama/Llama-2-7b-chat-hf

# Install with options
inferno install gpt2 --auto-update
inferno install bert-base --yes --no-deps

# Install from specific repository
inferno search llama --repo ollama
inferno install ollama/llama2:7b
```

### Post-Error Guidance

```bash
# After install command fails
$ inferno install nonexistent-model
❌ Package 'nonexistent-model' not found
# ... error details ...

💡 Try searching for the model first:
   inferno search [partial-model-name]
   inferno search [model] --repo huggingface
```

```bash
# After run command fails due to no models
$ inferno run --prompt "Hello"
❌ No models available for inference.
# ... error details ...

💡 Make sure you have models available:
   inferno list                    # Check installed models
   inferno models list             # Check all models
   inferno install [model-name]    # Install a model
```

## 🔄 Intelligent Aliases and Shortcuts

### Common Command Aliases

```bash
# Package management aliases
inferno rm model-name        # → inferno remove model-name
inferno ls                   # → inferno list
inferno add model-name       # → inferno install model-name
inferno get model-name       # → inferno install model-name

# Search aliases
inferno find "query"         # → inferno search "query"
inferno query "text"         # → inferno search "text"

# Repository aliases
inferno repos                # → inferno repo list
inferno repository add       # → inferno repo add

# Configuration aliases
inferno cfg show             # → inferno config show
inferno settings             # → inferno config show
```

### Subcommand Suggestions

```bash
# User forgets subcommand
$ inferno package
💡 Package management commands:
   • inferno package install <model>
   • inferno package remove <model>
   • inferno package search <query>
   • inferno package list
   • inferno package update
   • inferno package upgrade
```

## 🌟 Pro Tips Integration

The CLI automatically provides relevant pro tips based on user actions:

```bash
$ inferno install microsoft/DialoGPT-medium
✅ Package installation started successfully!
📦 Package: microsoft/DialoGPT-medium
🔄 Download ID: abc123
📋 Dependencies will be resolved automatically

💡 Monitor progress with:
   inferno marketplace progress abc123

💡 After installation, try:
   inferno run --model DialoGPT-medium --prompt "Hello!"
```

## 🚀 Progressive Complexity

The CLI adapts to user expertise:

### Beginner Mode (auto-detected)
```bash
# Lots of helpful hints and explanations
$ inferno install gpt2
✅ Package installation started successfully!
💡 This may take a few minutes for large models...
💡 You can monitor progress with: inferno marketplace progress [id]
💡 After installation, try: inferno run --model gpt2 --prompt "Hello!"
```

### Advanced Mode (fewer hints)
```bash
# More concise output for experienced users
$ inferno install gpt2 --yes --auto-update
✅ gpt2 installation started (download: abc123)
```

This enhanced CLI experience makes Inferno much more approachable for newcomers while still being efficient for power users!