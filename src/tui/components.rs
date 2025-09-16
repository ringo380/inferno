use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub struct ProgressBar {
    pub label: String,
    pub progress: f64,
    pub color: Color,
}

impl ProgressBar {
    pub fn new(label: String, progress: f64) -> Self {
        let color = if progress < 0.3 {
            Color::Red
        } else if progress < 0.7 {
            Color::Yellow
        } else {
            Color::Green
        };

        Self {
            label,
            progress,
            color,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(self.label.clone()))
            .gauge_style(Style::default().fg(self.color))
            .percent((self.progress * 100.0) as u16)
            .label(format!("{:.1}%", self.progress * 100.0));

        f.render_widget(gauge, area);
    }
}

#[allow(dead_code)]
pub struct StatusDisplay {
    pub items: Vec<(String, String)>,
    pub title: String,
}

#[allow(dead_code)]
impl StatusDisplay {
    pub fn new(title: String) -> Self {
        Self {
            items: Vec::new(),
            title,
        }
    }

    pub fn add_item(&mut self, key: String, value: String) {
        self.items.push((key, value));
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let text = self.items
            .iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect::<Vec<_>>()
            .join("\n");

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(self.title.clone()))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

#[allow(dead_code)]
pub struct ModelCard {
    pub name: String,
    pub path: String,
    pub size: String,
    pub backend: String,
    pub is_loaded: bool,
}

#[allow(dead_code)]
impl ModelCard {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let status_indicator = if self.is_loaded { "●" } else { "○" };
        let status_color = if self.is_loaded { Color::Green } else { Color::Gray };

        let content = format!(
            "{} {}\n\
             Backend: {}\n\
             Size: {}\n\
             Path: {}",
            status_indicator, self.name, self.backend, self.size, self.path
        );

        let card = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Model Info ")
                    .border_style(Style::default().fg(status_color))
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(card, area);
    }
}

#[allow(dead_code)]
pub struct TokenStream {
    pub tokens: Vec<String>,
    pub max_display: usize,
}

#[allow(dead_code)]
impl TokenStream {
    pub fn new(max_display: usize) -> Self {
        Self {
            tokens: Vec::new(),
            max_display,
        }
    }

    pub fn add_token(&mut self, token: String) {
        self.tokens.push(token);
        if self.tokens.len() > self.max_display {
            self.tokens.remove(0);
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.tokens
            .iter()
            .enumerate()
            .map(|(i, token)| {
                let style = if i == self.tokens.len() - 1 {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(token.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Token Stream "));

        f.render_widget(list, area);
    }
}