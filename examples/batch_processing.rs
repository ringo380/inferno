use inferno::io::json;
use serde_json::json;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔥 Inferno Batch Processing Example");
    println!("===================================");

    // Create sample batch input data
    let batch_data = vec![
        json!({
            "id": 1,
            "prompt": "What is artificial intelligence?",
            "metadata": {"category": "ai_basics"}
        }),
        json!({
            "id": 2,
            "prompt": "Explain machine learning in simple terms",
            "metadata": {"category": "ml_basics"}
        }),
        json!({
            "id": 3,
            "prompt": "What are the benefits of neural networks?",
            "metadata": {"category": "deep_learning"}
        }),
        json!({
            "id": 4,
            "prompt": "How does natural language processing work?",
            "metadata": {"category": "nlp"}
        }),
        json!({
            "id": 5,
            "prompt": "What is the future of AI technology?",
            "metadata": {"category": "future_tech"}
        }),
    ];

    // Save batch input to JSONL file
    let input_path = Path::new("batch_input.jsonl");
    println!("📝 Creating batch input file: {}", input_path.display());

    for (i, item) in batch_data.iter().enumerate() {
        json::append_jsonl_file(input_path, item).await?;
        println!("   Added prompt {}: {}", i + 1, item["prompt"]);
    }

    println!("✅ Batch input file created successfully!");
    println!("");

    // Create example configuration for batch processing
    let batch_config = json!({
        "model": "your_model.gguf",
        "parameters": {
            "max_tokens": 150,
            "temperature": 0.7,
            "top_p": 0.9
        },
        "output_format": "jsonl",
        "include_metadata": true
    });

    let config_path = Path::new("batch_config.json");
    json::write_json_file(config_path, &batch_config).await?;
    println!("⚙️  Configuration saved to: {}", config_path.display());

    println!("");
    println!("🚀 To run batch processing with Inferno:");
    println!("   inferno run --model your_model.gguf \\");
    println!("            --input batch_input.jsonl \\");
    println!("            --batch \\");
    println!("            --output batch_results.json");

    println!("");
    println!("📊 Expected output format:");
    let sample_output = json!([
        {
            "input": "What is artificial intelligence?",
            "output": "Artificial intelligence (AI) is...",
            "index": 0,
            "metadata": {"category": "ai_basics"},
            "inference_time_ms": 1250,
            "token_count": 45
        }
    ]);
    println!("{}", serde_json::to_string_pretty(&sample_output)?);

    println!("");
    println!("💡 Tips for batch processing:");
    println!("   - Use smaller batch sizes for memory efficiency");
    println!("   - Monitor system resources during processing");
    println!("   - Consider using --stream for real-time progress");
    println!("   - Save intermediate results for long batches");

    Ok(())
}
