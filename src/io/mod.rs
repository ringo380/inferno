use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum InputFormat {
    #[value(name = "text")]
    Text,
    #[value(name = "image")]
    Image,
    #[value(name = "audio")]
    Audio,
    #[value(name = "json")]
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    #[value(name = "text")]
    Text,
    #[value(name = "json")]
    Json,
    #[value(name = "jsonl")]
    JsonLines,
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputFormat::Text => write!(f, "text"),
            InputFormat::Image => write!(f, "image"),
            InputFormat::Audio => write!(f, "audio"),
            InputFormat::Json => write!(f, "json"),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::JsonLines => write!(f, "jsonl"),
        }
    }
}

pub mod text {
    use anyhow::Result;
    use std::path::Path;
    use tokio::fs;

    pub async fn read_text_file(path: &Path) -> Result<String> {
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }

    pub async fn write_text_file(path: &Path, content: &str) -> Result<()> {
        fs::write(path, content).await?;
        Ok(())
    }

    pub fn validate_utf8(data: &[u8]) -> Result<String> {
        String::from_utf8(data.to_vec())
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }
}

pub mod image {
    use anyhow::Result;
    use image::{ImageBuffer, ImageFormat, Rgb};
    use std::path::Path;

    pub async fn load_image(path: &Path) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let data = tokio::fs::read(path).await?;
        let img = image::load_from_memory(&data)?;
        Ok(img.to_rgb8())
    }

    pub async fn save_image(
        path: &Path,
        img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
        format: ImageFormat,
    ) -> Result<()> {
        let mut buffer = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buffer), format)?;
        tokio::fs::write(path, buffer).await?;
        Ok(())
    }

    pub fn resize_image(
        img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
        width: u32,
        height: u32,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        image::imageops::resize(img, width, height, image::imageops::FilterType::Lanczos3)
    }

    pub fn normalize_image(img: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<f32> {
        img.as_raw()
            .iter()
            .map(|&pixel| (pixel as f32) / 255.0)
            .collect()
    }

    pub fn get_image_format(path: &Path) -> Result<ImageFormat> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => Ok(ImageFormat::Jpeg),
            Some("png") => Ok(ImageFormat::Png),
            Some("bmp") => Ok(ImageFormat::Bmp),
            Some("tiff") | Some("tif") => Ok(ImageFormat::Tiff),
            Some("webp") => Ok(ImageFormat::WebP),
            _ => Err(anyhow::anyhow!("Unsupported image format")),
        }
    }
}

pub mod audio {
    use anyhow::Result;
    use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
    use std::path::Path;

    pub async fn load_wav_file(path: &Path) -> Result<(Vec<f32>, WavSpec)> {
        let mut reader = WavReader::open(path)?;
        let spec = reader.spec();

        let samples: Result<Vec<f32>, _> = match spec.sample_format {
            SampleFormat::Float => reader.samples::<f32>().collect(),
            SampleFormat::Int => reader
                .samples::<i32>()
                .map(|s| s.map(|s| s as f32 / i32::MAX as f32))
                .collect(),
        };

        let samples = samples?;
        Ok((samples, spec))
    }

    pub async fn save_wav_file(path: &Path, samples: &[f32], spec: WavSpec) -> Result<()> {
        let mut writer = WavWriter::create(path, spec)?;

        match spec.sample_format {
            SampleFormat::Float => {
                for &sample in samples {
                    writer.write_sample(sample)?;
                }
            }
            SampleFormat::Int => {
                for &sample in samples {
                    let int_sample = (sample * i32::MAX as f32) as i32;
                    writer.write_sample(int_sample)?;
                }
            }
        }

        writer.finalize()?;
        Ok(())
    }

    pub fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return samples.to_vec();
        }

        let ratio = to_rate as f64 / from_rate as f64;
        let new_length = (samples.len() as f64 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_length);

        for i in 0..new_length {
            let src_index = (i as f64 / ratio) as usize;
            if src_index < samples.len() {
                resampled.push(samples[src_index]);
            } else {
                resampled.push(0.0);
            }
        }

        resampled
    }

    pub fn normalize_audio(samples: &[f32]) -> Vec<f32> {
        let max_amplitude = samples.iter()
            .map(|&s| s.abs())
            .fold(0.0f32, f32::max);

        if max_amplitude > 0.0 {
            samples.iter().map(|&s| s / max_amplitude).collect()
        } else {
            samples.to_vec()
        }
    }

    pub fn extract_features(samples: &[f32], sample_rate: u32) -> AudioFeatures {
        // Simple feature extraction
        let rms = (samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        let zero_crossings = samples
            .windows(2)
            .filter(|w| (w[0] > 0.0) != (w[1] > 0.0))
            .count();

        AudioFeatures {
            rms_energy: rms,
            zero_crossing_rate: zero_crossings as f32 / samples.len() as f32,
            duration_seconds: samples.len() as f32 / sample_rate as f32,
        }
    }

    #[derive(Debug, Clone)]
    pub struct AudioFeatures {
        pub rms_energy: f32,
        pub zero_crossing_rate: f32,
        pub duration_seconds: f32,
    }
}

pub mod json {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use std::path::Path;

    pub async fn read_json_file<T>(path: &Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let content = tokio::fs::read_to_string(path).await?;
        let data: T = serde_json::from_str(&content)?;
        Ok(data)
    }

    pub async fn write_json_file<T>(path: &Path, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json_string = serde_json::to_string_pretty(data)?;
        tokio::fs::write(path, json_string).await?;
        Ok(())
    }

    pub async fn append_jsonl_file<T>(path: &Path, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json_line = serde_json::to_string(data)? + "\n";

        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        file.write_all(json_line.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    pub async fn read_jsonl_file<T>(path: &Path) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let content = tokio::fs::read_to_string(path).await?;
        let mut results = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() {
                let item: T = serde_json::from_str(line)?;
                results.push(item);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_text_io() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let test_content = "Hello, world!\nThis is a test.";
        text::write_text_file(&file_path, test_content).await.unwrap();

        let read_content = text::read_text_file(&file_path).await.unwrap();
        assert_eq!(test_content, read_content);
    }

    #[tokio::test]
    async fn test_json_io() {
        use serde_json::json;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");

        let test_data = json!({
            "name": "test",
            "value": 42,
            "items": [1, 2, 3]
        });

        json::write_json_file(&file_path, &test_data).await.unwrap();
        let read_data: serde_json::Value = json::read_json_file(&file_path).await.unwrap();

        assert_eq!(test_data, read_data);
    }

    #[test]
    fn test_format_display() {
        assert_eq!(InputFormat::Text.to_string(), "text");
        assert_eq!(InputFormat::Image.to_string(), "image");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::JsonLines.to_string(), "jsonl");
    }
}