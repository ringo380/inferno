//! Multimodal Command Examples - New Architecture

use anyhow::Result;
use inferno::{
    cli::multimodal_v2::*,
    config::Config,
    interfaces::cli::{Command, CommandContext},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Multimodal Command Examples (v2 Architecture) ===\n");

    let config = Config::default();

    // Example 1: Vision processing
    println!("Example 1: Vision processing");
    let cmd = MultimodalVision::new(
        config.clone(),
        "llava".to_string(),
        "image.jpg".to_string(),
        Some("Describe this image".to_string()),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 2: Audio transcription
    println!("Example 2: Audio transcription");
    let cmd = MultimodalAudio::new(
        config.clone(),
        "whisper".to_string(),
        "audio.mp3".to_string(),
        "transcribe".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 3: Audio translation
    println!("Example 3: Audio translation");
    let cmd = MultimodalAudio::new(
        config.clone(),
        "whisper".to_string(),
        "audio.mp3".to_string(),
        "translate".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 4: Mixed media
    println!("Example 4: Mixed media");
    let cmd = MultimodalMixed::new(
        config.clone(),
        "gemini".to_string(),
        vec!["image.jpg".to_string(), "audio.mp3".to_string()],
        "Analyze this content".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 5: Batch processing
    println!("Example 5: Batch processing");
    let cmd = MultimodalBatch::new(
        config.clone(),
        "model".to_string(),
        "input/".to_string(),
        "*.jpg".to_string(),
        "output/".to_string(),
    );
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    // Example 6: Check capabilities
    println!("Example 6: Check capabilities");
    let cmd = MultimodalCapabilities::new(config.clone(), "llava".to_string());
    let mut ctx = CommandContext::new(config.clone());
    let result = cmd.execute(&mut ctx).await?;
    println!("Result: {}\n", result.message);

    println!("=== All examples completed successfully ===");
    Ok(())
}