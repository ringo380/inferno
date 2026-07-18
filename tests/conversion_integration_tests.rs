//! Integration tests for the model conversion subsystem.
//!
//! These tests exercise the REAL conversion API in `src/conversion.rs`. They are
//! deliberately honest about what the module actually does today:
//!
//!   * Real end-to-end: GGUF read/analyze, SafeTensors -> GGUF, GGUF quantization,
//!     GGUF passthrough, progress tracking.
//!   * Stub-but-runs: GGUF -> ONNX writes a placeholder ONNX header (the graph
//!     builder is not a full implementation); the test asserts only that the
//!     pipeline runs and produces a file, not that the ONNX is a real model.
//!   * Unsupported boundaries: ONNX input (reader disabled during the ort 2.0
//!     transition) and PyTorch input (the `tch` dependency was removed) both
//!     surface as a failed conversion. The tests assert that honest behavior.
//!
//! Fixtures are self-contained (a tiny valid synthetic GGUF and a real SafeTensors
//! file built with the `safetensors` crate) so the suite runs anywhere. One extra
//! test uses a real model when `INFERNO_TEST_MODEL` points at a GGUF file.
//!
//! Run with: `cargo test --test conversion_integration_tests --features gguf,onnx`

use anyhow::Result;
use inferno::{
    config::Config,
    conversion::{
        ConversionConfig, ModelConverter, ModelFormat, OptimizationLevel, QuantizationType,
    },
    models::ModelManager,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tempfile::TempDir;

// ── Fixture builders ─────────────────────────────────────────────────────────

fn write_gguf_string(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u64).to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

/// Build a minimal but structurally valid GGUF file with real (small) F32 tensor
/// data, matching the parser in `src/conversion.rs`. Sufficient for format
/// detection, metadata analysis, quantization, and GGUF->ONNX to run.
fn build_synthetic_gguf(path: &Path) -> Result<()> {
    const GGUF_VERSION: u32 = 3;
    const GGUF_TYPE_STRING: u32 = 8;
    const GGUF_TYPE_UINT32: u32 = 4;
    const GGML_F32: u32 = 0;

    let dims: [u64; 2] = [8, 8]; // 64 F32 elements = 256 bytes per tensor (2 Q4_0 blocks)
    let elems: usize = 64;
    let tensor_names = ["token_embd.weight", "output.weight"];
    let tensor_bytes = elems * 4;

    let mut out = Vec::new();
    out.extend_from_slice(b"GGUF");
    out.extend_from_slice(&GGUF_VERSION.to_le_bytes());
    out.extend_from_slice(&(tensor_names.len() as u64).to_le_bytes());
    out.extend_from_slice(&(3u64).to_le_bytes()); // metadata entry count

    // Metadata: two strings + one u32
    write_gguf_string(&mut out, "general.name");
    out.extend_from_slice(&GGUF_TYPE_STRING.to_le_bytes());
    write_gguf_string(&mut out, "synthetic_test_model");

    write_gguf_string(&mut out, "general.architecture");
    out.extend_from_slice(&GGUF_TYPE_STRING.to_le_bytes());
    write_gguf_string(&mut out, "llama");

    write_gguf_string(&mut out, "llama.context_length");
    out.extend_from_slice(&GGUF_TYPE_UINT32.to_le_bytes());
    out.extend_from_slice(&2048u32.to_le_bytes());

    // Tensor info records
    for (i, name) in tensor_names.iter().enumerate() {
        write_gguf_string(&mut out, name);
        out.extend_from_slice(&(dims.len() as u32).to_le_bytes());
        for d in dims {
            out.extend_from_slice(&d.to_le_bytes());
        }
        out.extend_from_slice(&GGML_F32.to_le_bytes());
        out.extend_from_slice(&((i * tensor_bytes) as u64).to_le_bytes());
    }

    // Align tensor data to a 32-byte boundary (as the reader expects)
    let aligned = (out.len() + 31) & !31;
    out.resize(aligned, 0);

    // Tensor data: contiguous F32 values for both tensors
    for i in 0..(tensor_names.len() * elems) {
        out.extend_from_slice(&((i as f32) * 0.01).to_le_bytes());
    }

    fs::write(path, &out)?;
    Ok(())
}

/// Build a real SafeTensors file with two small F32 tensors via the `safetensors`
/// crate. This is a genuine, parseable SafeTensors container.
fn build_safetensors(path: &Path) -> Result<()> {
    use safetensors::Dtype;
    use safetensors::tensor::TensorView;

    let rows = 8usize;
    let cols = 4usize;
    let bytes_for = |scale: f32| -> Vec<u8> {
        (0..rows * cols)
            .flat_map(|i| ((i as f32) * scale).to_le_bytes())
            .collect()
    };

    let d1 = bytes_for(0.01);
    let d2 = bytes_for(0.02);
    let v1 = TensorView::new(Dtype::F32, vec![rows, cols], &d1)?;
    let v2 = TensorView::new(Dtype::F32, vec![rows, cols], &d2)?;

    let mut tensors: HashMap<String, TensorView> = HashMap::new();
    tensors.insert("token_embd.weight".to_string(), v1);
    tensors.insert("output.weight".to_string(), v2);

    let serialized = safetensors::serialize(tensors, &None)?;
    fs::write(path, serialized)?;
    Ok(())
}

/// Write bytes that pass the ONNX validation gate (contain the "onnx" marker and a
/// protobuf-ish header) so a conversion attempt reaches the reader, which is
/// currently disabled.
fn write_onnxish_file(path: &Path) -> Result<()> {
    let mut content = Vec::new();
    content.extend_from_slice(&[0x08, 0x07]); // ir_version protobuf field
    content.extend_from_slice(b"onnx"); // marker the validator looks for
    content.extend_from_slice(&[0x12, 0x0e]);
    content.extend_from_slice(b"inferno_convert");
    content.resize(256, 0);
    fs::write(path, content)?;
    Ok(())
}

// ── Test harness helpers ─────────────────────────────────────────────────────

fn make_converter(models_dir: &Path) -> ModelConverter {
    let config = Config::default();
    let model_manager = Arc::new(ModelManager::new(models_dir));
    ModelConverter::new(model_manager, config)
}

fn base_config(output_format: ModelFormat) -> ConversionConfig {
    ConversionConfig {
        output_format,
        optimization_level: OptimizationLevel::Basic,
        quantization: None,
        target_precision: None,
        context_length: None,
        batch_size: None,
        preserve_metadata: true,
        verify_output: true,
    }
}

// ── Format detection & analysis ──────────────────────────────────────────────

#[tokio::test]
async fn test_format_detection() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let cases = [
        ("model.gguf", ModelFormat::Gguf),
        ("model.onnx", ModelFormat::Onnx),
        ("model.safetensors", ModelFormat::SafeTensors),
        ("model.pt", ModelFormat::Pytorch),
        ("model.pth", ModelFormat::Pytorch),
        ("model.pb", ModelFormat::TensorFlow),
    ];

    for (name, expected) in cases {
        let path = dir.path().join(name);
        fs::write(&path, b"placeholder")?;
        assert_eq!(
            converter.detect_format(&path).await?,
            expected,
            "format detection for {name}"
        );
    }

    // Unknown extension is an error, not a silent default.
    let unknown = dir.path().join("model.bin");
    fs::write(&unknown, b"x")?;
    assert!(converter.detect_format(&unknown).await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_analyze_gguf_metadata() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let gguf = dir.path().join("model.gguf");
    build_synthetic_gguf(&gguf)?;

    let analysis = converter.analyze_model(&gguf).await?;
    assert_eq!(analysis.format, ModelFormat::Gguf);
    assert!(analysis.file_size > 0);
    assert_eq!(analysis.tensor_count, 2, "two tensor infos were written");
    assert_eq!(
        analysis.metadata.get("general.name").map(String::as_str),
        Some("synthetic_test_model")
    );
    assert_eq!(
        analysis
            .metadata
            .get("general.architecture")
            .map(String::as_str),
        Some("llama")
    );
    assert_eq!(
        analysis
            .metadata
            .get("llama.context_length")
            .map(String::as_str),
        Some("2048"),
        "u32 metadata is decoded from little-endian bytes"
    );

    Ok(())
}

#[tokio::test]
async fn test_analyze_safetensors() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let st = dir.path().join("model.safetensors");
    build_safetensors(&st)?;

    let analysis = converter.analyze_model(&st).await?;
    assert_eq!(analysis.format, ModelFormat::SafeTensors);
    assert_eq!(analysis.tensor_count, 2);
    assert!(analysis.metadata.contains_key("token_embd.weight"));
    assert!(analysis.metadata.contains_key("output.weight"));

    Ok(())
}

// ── Real end-to-end conversions ──────────────────────────────────────────────

#[tokio::test]
async fn test_safetensors_to_gguf_conversion() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.safetensors");
    let output = dir.path().join("output.gguf");
    build_safetensors(&input)?;

    let config = base_config(ModelFormat::Gguf);
    let result = converter.convert_model(&input, &output, &config).await?;

    assert!(
        result.success,
        "SafeTensors -> GGUF should succeed: {:?}",
        result.errors
    );
    assert!(output.exists());
    assert_eq!(converter.detect_format(&output).await?, ModelFormat::Gguf);

    // The written GGUF is real: re-read it and confirm structure survived.
    let analysis = converter.analyze_model(&output).await?;
    assert_eq!(analysis.format, ModelFormat::Gguf);
    assert_eq!(analysis.tensor_count, 2, "both tensors carried through");
    assert_eq!(
        analysis
            .metadata
            .get("general.architecture")
            .map(String::as_str),
        Some("transformer"),
        "converter stamps a transformer architecture"
    );

    Ok(())
}

#[tokio::test]
async fn test_gguf_quantization() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.gguf");
    let output = dir.path().join("output_q4.gguf");
    build_synthetic_gguf(&input)?;

    // quantize_model's success is gated on quantization errors (unlike the
    // quantization step inside convert_model, which only warns), so this
    // directly verifies the GGUF quantizer ran.
    let result = converter
        .quantize_model(&input, &output, QuantizationType::Q4_0)
        .await?;
    assert!(
        result.success,
        "GGUF quantization should succeed: {:?}",
        result.errors
    );
    assert!(output.exists());
    assert!(fs::metadata(&output)?.len() > 0);

    // The quantized GGUF is well-formed: re-read it and confirm the tensors and
    // metadata survived (this fails if the writer corrupts string metadata).
    let analysis = converter.analyze_model(&output).await?;
    assert_eq!(analysis.tensor_count, 2);
    assert_eq!(
        analysis
            .metadata
            .get("general.architecture")
            .map(String::as_str),
        Some("llama")
    );

    Ok(())
}

#[tokio::test]
async fn test_gguf_passthrough_no_conversion() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.gguf");
    let output = dir.path().join("copy.gguf");
    build_synthetic_gguf(&input)?;

    // Same format, no quantization, no optimization => copy fast-path.
    let mut config = base_config(ModelFormat::Gguf);
    config.optimization_level = OptimizationLevel::None;

    let result = converter.convert_model(&input, &output, &config).await?;
    assert!(result.success);
    assert!(output.exists());
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("No conversion needed")),
        "passthrough should note it copied: {:?}",
        result.warnings
    );
    assert_eq!(
        fs::read(&input)?,
        fs::read(&output)?,
        "passthrough is a byte-for-byte copy"
    );

    Ok(())
}

#[tokio::test]
async fn test_gguf_to_onnx_runs() -> Result<()> {
    // NOTE: the ONNX graph builder currently emits a placeholder header rather
    // than a full ONNX model. This test asserts the pipeline RUNS and produces a
    // file with the right extension, not that the output is a loadable ONNX model.
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.gguf");
    let output = dir.path().join("output.onnx");
    build_synthetic_gguf(&input)?;

    let config = base_config(ModelFormat::Onnx);
    let result = converter.convert_model(&input, &output, &config).await?;

    assert!(
        result.success,
        "GGUF -> ONNX pipeline should run: {:?}",
        result.errors
    );
    assert!(output.exists());
    assert_eq!(converter.detect_format(&output).await?, ModelFormat::Onnx);

    Ok(())
}

#[tokio::test]
async fn test_conversion_result_fields() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.safetensors");
    let output = dir.path().join("output.gguf");
    build_safetensors(&input)?;

    let result = converter
        .convert_model(&input, &output, &base_config(ModelFormat::Gguf))
        .await?;

    assert!(result.success);
    assert_eq!(result.input_path, input);
    assert_eq!(result.output_path, output);
    assert!(result.input_size > 0, "input size recorded");
    assert!(result.output_size > 0, "output size recorded");
    assert!(result.compression_ratio > 0.0, "compression ratio computed");

    Ok(())
}

// ── Progress tracking ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_conversion_progress_tracking() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.safetensors");
    let output = dir.path().join("output.gguf");
    build_safetensors(&input)?;

    // No conversion started yet for this path.
    assert!(
        converter.get_conversion_progress(&input).await?.is_none(),
        "progress is None before any conversion"
    );

    // A clone shares the same progress map (Arc), matching how a spawned task
    // would observe progress written by convert_model.
    let observer = converter.clone();

    let result = converter
        .convert_model(&input, &output, &base_config(ModelFormat::Gguf))
        .await?;
    assert!(result.success);

    let progress = observer
        .get_conversion_progress(&input)
        .await?
        .expect("progress recorded after conversion");
    assert!(
        matches!(
            progress.stage,
            inferno::conversion::ConversionStage::Complete
        ),
        "final stage is Complete, got {:?}",
        progress.stage
    );
    assert_eq!(progress.progress_percent, 100.0);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_conversions() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let mut tasks = Vec::new();
    for i in 0..5 {
        let input = dir.path().join(format!("input_{i}.safetensors"));
        let output = dir.path().join(format!("output_{i}.gguf"));
        build_safetensors(&input)?;

        let converter = converter.clone();
        tasks.push(tokio::spawn(async move {
            converter
                .convert_model(&input, &output, &base_config(ModelFormat::Gguf))
                .await
        }));
    }

    for (i, task) in tasks.into_iter().enumerate() {
        let result = task.await??;
        assert!(
            result.success,
            "concurrent conversion {i} should succeed: {:?}",
            result.errors
        );
    }

    Ok(())
}

// ── Unsupported-input boundaries (honest current behavior) ───────────────────

#[tokio::test]
async fn test_onnx_input_unsupported() -> Result<()> {
    // ONNX reading is disabled during the ort 2.0 transition, so ONNX-input
    // conversions cannot succeed. Assert that failure explicitly.
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.onnx");
    let output = dir.path().join("output.gguf");
    write_onnxish_file(&input)?;

    let result = converter
        .convert_model(&input, &output, &base_config(ModelFormat::Gguf))
        .await?;

    assert!(!result.success, "ONNX input must not convert");
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("ONNX") || e.contains("validation")),
        "error should name the ONNX limitation: {:?}",
        result.errors
    );

    Ok(())
}

#[tokio::test]
async fn test_pytorch_input_unsupported() -> Result<()> {
    // PyTorch support was removed at the dependency level (tch), so .pt input
    // conversions fail. Assert that boundary.
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("input.pt");
    let output = dir.path().join("output.gguf");
    fs::write(&input, vec![0x80u8, 0x02, 0x00, 0x01])?; // non-empty pickle-ish bytes

    let result = converter
        .convert_model(&input, &output, &base_config(ModelFormat::Gguf))
        .await?;

    assert!(!result.success, "PyTorch input must not convert");
    assert!(
        result.errors.iter().any(|e| e.contains("PyTorch")),
        "error should name PyTorch: {:?}",
        result.errors
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_gguf_input() -> Result<()> {
    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let input = dir.path().join("invalid.gguf");
    let output = dir.path().join("output.onnx");
    fs::write(&input, b"NOTAGGUFHEADER____________")?; // wrong magic

    let result = converter
        .convert_model(&input, &output, &base_config(ModelFormat::Onnx))
        .await?;

    assert!(!result.success, "invalid GGUF must not convert");
    assert!(!result.errors.is_empty());

    Ok(())
}

// ── Optional real-model coverage ─────────────────────────────────────────────

/// Analyze a real model when `INFERNO_TEST_MODEL` points at a GGUF file. Skipped
/// (not failed) when the env var is unset, matching the backend/cache suites.
#[tokio::test]
async fn test_analyze_real_model() -> Result<()> {
    let Some(model) = std::env::var_os("INFERNO_TEST_MODEL").map(PathBuf::from) else {
        eprintln!("SKIP test_analyze_real_model: set INFERNO_TEST_MODEL to a GGUF file to run");
        return Ok(());
    };
    if !model.is_file() {
        eprintln!("SKIP test_analyze_real_model: INFERNO_TEST_MODEL is not a file: {model:?}");
        return Ok(());
    }

    let dir = TempDir::new()?;
    let converter = make_converter(dir.path());

    let analysis = converter.analyze_model(&model).await?;
    assert_eq!(analysis.format, ModelFormat::Gguf);
    assert!(analysis.tensor_count > 0, "real model has tensors");
    assert!(!analysis.metadata.is_empty(), "real model exposes metadata");

    Ok(())
}
