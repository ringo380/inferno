/// End-to-end integration tests that simulate real user workflows
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Minimal GGUF header the validator accepts: magic, version 3 (little-endian),
/// zero tensors, zero metadata entries, then whatever payload the caller wants.
fn write_stub_gguf(path: &Path, payload: &[u8]) {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"GGUF");
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(payload);
    fs::write(path, bytes).unwrap();
}

/// A CLI invocation pinned to a scratch directory. Some subcommands write
/// relative to the working directory (audit keeps `./audit_logs`), so every
/// test runs inside its own temp dir with its own models and cache dirs.
fn inferno(cwd: &Path, models_dir: &Path, cache_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.current_dir(cwd)
        .env("INFERNO_MODELS_DIR", models_dir)
        .env("INFERNO_CACHE_DIR", cache_dir);
    cmd
}

/// Pull the id out of `version create`'s "Version ID: <uuid>" line.
fn version_id_from(stdout: &[u8]) -> String {
    let text = String::from_utf8_lossy(stdout);
    let line = text
        .lines()
        .find(|l| l.contains("Version ID: "))
        .unwrap_or_else(|| panic!("no Version ID line in output:\n{}", text));
    line.split("Version ID: ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap()
        .to_string()
}

/// Test complete model lifecycle: discovery, validation, caching, inference
#[test]
fn test_complete_model_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let model_path = models_dir.join("test-model.gguf");
    write_stub_gguf(&model_path, b"test model data for lifecycle test");

    // Step 1: Model discovery
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("models")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-model.gguf"));

    // Step 2: Model validation
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("validate")
        .arg(&model_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("All validations passed"));

    // Step 3: Cache warm-up
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("cache")
        .arg("warmup")
        .arg("test-model.gguf")
        .assert()
        .success()
        .stdout(predicate::str::contains("Warmup completed"));

    // Step 4: Cache statistics
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("cache")
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Model Cache Statistics"));

    // Step 5: Inference attempt. The stub has no tensors, so the backend must
    // refuse it cleanly rather than crash.
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("run")
        .arg("--model")
        .arg("test-model.gguf")
        .arg("--prompt")
        .arg("Hello, world!")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// Test batch processing workflow
#[test]
fn test_batch_processing_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&models_dir).unwrap();
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    write_stub_gguf(&models_dir.join("batch-model.gguf"), b"batch test model");

    // JSONL inputs carry the text in a `content` field.
    let input_file = input_dir.join("inputs.jsonl");
    let test_inputs = [
        r#"{"content": "Hello, world!"}"#,
        r#"{"content": "How are you?"}"#,
        r#"{"content": "What is AI?"}"#,
    ];
    fs::write(&input_file, test_inputs.join("\n")).unwrap();

    // Step 1: Validate batch input format. The count is reported through the
    // info log, so pin the log settings the assertion depends on.
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .env("INFERNO_LOG_LEVEL", "info")
        .env("INFERNO_LOG_FORMAT", "pretty")
        .arg("batch")
        .arg("--model")
        .arg("batch-model.gguf")
        .arg("--input")
        .arg(&input_file)
        .arg("--output")
        .arg(output_dir.join("results.jsonl"))
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("parsed 3 inputs"));

    // Step 2: A JSONL line without `content` is rejected up front.
    let legacy_file = input_dir.join("prompt-only.jsonl");
    fs::write(&legacy_file, r#"{"prompt": "Hello, world!"}"#).unwrap();

    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("batch")
        .arg("--model")
        .arg("batch-model.gguf")
        .arg("--input")
        .arg(&legacy_file)
        .arg("--output")
        .arg(output_dir.join("unused.jsonl"))
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No content field"));

    // Step 3: Run batch processing. The stub model cannot load, so either a
    // clean success or a clean failure is acceptable; a crash is not.
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("batch")
        .arg("--model")
        .arg("batch-model.gguf")
        .arg("--input")
        .arg(&input_file)
        .arg("--output")
        .arg(output_dir.join("results.jsonl"))
        .arg("--concurrency")
        .arg("2")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

/// Test advanced queue management workflow
#[test]
fn test_queue_management_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    write_stub_gguf(&models_dir.join("queue-model.gguf"), b"queue test model");

    // Step 1: Create job queue
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("queue")
        .arg("create")
        .arg("--name")
        .arg("test-processing-queue")
        .arg("--max-concurrent")
        .arg("3")
        .arg("test-processing-queue")
        .assert()
        .success()
        .stdout(predicate::str::contains("created successfully"));

    // Step 2: List queues, table and JSON
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("queue")
        .arg("list-queues")
        .assert()
        .success();

    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("queue")
        .arg("list-queues")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Step 3: Submitting to a queue that was never created is rejected
    let input_file = temp_dir.path().join("queue_input.txt");
    fs::write(&input_file, "Test input for queue processing").unwrap();

    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("queue")
        .arg("submit")
        .arg("--name")
        .arg("first-job")
        .arg("--input-file")
        .arg(&input_file)
        .arg("--model")
        .arg("queue-model.gguf")
        .arg("--priority")
        .arg("high")
        .arg("no-such-queue")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test model versioning and deployment workflow
#[test]
fn test_versioning_and_deployment_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    let model_v1 = models_dir.join("chat-model-v1.gguf");
    let model_v2 = models_dir.join("chat-model-v2.gguf");
    write_stub_gguf(&model_v1, b"chat model version 1.0");
    write_stub_gguf(&model_v2, b"chat model version 2.0 with improvements");

    let create = |version: &str, description: &str, file: &Path| -> String {
        let output = inferno(temp_dir.path(), &models_dir, &cache_dir)
            .arg("version")
            .arg("create")
            .arg("--model-type")
            .arg("llm")
            .arg("--architecture")
            .arg("transformer")
            .arg("--framework")
            .arg("llama.cpp")
            .arg("--framework-version")
            .arg("1.0")
            .arg("--format")
            .arg("gguf")
            .arg("--created-by")
            .arg("e2e-test")
            .arg("--version")
            .arg(version)
            .arg("--description")
            .arg(description)
            .arg("chat-model")
            .arg(file)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Model version created successfully",
            ))
            .get_output()
            .clone();
        version_id_from(&output.stdout)
    };

    // Step 1: Register two versions
    let v1 = create("1.0.0", "Initial release", &model_v1);
    let v2 = create("2.0.0", "Performance improvements", &model_v2);
    assert_ne!(v1, v2);
    assert!(models_dir.join("versions").join("registry.json").exists());

    // Step 2: List versions
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("version")
        .arg("list")
        .arg("chat-model")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0").and(predicate::str::contains("2.0.0")));

    // Step 3: Promote to staging
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("version")
        .arg("promote")
        .arg("--promoted-by")
        .arg("e2e-test")
        .arg("chat-model")
        .arg(&v2)
        .arg("staging")
        .assert()
        .success()
        .stdout(predicate::str::contains("promoted successfully"));

    // Step 4: Deploy to production
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("version")
        .arg("deploy")
        .arg("chat-model")
        .arg(&v2)
        .arg("production")
        .assert()
        .success()
        .stdout(predicate::str::contains("deployed successfully"));

    // Step 5: Compare versions
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("version")
        .arg("compare")
        .arg("chat-model")
        .arg(&v1)
        .arg(&v2)
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0").and(predicate::str::contains("2.0.0")));
}

/// Test A/B testing workflow
#[test]
fn test_ab_testing_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    write_stub_gguf(
        &models_dir.join("control-model.gguf"),
        b"control model baseline",
    );
    write_stub_gguf(
        &models_dir.join("treatment-model.gguf"),
        b"treatment model experimental",
    );

    // Step 1: Start A/B test
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("ab-test")
        .arg("start")
        .arg("--name")
        .arg("performance-comparison")
        .arg("--control-model")
        .arg("control-model.gguf")
        .arg("--treatment-model")
        .arg("treatment-model.gguf")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Control Model: control-model.gguf").and(
                predicate::str::contains("Treatment Model: treatment-model.gguf"),
            ),
        );

    // Step 2: Control and treatment must differ
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("ab-test")
        .arg("start")
        .arg("--name")
        .arg("same-model")
        .arg("--control-model")
        .arg("control-model.gguf")
        .arg("--treatment-model")
        .arg("control-model.gguf")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be different"));

    // Step 3: List tests
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("ab-test")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("A/B Tests"));

    // Step 4: Check test status
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("ab-test")
        .arg("status")
        .arg("performance-comparison")
        .assert()
        .success()
        .stdout(predicate::str::contains("Name: performance-comparison"));

    // Step 5: Stop test
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("ab-test")
        .arg("stop")
        .arg("performance-comparison")
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopping A/B Test"));
}

/// Test monitoring and alerting workflow
#[test]
fn test_monitoring_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    // Step 1: Check monitoring status
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("monitor")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Monitoring System Status"));

    // Step 2: List active alerts
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("monitor")
        .arg("alerts")
        .assert()
        .success()
        .stdout(predicate::str::contains("Active Alerts"));

    // Step 3: Filtered alert listing
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("monitor")
        .arg("alerts")
        .arg("--severity")
        .arg("critical")
        .arg("--limit")
        .arg("5")
        .assert()
        .success();
}

/// Test audit and compliance workflow
#[test]
fn test_audit_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    // Step 1: Query recent audit events (fresh directory, so none yet)
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("audit")
        .arg("query")
        .arg("--limit")
        .arg("50")
        .assert()
        .success()
        .stdout(predicate::str::contains("event"));

    // Step 2: Search for specific events
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("audit")
        .arg("search")
        .arg("model_loaded")
        .assert()
        .success();

    // Step 3: Export audit logs
    let export_file = temp_dir.path().join("audit_export.json");
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("audit")
        .arg("export")
        .arg("--format")
        .arg("json")
        .arg(&export_file)
        .assert()
        .success();
    assert!(export_file.exists());

    // Step 4: Statistics
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("audit")
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Audit Statistics"));
}

/// Test GPU management workflow. Must pass on machines with no GPU at all.
#[test]
fn test_gpu_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    // Step 1: List available GPUs, either a table or an empty-result notice
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("gpu")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("ID").or(predicate::str::contains("No GPUs found")));

    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("gpu")
        .arg("list")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Step 2: List allocations
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("gpu")
        .arg("allocations")
        .assert()
        .success();

    // Step 3: Benchmark GPU performance
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("gpu")
        .arg("benchmark")
        .arg("0")
        .arg("--iterations")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Benchmark"));
}

/// Test distributed processing workflow
#[test]
fn test_distributed_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    write_stub_gguf(
        &models_dir.join("dist-model.gguf"),
        b"distributed test model",
    );

    // Step 1: Show the distributed configuration
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("distributed")
        .arg("stats")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Distributed Configuration")
                .and(predicate::str::contains("worker_count")),
        );

    // Step 2: A test request against an unloadable model fails cleanly
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("distributed")
        .arg("test")
        .arg("--model")
        .arg("dist-model.gguf")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// Test metrics and observability workflow
#[test]
fn test_metrics_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();

    // Step 1: JSON metrics
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("metrics")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("inference_metrics"));

    // Step 2: Prometheus exposition format
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("metrics")
        .arg("prometheus")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "# TYPE inferno_inference_requests_total counter",
        ));

    // Step 3: Pretty snapshot
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("metrics")
        .arg("snapshot")
        .arg("--pretty")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_requests\""));
}

/// Test configuration management across all features
#[test]
fn test_configuration_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&models_dir).unwrap();
    let config_file = temp_dir.path().join("inferno_config.toml");

    // Step 1: Show current configuration
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("config")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("models_dir"));

    // Step 2: Write a configuration file
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("config")
        .arg("init")
        .arg("--path")
        .arg(&config_file)
        .assert()
        .success();

    assert!(config_file.exists());
    let config_content = fs::read_to_string(&config_file).unwrap();
    assert!(config_content.contains("models_dir"));

    // Step 3: Validate it
    inferno(temp_dir.path(), &models_dir, &cache_dir)
        .arg("config")
        .arg("validate")
        .arg("--path")
        .arg(&config_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

/// Test error recovery and resilience
#[tokio::test]
async fn test_error_recovery_workflow() {
    let temp_dir = tempdir().unwrap();

    // Test graceful handling of missing files
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("validate").arg("/nonexistent/path/model.gguf");

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("does not exist"));

    // Test handling of invalid model files
    let invalid_model = temp_dir.path().join("invalid.gguf");
    fs::write(&invalid_model, b"INVALID_MODEL_DATA").unwrap();

    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("validate").arg(invalid_model.to_str().unwrap());

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("validations failed"));

    // Test handling of permission errors (when possible)
    // This is platform-dependent and may not work in all environments

    // Test handling of resource exhaustion scenarios
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("batch")
        .arg("--model")
        .arg("nonexistent-model")
        .arg("--input")
        .arg("/dev/null")
        .arg("--output")
        .arg("/tmp/test_output")
        .arg("--max-concurrent")
        .arg("1000"); // Unrealistic value

    // Should fail gracefully
    cmd.assert().failure();
}

/// Integration test for TUI mode
#[test]
fn test_tui_launch() {
    // TUI requires interactive terminal, so we just test that it can start
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("tui").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Launch terminal user interface"));
}

/// Test complete server workflow
#[tokio::test]
async fn test_server_workflow() {
    let temp_dir = tempdir().unwrap();
    let models_dir = temp_dir.path().join("models");
    fs::create_dir_all(&models_dir).unwrap();

    // Create mock model for server
    write_stub_gguf(&models_dir.join("server-model.gguf"), b"server test model");

    // Test server help
    let mut cmd = Command::cargo_bin("inferno").unwrap();
    cmd.arg("serve").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start local HTTP API server"));

    // Note: We don't actually start the server in tests as it would bind to ports
    // and potentially conflict with other tests or running instances
}
