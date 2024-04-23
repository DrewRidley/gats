
use crossterm::{event::{self, read, Event, KeyCode, KeyEventKind}, terminal::enable_raw_mode};
use ratatui::{
    backend::Backend, layout::{Constraint, Layout}, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, Terminal
};

use std::io;
use std::time::{Duration, Instant};
use tokio::time;


pub struct DisplayWindow {
    text: String
}

impl DisplayWindow {
    pub async fn run<B: Backend>(
        terminal: &mut Terminal<B>,
        text: String
    ) -> Result<(), io::Error> {
        let diag = DisplayWindow { text };
        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Ok(())
            }


            terminal.draw(|frame| {
                let size = frame.size();
                let block = Block::default().borders(Borders::ALL).title("Error").title_alignment(ratatui::layout::Alignment::Center);
                let area = Layout::default()
                    .constraints([Constraint::Percentage(100)])
                    .margin(2)
                    .split(size)[0];    
                let combined_text = format!("{}\n\nPress Esc or Enter to return", diag.text.clone());

                // Render the paragraph with the combined text
                let paragraph = Paragraph::new(combined_text)
                    .block(block)
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().fg(Color::White))
                    .wrap(ratatui::widgets::Wrap { trim: true }); // Enable text wrapping
                
                frame.render_widget(paragraph, area);
            })?;



            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => {
                            return Ok(());
                        },
                        KeyCode::Enter => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}