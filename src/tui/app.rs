use crate::{
    backends::{Backend, BackendType, InferenceParams},
    config::Config,
    models::{ModelInfo, ModelManager},
    tui::components::ProgressBar,
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    ModelSelection,
    Loading,
    InputPrompt,
    Running,
    ViewingOutput,
    #[allow(dead_code)]
    Help,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug)]
pub enum StreamMessage {
    Token(String),
    Error(String),
    Complete,
}

pub struct App {
    config: Config,
    #[allow(dead_code)]
    model_manager: ModelManager,
    models: Vec<ModelInfo>,
    selected_model: Option<usize>,
    model_list_state: ListState,

    backend: Option<Arc<Mutex<Backend>>>,
    loaded_model: Option<ModelInfo>,

    state: AppState,
    input_buffer: String,
    output_buffer: String,
    logs: VecDeque<LogEntry>,

    show_help: bool,
    inference_stats: InferenceStats,
    loading_progress: f64,
    streaming_tokens: Vec<String>,

    // Streaming channels
    stream_receiver: Option<mpsc::UnboundedReceiver<StreamMessage>>,
    inference_start_time: Option<std::time::Instant>,
}

#[derive(Debug, Default)]
pub struct InferenceStats {
    pub tokens_generated: u32,
    pub time_elapsed: Duration,
    pub tokens_per_second: f32,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let model_manager = ModelManager::new(&config.models_dir);
        let models = model_manager.list_models().await?;

        let mut app = Self {
            config,
            model_manager,
            models,
            selected_model: None,
            model_list_state: ListState::default(),

            backend: None,
            loaded_model: None,

            state: AppState::ModelSelection,
            input_buffer: String::new(),
            output_buffer: String::new(),
            logs: VecDeque::with_capacity(100),

            show_help: false,
            inference_stats: InferenceStats::default(),
            loading_progress: 0.0,
            streaming_tokens: Vec::new(),

            stream_receiver: None,
            inference_start_time: None,
        };

        if !app.models.is_empty() {
            app.model_list_state.select(Some(0));
            app.selected_model = Some(0);
        }

        app.add_log("info", "Inferno TUI initialized");
        Ok(app)
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(f.size());

        // Header
        self.draw_header(f, main_chunks[0]);

        // Main content area
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left panel
                Constraint::Percentage(70), // Right panel
            ])
            .split(main_chunks[1]);

        // Left panel (models and logs)
        self.draw_left_panel(f, content_chunks[0]);

        // Right panel (input/output)
        self.draw_right_panel(f, content_chunks[1]);

        // Status bar
        self.draw_status_bar(f, main_chunks[2]);

        // Help overlay
        if self.show_help {
            self.draw_help_overlay(f);
        }
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = format!(
            " Inferno AI/ML Runner v{} ",
            std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
        );

        let header = Paragraph::new(title)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(header, area);
    }

    fn draw_left_panel(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Models
                Constraint::Percentage(40), // Logs
            ])
            .split(area);

        // Models list
        self.draw_models_list(f, chunks[0]);

        // Logs
        self.draw_logs(f, chunks[1]);
    }

    fn draw_models_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .models
            .iter()
            .enumerate()
            .map(|(i, model)| {
                let style = if Some(i) == self.selected_model {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let indicator = if self.loaded_model.as_ref().map(|m| &m.name) == Some(&model.name)
                {
                    "● "
                } else {
                    "  "
                };

                ListItem::new(format!("{}{}", indicator, model.name)).style(style)
            })
            .collect();

        let models_list = List::new(items)
            .block(
                Block::default()
                    .title(" Models ")
                    .borders(Borders::ALL)
                    .border_style(if self.state == AppState::ModelSelection {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(models_list, area, &mut self.model_list_state);
    }

    fn draw_logs(&self, f: &mut Frame, area: Rect) {
        let log_items: Vec<ListItem> = self
            .logs
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
            .map(|log| {
                let style = match log.level.as_str() {
                    "error" => Style::default().fg(Color::Red),
                    "warn" => Style::default().fg(Color::Yellow),
                    "info" => Style::default().fg(Color::Green),
                    _ => Style::default(),
                };

                ListItem::new(format!(
                    "{} [{}] {}",
                    log.timestamp.format("%H:%M:%S"),
                    log.level.to_uppercase(),
                    log.message
                ))
                .style(style)
            })
            .rev()
            .collect();

        let logs_list =
            List::new(log_items).block(Block::default().title(" Logs ").borders(Borders::ALL));

        f.render_widget(logs_list, area);
    }

    fn draw_right_panel(&self, f: &mut Frame, area: Rect) {
        match self.state {
            AppState::ModelSelection => {
                self.draw_model_info(f, area);
            }
            AppState::Loading => {
                self.draw_loading_progress(f, area);
            }
            AppState::InputPrompt => {
                self.draw_input_area(f, area);
            }
            AppState::Running => {
                self.draw_inference_progress(f, area);
            }
            AppState::ViewingOutput => {
                self.draw_output_area(f, area);
            }
            AppState::Help => {
                // Help is drawn as overlay
            }
        }
    }

    fn draw_model_info(&self, f: &mut Frame, area: Rect) {
        let content = if let Some(selected) = self.selected_model {
            if let Some(model) = self.models.get(selected) {
                format!(
                    "Model: {}\n\
                     Path: {}\n\
                     Type: {}\n\
                     Size: {:.1} MB\n\
                     Modified: {}\n\n\
                     Press Enter to load this model\n\
                     Press 'i' to enter input prompt\n\
                     Press 'h' for help",
                    model.name,
                    model.path.display(),
                    model.backend_type,
                    model.size as f64 / 1024.0 / 1024.0,
                    model.modified.format("%Y-%m-%d %H:%M:%S")
                )
            } else {
                "No model selected".to_string()
            }
        } else {
            "No models available.\n\nPlace GGUF (*.gguf) or ONNX (*.onnx) files\nin the models directory to get started.".to_string()
        };

        let info = Paragraph::new(content)
            .block(Block::default().title(" Model Info ").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(info, area);
    }

    fn draw_input_area(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Input
                Constraint::Length(8), // Instructions
            ])
            .split(area);

        let input = Paragraph::new(self.input_buffer.as_str())
            .block(
                Block::default()
                    .title(" Input Prompt ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(input, chunks[0]);

        let instructions = Paragraph::new(
            "Enter your prompt above.\n\n\
             Controls:\n\
             - Type to enter prompt\n\
             - Enter: Run inference\n\
             - Esc: Back to model selection\n\
             - Ctrl+C: Quit",
        )
        .block(
            Block::default()
                .title(" Instructions ")
                .borders(Borders::ALL),
        );

        f.render_widget(instructions, chunks[1]);
    }

    fn draw_inference_progress(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Stats
                Constraint::Min(3),    // Streaming output
            ])
            .split(area);

        // Stats section
        let stats_content = format!(
            "Running inference...\n\n\
             Model: {}\n\
             Tokens generated: {}\n\
             Time elapsed: {:.1}s\n\
             Speed: {:.1} tokens/sec",
            self.loaded_model
                .as_ref()
                .map(|m| &m.name)
                .unwrap_or(&"Unknown".to_string()),
            self.inference_stats.tokens_generated,
            self.inference_stats.time_elapsed.as_secs_f32(),
            self.inference_stats.tokens_per_second
        );

        let stats = Paragraph::new(stats_content).block(
            Block::default()
                .title(" Inference Stats ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(stats, chunks[0]);

        // Streaming output section
        let output_content = if !self.output_buffer.is_empty() {
            self.output_buffer.as_str()
        } else {
            "Waiting for first token..."
        };

        let streaming_output = Paragraph::new(output_content)
            .block(
                Block::default()
                    .title(" Live Output (Press Esc to cancel) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(streaming_output, chunks[1]);
    }

    fn draw_loading_progress(&self, f: &mut Frame, area: Rect) {
        let progress_bar = ProgressBar::new("Loading model...".to_string(), self.loading_progress);

        progress_bar.render(f, area);
    }

    fn draw_output_area(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Output
                Constraint::Length(6), // Controls
            ])
            .split(area);

        let output = Paragraph::new(self.output_buffer.as_str())
            .block(Block::default().title(" Output ").borders(Borders::ALL))
            .wrap(Wrap { trim: false });

        f.render_widget(output, chunks[0]);

        let controls = Paragraph::new(
            "Controls:\n\
             - 'i': New input prompt\n\
             - 'm': Back to model selection\n\
             - Esc: Back to previous screen\n\
             - 'q': Quit",
        )
        .block(Block::default().title(" Controls ").borders(Borders::ALL));

        f.render_widget(controls, chunks[1]);
    }

    fn draw_status_bar(&self, f: &mut Frame, area: Rect) {
        let status = format!(
            " State: {:?} | Models: {} | Loaded: {} | Press 'h' for help ",
            self.state,
            self.models.len(),
            self.loaded_model
                .as_ref()
                .map(|m| &m.name)
                .unwrap_or(&"None".to_string())
        );

        let status_bar = Paragraph::new(status)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .block(Block::default());

        f.render_widget(status_bar, area);
    }

    fn draw_help_overlay(&self, f: &mut Frame) {
        let area = centered_rect(60, 70, f.size());

        f.render_widget(Clear, area);

        let help_text = "Inferno TUI Help\n\n\
             Global Keys:\n\
             - h: Show/hide this help\n\
             - q: Quit application\n\
             - Esc: Go back/cancel\n\n\
             Model Selection:\n\
             - ↑/↓: Navigate models\n\
             - Enter: Load selected model\n\
             - i: Input prompt (if model loaded)\n\n\
             Input Prompt:\n\
             - Type: Enter text\n\
             - Enter: Run inference\n\
             - Esc: Back to model selection\n\n\
             Output View:\n\
             - i: New input prompt\n\
             - m: Back to model selection\n\n\
             Press 'h' again to close help";

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(help, area);
    }

    pub async fn handle_events(&mut self) -> Result<bool> {
        // Use a simple non-blocking approach for event handling
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    return self.handle_key_event(key.code).await;
                }
            }
        }

        Ok(false)
    }

    async fn handle_key_event(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('q') => return Ok(true), // Quit
            KeyCode::Char('h') => {
                self.show_help = !self.show_help;
                return Ok(false);
            }
            _ => {}
        }

        if self.show_help {
            if matches!(key, KeyCode::Esc | KeyCode::Char('h')) {
                self.show_help = false;
            }
            return Ok(false);
        }

        match self.state {
            AppState::ModelSelection => self.handle_model_selection_keys(key).await,
            AppState::Loading => Ok(false), // No user input during loading
            AppState::InputPrompt => self.handle_input_keys(key).await,
            AppState::Running => self.handle_running_keys(key).await,
            AppState::ViewingOutput => self.handle_output_keys(key).await,
            AppState::Help => Ok(false),
        }
    }

    async fn handle_model_selection_keys(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Up => {
                if let Some(selected) = self.selected_model {
                    if selected > 0 {
                        self.selected_model = Some(selected - 1);
                        self.model_list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.selected_model {
                    if selected < self.models.len() - 1 {
                        self.selected_model = Some(selected + 1);
                        self.model_list_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = self.selected_model {
                    self.load_model(selected).await?;
                }
            }
            KeyCode::Char('i') => {
                if self.loaded_model.is_some() {
                    self.state = AppState::InputPrompt;
                    self.input_buffer.clear();
                }
            }
            _ => {}
        }

        Ok(false)
    }

    async fn handle_input_keys(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Enter => {
                if !self.input_buffer.trim().is_empty() {
                    self.run_inference().await?;
                }
            }
            KeyCode::Esc => {
                self.state = AppState::ModelSelection;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }

        Ok(false)
    }

    async fn handle_running_keys(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Esc => {
                // Cancel inference
                self.add_log("warn", "Inference cancelled by user");
                self.state = AppState::InputPrompt;
                self.stream_receiver = None;
                self.inference_start_time = None;
                self.output_buffer.clear();
                self.streaming_tokens.clear();
            }
            _ => {}
        }

        Ok(false)
    }

    async fn handle_output_keys(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('i') => {
                self.state = AppState::InputPrompt;
                self.input_buffer.clear();
            }
            KeyCode::Char('m') => {
                self.state = AppState::ModelSelection;
            }
            KeyCode::Esc => {
                self.state = AppState::InputPrompt;
            }
            _ => {}
        }

        Ok(false)
    }

    pub async fn update(&mut self) -> Result<()> {
        // Handle streaming messages if available
        let mut messages_to_process = Vec::new();

        if let Some(receiver) = &mut self.stream_receiver {
            while let Ok(message) = receiver.try_recv() {
                messages_to_process.push(message);
            }
        }

        // Process messages outside the borrow
        for message in messages_to_process {
            match message {
                StreamMessage::Token(token) => {
                    self.streaming_tokens.push(token.clone());
                    self.output_buffer.push_str(&token);

                    // Update real-time stats
                    if let Some(start_time) = self.inference_start_time {
                        let elapsed = start_time.elapsed();
                        self.inference_stats.tokens_generated = self.streaming_tokens.len() as u32;
                        self.inference_stats.time_elapsed = elapsed;
                        self.inference_stats.tokens_per_second =
                            self.streaming_tokens.len() as f32 / elapsed.as_secs_f32().max(0.001);
                    }
                }
                StreamMessage::Error(error) => {
                    self.add_log("error", &format!("Stream error: {}", error));
                    self.state = AppState::InputPrompt;
                    self.stream_receiver = None;
                    self.inference_start_time = None;
                }
                StreamMessage::Complete => {
                    if let Some(start_time) = self.inference_start_time {
                        let elapsed = start_time.elapsed();
                        self.add_log(
                            "info",
                            &format!(
                                "Streaming inference completed in {:.1}s ({:.1} tok/s)",
                                elapsed.as_secs_f32(),
                                self.inference_stats.tokens_per_second
                            ),
                        );
                    }
                    self.state = AppState::ViewingOutput;
                    self.stream_receiver = None;
                    self.inference_start_time = None;
                }
            }
        }

        Ok(())
    }

    async fn load_model(&mut self, index: usize) -> Result<()> {
        if let Some(model) = self.models.get(index).cloned() {
            self.add_log("info", &format!("Loading model: {}", model.name));

            // Enter loading state
            self.state = AppState::Loading;
            self.loading_progress = 0.0;

            // First, validate the model comprehensively
            let model_manager = crate::models::ModelManager::new(&self.config.models_dir);
            match model_manager
                .validate_model_comprehensive(&model.path, Some(&self.config))
                .await
            {
                Ok(validation_result) => {
                    if !validation_result.is_valid {
                        let error_msg = if validation_result.errors.is_empty() {
                            "Model validation failed".to_string()
                        } else {
                            format!(
                                "Model validation failed: {}",
                                validation_result.errors.join(", ")
                            )
                        };
                        self.add_log("error", &error_msg);
                        self.state = AppState::ModelSelection;
                        return Ok(());
                    }
                    self.add_log("info", "Model validation passed");
                }
                Err(e) => {
                    self.add_log("error", &format!("Model validation error: {}", e));
                    self.state = AppState::ModelSelection;
                    return Ok(());
                }
            }

            self.loading_progress = 0.3;

            let backend_type = match BackendType::from_model_path(&model.path) {
                Some(bt) => bt,
                None => {
                    self.add_log("error", &format!("No suitable backend found for model: {}", model.path.display()));
                    self.state = AppState::ModelSelection;
                    return Ok(());
                }
            };
            let mut backend = match Backend::new(backend_type, &self.config.backend_config) {
                Ok(b) => b,
                Err(e) => {
                    self.add_log("error", &format!("Failed to create backend: {}", e));
                    self.state = AppState::ModelSelection;
                    return Ok(());
                }
            };

            self.loading_progress = 0.7;

            match backend.load_model(&model).await {
                Ok(_) => {
                    self.loading_progress = 1.0;
                    self.backend = Some(Arc::new(Mutex::new(backend)));
                    self.loaded_model = Some(model.clone());
                    self.add_log(
                        "info",
                        &format!("Model loaded successfully: {}", model.name),
                    );
                    self.state = AppState::InputPrompt;
                }
                Err(e) => {
                    self.add_log(
                        "error",
                        &format!("Failed to load model into backend: {}", e),
                    );
                    self.state = AppState::ModelSelection;
                }
            }
        }

        Ok(())
    }

    async fn run_inference(&mut self) -> Result<()> {
        let input_buffer = self.input_buffer.clone();

        self.state = AppState::Running;
        self.streaming_tokens.clear();
        self.output_buffer.clear();
        self.inference_start_time = Some(std::time::Instant::now());
        self.add_log("info", "Starting streaming inference");

        let inference_params = InferenceParams {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            stream: true,
        };

        // Create channel for streaming
        let (sender, receiver) = mpsc::unbounded_channel();
        self.stream_receiver = Some(receiver);

        // Clone backend for the async task
        if let Some(backend_arc) = self.backend.clone() {
            // Spawn async task to handle streaming
            tokio::spawn(async move {
                let stream_result = {
                    let mut backend = backend_arc.lock().await;
                    backend.infer_stream(&input_buffer, &inference_params).await
                };

                match stream_result {
                    Ok(mut token_stream) => {
                        use futures::StreamExt;
                        while let Some(token_result) = token_stream.next().await {
                            match token_result {
                                Ok(token) => {
                                    if sender.send(StreamMessage::Token(token)).is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                                Err(e) => {
                                    let _ = sender.send(StreamMessage::Error(e.to_string()));
                                    break;
                                }
                            }
                        }
                        let _ = sender.send(StreamMessage::Complete);
                    }
                    Err(e) => {
                        let _ = sender.send(StreamMessage::Error(e.to_string()));
                    }
                }
            });

            self.add_log("info", "Inference task started");
        } else {
            self.add_log("error", "No backend available");
            self.state = AppState::InputPrompt;
        }

        Ok(())
    }

    fn estimate_token_count(&self, text: &str) -> u32 {
        (text.len() as f32 / 4.0).ceil() as u32
    }

    fn add_log(&mut self, level: &str, message: &str) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
        };

        self.logs.push_back(log_entry);

        // Keep only the last 100 log entries
        if self.logs.len() > 100 {
            self.logs.pop_front();
        }

        // Also log to the regular logging system
        match level {
            "error" => tracing::error!("{}", message),
            "warn" => warn!("{}", message),
            "info" => info!("{}", message),
            _ => info!("{}", message),
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
