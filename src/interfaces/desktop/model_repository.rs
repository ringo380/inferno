use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExternalModelInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub model_type: String,
    pub size_bytes: Option<u64>,
    pub download_url: String,
    pub repository_url: String,
    pub license: String,
    pub downloads: u32,
    pub likes: u32,
    pub created_at: String,
    pub updated_at: String,
    pub file_info: Vec<ModelFileInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelFileInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub download_url: String,
    pub file_type: String, // "gguf", "onnx", "safetensors", etc.
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelSearchQuery {
    pub query: Option<String>,
    pub task: Option<String>,
    pub tags: Vec<String>,
    pub sort: String,      // "downloads", "likes", "created", "updated"
    pub direction: String, // "asc", "desc"
    pub limit: u32,
    pub offset: u32,
}

impl Default for ModelSearchQuery {
    fn default() -> Self {
        Self {
            query: None,
            task: None,
            tags: vec![],
            sort: "downloads".to_string(),
            direction: "desc".to_string(),
            limit: 20,
            offset: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelSearchResponse {
    pub models: Vec<ExternalModelInfo>,
    pub total: u32,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadProgress {
    pub download_id: String,
    pub model_id: String,
    pub filename: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress_percent: f64,
    pub status: String, // "downloading", "completed", "failed", "paused"
    pub error_message: Option<String>,
    pub download_speed_bps: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

pub struct ModelRepositoryService {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl ModelRepositoryService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://huggingface.co".to_string(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub async fn search_models(&self, query: ModelSearchQuery) -> Result<ModelSearchResponse> {
        let mut url = format!("{}/api/models", self.base_url);
        let mut params = vec![];

        if let Some(search_query) = &query.query {
            params.push(("search", search_query.as_str()));
        }

        if let Some(task) = &query.task {
            params.push(("pipeline_tag", task.as_str()));
        }

        for tag in &query.tags {
            params.push(("tag", tag.as_str()));
        }

        params.push(("sort", query.sort.as_str()));
        params.push(("direction", query.direction.as_str()));
        let limit_str = query.limit.to_string();
        params.push(("limit", &limit_str));

        // Build URL with query parameters
        if !params.is_empty() {
            url.push('?');
            url.push_str(
                &params
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to search models: {}", response.status()));
        }

        // Parse the response from Hugging Face API
        let raw_models: serde_json::Value = response.json().await?;
        let models = self.parse_huggingface_models(raw_models)?;

        Ok(ModelSearchResponse {
            total: models.len() as u32,
            has_more: models.len() == query.limit as usize,
            models,
        })
    }

    pub async fn get_model_details(&self, model_id: &str) -> Result<ExternalModelInfo> {
        let url = format!("{}/api/models/{}", self.base_url, model_id);

        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get model details: {}",
                response.status()
            ));
        }

        let raw_model: serde_json::Value = response.json().await?;
        self.parse_huggingface_model(raw_model)
    }

    pub async fn get_featured_models(&self) -> Result<Vec<ExternalModelInfo>> {
        let query = ModelSearchQuery {
            query: None,
            task: None,
            tags: vec!["featured".to_string()],
            sort: "downloads".to_string(),
            direction: "desc".to_string(),
            limit: 10,
            offset: 0,
        };

        let response = self.search_models(query).await?;
        Ok(response.models)
    }

    pub async fn get_trending_models(&self) -> Result<Vec<ExternalModelInfo>> {
        let query = ModelSearchQuery {
            query: None,
            task: None,
            tags: vec![],
            sort: "created".to_string(),
            direction: "desc".to_string(),
            limit: 10,
            offset: 0,
        };

        let response = self.search_models(query).await?;
        Ok(response.models)
    }

    fn parse_huggingface_models(
        &self,
        raw_data: serde_json::Value,
    ) -> Result<Vec<ExternalModelInfo>> {
        let models_array = raw_data
            .as_array()
            .ok_or_else(|| anyhow!("Invalid response format: expected array"))?;

        let mut models = Vec::new();
        for model_data in models_array {
            match self.parse_huggingface_model(model_data.clone()) {
                Ok(model) => models.push(model),
                Err(e) => {
                    // Log error but continue processing other models
                    eprintln!("Failed to parse model: {}", e);
                    continue;
                }
            }
        }

        Ok(models)
    }

    fn parse_huggingface_model(&self, raw_model: serde_json::Value) -> Result<ExternalModelInfo> {
        let id = raw_model["id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing model id"))?
            .to_string();

        let name = raw_model["id"]
            .as_str()
            .unwrap_or("Unknown Model")
            .to_string();
        let author = raw_model["author"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let description = raw_model["description"].as_str().unwrap_or("").to_string();

        let tags = raw_model["tags"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let model_type = raw_model["pipeline_tag"]
            .as_str()
            .unwrap_or("text-generation")
            .to_string();
        let downloads = raw_model["downloads"].as_u64().unwrap_or(0) as u32;
        let likes = raw_model["likes"].as_u64().unwrap_or(0) as u32;
        let license = raw_model["license"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let created_at = raw_model["createdAt"]
            .as_str()
            .unwrap_or(&chrono::Utc::now().to_rfc3339())
            .to_string();
        let updated_at = raw_model["lastModified"]
            .as_str()
            .unwrap_or(&chrono::Utc::now().to_rfc3339())
            .to_string();

        // Parse file information
        let file_info = self.parse_model_files(&raw_model, &id)?;

        let download_url = format!("{}/{}", self.base_url, id);
        let repository_url = format!("{}/{}", self.base_url, id);

        // Calculate total size from files
        let size_bytes = file_info.iter().map(|f| f.size_bytes).sum::<u64>();

        Ok(ExternalModelInfo {
            id,
            name,
            author,
            description,
            tags,
            model_type,
            size_bytes: if size_bytes > 0 {
                Some(size_bytes)
            } else {
                None
            },
            download_url,
            repository_url,
            license,
            downloads,
            likes,
            created_at,
            updated_at,
            file_info,
        })
    }

    fn parse_model_files(
        &self,
        raw_model: &serde_json::Value,
        model_id: &str,
    ) -> Result<Vec<ModelFileInfo>> {
        let mut files = Vec::new();

        // Check if files are in the siblings array
        if let Some(siblings) = raw_model["siblings"].as_array() {
            for file_data in siblings {
                if let Some(filename) = file_data["rfilename"].as_str() {
                    // Filter for supported model formats
                    let file_extension = filename.split('.').last().unwrap_or("").to_lowercase();
                    if matches!(
                        file_extension.as_str(),
                        "gguf" | "onnx" | "safetensors" | "bin" | "pt"
                    ) {
                        let size_bytes = file_data["size"].as_u64().unwrap_or(0);
                        let download_url =
                            format!("{}/{}/resolve/main/{}", self.base_url, model_id, filename);

                        files.push(ModelFileInfo {
                            filename: filename.to_string(),
                            size_bytes,
                            download_url,
                            file_type: file_extension,
                        });
                    }
                }
            }
        }

        // If no files found in siblings, create a default entry
        if files.is_empty() {
            files.push(ModelFileInfo {
                filename: "model.bin".to_string(),
                size_bytes: 0,
                download_url: format!("{}/{}/resolve/main/model.bin", self.base_url, model_id),
                file_type: "bin".to_string(),
            });
        }

        Ok(files)
    }
}

#[derive(Clone)]
pub struct ModelDownloadManager {
    downloads: std::sync::Arc<std::sync::Mutex<HashMap<String, DownloadProgress>>>,
    client: reqwest::Client,
}

impl ModelDownloadManager {
    pub fn new() -> Self {
        Self {
            downloads: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub async fn start_download(
        &self,
        model: &ExternalModelInfo,
        target_dir: &str,
    ) -> Result<String> {
        let download_id = Uuid::new_v4().to_string();

        // For now, download the first available file
        let file_to_download = model
            .file_info
            .first()
            .ok_or_else(|| anyhow!("No downloadable files found for model"))?;

        let progress = DownloadProgress {
            download_id: download_id.clone(),
            model_id: model.id.clone(),
            filename: file_to_download.filename.clone(),
            downloaded_bytes: 0,
            total_bytes: file_to_download.size_bytes,
            progress_percent: 0.0,
            status: "starting".to_string(),
            error_message: None,
            download_speed_bps: None,
            eta_seconds: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
        };

        // Store initial progress
        {
            let mut downloads = self.downloads.lock().unwrap();
            downloads.insert(download_id.clone(), progress);
        }

        // Start download in background
        let downloads_ref = self.downloads.clone();
        let client = self.client.clone();
        let download_url = file_to_download.download_url.clone();
        let target_path = format!("{}/{}", target_dir, file_to_download.filename);
        let download_id_clone = download_id.clone();

        tokio::spawn(async move {
            let result = Self::download_file_with_progress(
                client,
                download_url,
                target_path,
                download_id_clone.clone(),
                downloads_ref.clone(),
            )
            .await;

            // Update final status
            if let Ok(mut downloads) = downloads_ref.lock() {
                if let Some(progress) = downloads.get_mut(&download_id_clone) {
                    match result {
                        Ok(_) => {
                            progress.status = "completed".to_string();
                            progress.progress_percent = 100.0;
                            progress.completed_at = Some(chrono::Utc::now().to_rfc3339());
                        }
                        Err(e) => {
                            progress.status = "failed".to_string();
                            progress.error_message = Some(e.to_string());
                        }
                    }
                }
            }
        });

        Ok(download_id)
    }

    pub fn get_download_progress(&self, download_id: &str) -> Option<DownloadProgress> {
        self.downloads.lock().ok()?.get(download_id).cloned()
    }

    pub fn get_all_downloads(&self) -> Vec<DownloadProgress> {
        self.downloads
            .lock()
            .map(|downloads| downloads.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn cancel_download(&self, download_id: &str) -> bool {
        if let Ok(mut downloads) = self.downloads.lock() {
            if let Some(progress) = downloads.get_mut(download_id) {
                progress.status = "cancelled".to_string();
                return true;
            }
        }
        false
    }

    pub fn clear_completed_downloads(&self) {
        if let Ok(mut downloads) = self.downloads.lock() {
            downloads.retain(|_, progress| progress.status != "completed");
        }
    }

    async fn download_file_with_progress(
        client: reqwest::Client,
        url: String,
        target_path: String,
        download_id: String,
        downloads: std::sync::Arc<std::sync::Mutex<HashMap<String, DownloadProgress>>>,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // Update status to downloading
        {
            if let Ok(mut downloads_map) = downloads.lock() {
                if let Some(progress) = downloads_map.get_mut(&download_id) {
                    progress.status = "downloading".to_string();
                }
            }
        }

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let total_size = response.content_length().unwrap_or(0);

        // Update total size if we got it from headers
        {
            if let Ok(mut downloads_map) = downloads.lock() {
                if let Some(progress) = downloads_map.get_mut(&download_id) {
                    if total_size > 0 {
                        progress.total_bytes = total_size;
                    }
                }
            }
        }

        let mut file = tokio::fs::File::create(&target_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let start_time = std::time::Instant::now();

        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Update progress every 1MB or 5% progress
            if downloaded % (1024 * 1024) == 0
                || downloaded * 20 / total_size
                    != (downloaded - chunk.len() as u64) * 20 / total_size
            {
                let elapsed = start_time.elapsed().as_secs();
                let speed = if elapsed > 0 { downloaded / elapsed } else { 0 };
                let eta = if speed > 0 && total_size > downloaded {
                    Some((total_size - downloaded) / speed)
                } else {
                    None
                };

                if let Ok(mut downloads_map) = downloads.lock() {
                    if let Some(progress) = downloads_map.get_mut(&download_id) {
                        // Check if cancelled
                        if progress.status == "cancelled" {
                            return Err(anyhow!("Download cancelled"));
                        }

                        progress.downloaded_bytes = downloaded;
                        progress.progress_percent = if total_size > 0 {
                            (downloaded as f64 / total_size as f64) * 100.0
                        } else {
                            0.0
                        };
                        progress.download_speed_bps = Some(speed);
                        progress.eta_seconds = eta;
                    }
                }
            }
        }

        file.flush().await?;
        Ok(())
    }
}
