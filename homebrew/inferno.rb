# Homebrew formula for Inferno
class Inferno < Formula
  desc "Enterprise AI/ML model runner with automatic updates and real-time monitoring"
  homepage "https://github.com/ringo380/inferno"
  license "MIT OR Apache-2.0"
  version "0.10.4"

  # Platform-specific downloads
  # NOTE: SHA256 checksums will be populated by the release workflow after binaries are built
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER_MACOS_ARM64_SHA256"  # TODO: Update after v0.10.4 release
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_MACOS_X86_64_SHA256"  # TODO: Update after v0.10.4 release
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_ARM64_SHA256"  # TODO: Update after v0.10.4 release
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_X86_64_SHA256"  # TODO: Update after v0.10.4 release
    end
  end

  def install
    # Install the binary (extracted from tar.gz)
    bin.install "inferno"

    # Create directories for models and config
    (var/"inferno/models").mkpath
    (var/"inferno/cache").mkpath
    (etc/"inferno").mkpath

    # Install shell completions
    generate_completions_from_executable(bin/"inferno", "completions")
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
      Inferno has been installed!

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