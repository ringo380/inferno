use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    ModelLoaded(String),
    InferenceProgress(u32),
    InferenceComplete(String),
    InferenceError(String),
    Quit,
}

#[allow(dead_code)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
}

#[allow(dead_code)]
impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.sender.clone()
    }

    pub async fn next_event(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }

    pub async fn poll_terminal_events(&self) -> Result<()> {
        loop {
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        let _ = self.sender.send(AppEvent::Key(key));
                    }
                    _ => {}
                }
            }

            // Send tick events periodically
            let _ = self.sender.send(AppEvent::Tick);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub fn emit_model_loaded(&self, model_name: String) {
        let _ = self.sender.send(AppEvent::ModelLoaded(model_name));
    }

    pub fn emit_inference_progress(&self, tokens: u32) {
        let _ = self.sender.send(AppEvent::InferenceProgress(tokens));
    }

    pub fn emit_inference_complete(&self, output: String) {
        let _ = self.sender.send(AppEvent::InferenceComplete(output));
    }

    pub fn emit_inference_error(&self, error: String) {
        let _ = self.sender.send(AppEvent::InferenceError(error));
    }

    pub fn emit_quit(&self) {
        let _ = self.sender.send(AppEvent::Quit);
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
