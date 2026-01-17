# Homebrew formula for Inferno
class Inferno < Formula
  desc "Enterprise AI/ML model runner with automatic updates and real-time monitoring"
  homepage "https://github.com/ringo380/inferno"
  license "MIT OR Apache-2.0"
  version "0.9.0"

  # Platform-specific downloads
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-arm64"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-x86_64"
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-arm64"
      sha256 "LINUX_0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-x86_64"
      sha256 "LINUX_0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  def install
    # Rename downloaded file to inferno
    bin.install Dir["inferno-*"].first => "inferno"

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