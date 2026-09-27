// Basic functionality tests for Inferno
use inferno::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_config_creation() {
    let config = config::Config::default();
    // models_dir is now a PathBuf directly, not Option<PathBuf>
    assert!(!config.models_dir.as_os_str().is_empty());
    println!("✅ Config creation works");
}

#[test]
fn test_backend_types() {
    // Note: BackendType variants are feature-gated, using Gguf as default
    #[cfg(feature = "gguf")]
    {
        use backends::BackendType;

        let gguf_type = BackendType::Gguf;
        assert_eq!(format!("{:?}", gguf_type), "Gguf");
    }
    println!("✅ Backend types work");
}

#[test]
fn test_model_info() {
    use models::ModelInfo;

    let model = ModelInfo {
        name: "test-model".to_string(),
        path: PathBuf::from("test.gguf"),
        file_path: PathBuf::from("test.gguf"),
        size: 1024,
        size_bytes: 1024,
        modified: chrono::Utc::now(),
        backend_type: "gguf".to_string(),
        format: "gguf".to_string(),
        checksum: None,
        metadata: HashMap::new(),
    };

    assert_eq!(model.name, "test-model");
    assert_eq!(model.size_bytes, 1024);
    println!("✅ Model info creation works");
}

#[test]
fn test_error_types() {
    // InfernoError::Backend is a simple string variant
    let error = InfernoError::Backend("test error".to_string());

    match error {
        InfernoError::Backend(msg) => {
            assert_eq!(msg, "test error");
            println!("✅ Error handling works");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_inference_params() {
    use backends::InferenceParams;

    let params = InferenceParams::default();
    assert!(params.max_tokens > 0);
    println!("✅ Inference params work");
}

/// Every subcommand clap knows runs without the CLI first calling it unknown.
/// The suggestion layer used to check a hardcoded list of 11 names ahead of
/// clap and printed an "Unknown command" wall for the other 17 (#79).
#[test]
fn test_every_subcommand_runs_without_unknown_command_noise() {
    use assert_cmd::Command;
    use clap::CommandFactory;
    use predicates::prelude::*;

    let names: Vec<String> = cli::Cli::command()
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect();
    assert!(
        names.len() > 20,
        "expected the full command set, got {:?}",
        names
    );

    for name in names {
        Command::cargo_bin("inferno")
            .unwrap()
            .arg(&name)
            .arg("--help")
            .assert()
            .success()
            .stderr(predicate::str::contains("Unknown command").not());
    }
}

/// A mistyped subcommand still gets pointed at the right one, and an informal
/// alias still gets its note, both after clap's own error.
#[test]
fn test_mistyped_and_aliased_subcommands_get_suggestions() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    Command::cargo_bin("inferno")
        .unwrap()
        .arg("modles")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'modles'"))
        .stderr(predicate::str::contains("'models'"));

    Command::cargo_bin("inferno")
        .unwrap()
        .arg("cfg")
        .arg("show")
        .assert()
        .failure()
        .stderr(predicate::str::contains("'cfg' is an alias for 'config'"));
}
