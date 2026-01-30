# Homebrew formula for Inferno AI
class InfernoAi < Formula
  desc "Enterprise AI/ML model runner with automatic updates and real-time monitoring"
  homepage "https://github.com/ringo380/inferno"
  license "MIT OR Apache-2.0"
  version "0.10.5"

  # Platform-specific downloads
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-aarch64.tar.gz"
      sha256 "ea37fef8009c00a2bec5a6df0a52c95b6c2e2218182edc7808dcd97d52893208"
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-x86_64.tar.gz"
      sha256 "646b293bbbe467919f3e71a9671f1b0b0986057f712f44bfc1221ab29fc39d1f"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-aarch64.tar.gz"
      sha256 "f25d3164eeaa88d9174ef65e8f8a43f8cb06865871e11b8234fe8348ab63b514"
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-x86_64.tar.gz"
      sha256 "b08e6936e6e73ad53bc74e373e42b5f6f9f6ac0bb9c6abe97308e2b8af947e99"
    end
  end

  def install
    # Install the binary (extracted from tar.gz)
    bin.install "inferno"

    # Create directories for models and config
    (var/"inferno/models").mkpath
    (var/"inferno/cache").mkpath
    (etc/"inferno").mkpath

    # Note: Shell completions not yet supported by inferno CLI
    # TODO: Add completions when `inferno completions` command is implemented
  end

  def post_install
    # Create default config if it doesn't exist
    unless (etc/"inferno/config.toml").exist?
      (etc/"inferno/config.toml").write <<~EOS
        # Inferno Configuration
        models_dir = "#{var}/inferno/models"
        cache_dir = "#{var}/inferno/cache"
        log_level = "info"

        [server]
        bind_address = "127.0.0.1"
        port = 8080

        [backend_config]
        gpu_enabled = true
        context_size = 4096
      EOS
    end
  end

  def caveats
    <<~EOS
      Inferno AI has been installed!

      Configuration file: #{etc}/inferno/config.toml
      Models directory: #{var}/inferno/models
      Cache directory: #{var}/inferno/cache

      To get started:
        inferno --help
        inferno models list
        inferno serve

      To install models:
        Place your GGUF/ONNX models in #{var}/inferno/models
    EOS
  end

  service do
    run [opt_bin/"inferno", "serve"]
    keep_alive true
    log_path var/"log/inferno.log"
    error_log_path var/"log/inferno-error.log"
    environment_variables INFERNO_CONFIG_FILE: etc/"inferno/config.toml"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/inferno --version")
    assert_match "Usage:", shell_output("#{bin}/inferno --help")
  end
end
