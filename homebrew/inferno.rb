# Homebrew formula for Inferno
class Inferno < Formula
  desc "Enterprise AI/ML model runner with automatic updates and real-time monitoring"
  homepage "https://github.com/ringo380/inferno"
  license "MIT OR Apache-2.0"
  version "0.10.4"

  # Platform-specific downloads
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-aarch64.tar.gz"
      sha256 "491cd2d33f7be554d6bcfa332bdb9ffc854299d18f4ec48444d53b0e4691b7f5"
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-macos-x86_64.tar.gz"
      sha256 "166cc6625170abe1d149c1979df79ca8d664b2af8f14b961890e2c428d526812"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-aarch64.tar.gz"
      sha256 "d438ee72653d5f517f4d5b6f720f5a62398c3031ed2582d068a714cecc690b94"
    else
      url "https://github.com/ringo380/inferno/releases/download/v#{version}/inferno-linux-x86_64.tar.gz"
      sha256 "3811b4c9614b3c25e10336c16d6a845fe37cd023dbac43dda224e661ebe1bf32"
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