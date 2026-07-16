use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

/// Test A/B Testing Feature Integration
#[test]
fn test_ab_testing_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("ab-test").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "A/B testing and canary deployment management",
    ));
}

#[test]
fn test_ab_testing_start_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("ab-test")
        .arg("start")
        .arg("--name")
        .arg("test1")
        .arg("--control-model")
        .arg("model1")
        .arg("--treatment-model")
        .arg("model2");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A/B Test Configuration"))
        .stdout(predicate::str::contains("Name: test1"));
}

#[test]
fn test_ab_testing_list_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("ab-test").arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A/B Tests"));
}

/// Test Job Queue Feature Integration
#[test]
fn test_queue_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("queue").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Advanced batch processing with job queues and scheduling",
    ));
}

#[test]
fn test_queue_list_empty() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("queue").arg("list-queues");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No queues found"));
}

#[test]
fn test_queue_create_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("queue")
        .arg("create")
        .arg("--name")
        .arg("test-queue")
        .arg("--max-concurrent")
        .arg("5")
        .arg("test-queue-id");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Queue 'test-queue-id' created successfully",
        ))
        .stdout(predicate::str::contains("Max concurrent jobs: 5"));
}

/// Test GPU Management Feature Integration
#[test]
fn test_gpu_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("gpu").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "GPU acceleration support and management",
    ));
}

#[test]
fn test_gpu_list_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("gpu").arg("list");

    // Output depends on the host: a table of GPUs where one is present, the
    // empty-set message otherwise. Both are a successful listing.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No GPUs found").or(predicate::str::contains("Vendor")));
}

#[test]
fn test_gpu_monitor_command() {
    // `gpu monitor` is a live watcher: it loops until interrupted and never
    // exits on its own. Cap it so a stuck watcher cannot block the suite, and
    // assert it reached the monitor loop before being killed.
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("gpu")
        .arg("monitor")
        .arg("--interval")
        .arg("1")
        .timeout(Duration::from_secs(10));

    cmd.assert()
        .interrupted()
        .stdout(predicate::str::contains("Monitoring GPUs"));
}

/// Test Model Versioning Feature Integration
#[test]
fn test_versioning_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("version").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Model versioning and rollback management",
    ));
}

#[test]
fn test_versioning_list_command() {
    // The version registry lives at "./models/versions" relative to the working
    // directory (VersioningConfig::default), so run from a tempdir to get an
    // empty registry instead of whatever exists in the checkout.
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("version").arg("list").current_dir(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No models found in registry"));
}

#[test]
fn test_versioning_create_command() {
    let temp_dir = tempdir().unwrap();
    let model_path = temp_dir.path().join("test.gguf");
    fs::write(&model_path, b"GGUF\x00\x00\x00\x01test data").unwrap();

    // `version create` writes its registry to "./models/versions" relative to the
    // working directory, so run from the tempdir to keep it out of the checkout.
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("version")
        .arg("create")
        .arg("--model-type")
        .arg("llm")
        .arg("--architecture")
        .arg("transformer")
        .arg("--framework")
        .arg("onnx")
        .arg("--framework-version")
        .arg("1.0")
        .arg("--format")
        .arg("gguf")
        .arg("--created-by")
        .arg("integration-test")
        .arg("--version")
        .arg("1.0.0")
        .arg("test-model")
        .arg(model_path.to_str().unwrap())
        .current_dir(temp_dir.path());

    cmd.assert().success().stdout(predicate::str::contains(
        "Model version created successfully",
    ));
}

/// Test Monitoring Feature Integration
#[test]
fn test_monitoring_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("monitor").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Real-time performance monitoring and alerting",
    ));
}

#[test]
fn test_monitoring_status_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("monitor").arg("status");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Monitoring System Status"));
}

/// Test Audit Feature Integration
#[test]
fn test_audit_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("audit").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Comprehensive audit logging and compliance tracking",
    ));
}

#[test]
fn test_audit_query_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("audit").arg("query").arg("--limit").arg("10");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No events found"));
}

/// Test Distributed Processing Feature Integration
#[test]
fn test_distributed_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("distributed").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Distributed inference with worker pools",
    ));
}

#[test]
fn test_distributed_status_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("distributed").arg("stats");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Distributed Configuration"));
}

/// Test Metrics Feature Integration
#[test]
fn test_metrics_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("metrics").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Metrics collection and export"));
}

#[test]
fn test_metrics_show_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("metrics").arg("snapshot");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("inference_metrics"));
}

/// Test Cache Feature Integration
#[test]
fn test_cache_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("cache").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Model caching and warm-up management",
    ));
}

#[test]
fn test_cache_status_command() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("cache").arg("stats");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Model Cache Statistics"));
}

/// Test Response Cache Feature Integration
#[test]
fn test_response_cache_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("response-cache").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Response caching and deduplication management",
    ));
}

/// Test Model Conversion Feature Integration
#[test]
fn test_convert_cli_integration() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("convert").arg("--help");
    cmd.assert().success().stdout(predicate::str::contains(
        "Convert and optimize models between formats",
    ));
}

/// Cross-Feature Integration Tests
#[test]
fn test_config_affects_all_commands() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();

    // Test that custom models directory is respected across commands
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("models")
        .arg("list")
        .env("INFERNO_MODELS_DIR", models_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No models found"));
}

#[test]
fn test_all_commands_show_help() {
    let commands = vec![
        "run",
        "batch",
        "serve",
        "models",
        "metrics",
        "bench",
        "validate",
        "config",
        "cache",
        "convert",
        "response-cache",
        "monitor",
        "distributed",
        "ab-test",
        "audit",
        "queue",
        "version",
        "gpu",
        "tui",
    ];

    for command in commands {
        let mut cmd = Command::cargo_bin("inferno").unwrap();
        cmd.arg(command).arg("--help");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("help").or(predicate::str::contains("Usage")));
    }
}

#[test]
fn test_invalid_commands_fail_gracefully() {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

/// Test CLI Consistency
#[test]
fn test_consistent_error_handling() {
    // Test that all commands handle missing required arguments consistently
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("run"); // Missing required --model and --prompt

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_environment_variable_support() {
    // Test that environment variables are respected
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("models")
        .arg("list")
        .env("INFERNO_MODELS_DIR", temp_dir.path())
        .env("INFERNO_LOG_LEVEL", "debug");

    cmd.assert().success();
}
