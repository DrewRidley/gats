use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{backend::Backend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, Borders, List, ListItem, ListState}, Terminal};
use sqlx::MySqlPool;



/// A generic dialog used to create records with 'entries' fields.
pub struct CreateRecordDialog {
    entries: Vec<String>,
    labels: Vec<String>,
    validate: fn(&CreateRecordDialog) -> bool,
    cursor: usize,
}

pub enum CreateResults {
    Create(Vec<String>),
    Quit,
}

impl CreateRecordDialog {
    pub fn new(labels: Vec<String>, validator: fn(&CreateRecordDialog) -> bool) -> Self {
        let entries = vec![String::new(); labels.len()];
        CreateRecordDialog {
            labels,
            entries,
            validate: validator,
            cursor: 0,
        }
    }

    pub fn new_edit(labels: Vec<String>, current_data: Vec<String>, validator: fn(&CreateRecordDialog) -> bool) -> Self {
        CreateRecordDialog {
            labels,
            entries: current_data,
            validate: validator,
            cursor: 0,
        }
    }

    pub async fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        pool: &MySqlPool,
    ) -> std::io::Result<CreateResults> {
        loop {
            self.draw(terminal)?;

            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Char(c) => {
                        self.entries[self.cursor].push(c);
                    },
                    KeyCode::Enter => {
                        if self.cursor == self.entries.len() { 
                            if (self.validate)(&self) {
                                return Ok(CreateResults::Create(self.entries.clone()));
                            } else {
                                // Do nothing until we have some way to visualize an error.
                            }
                        } else {
                            // Move cursor to submit button or wrap around
                            self.cursor = (self.cursor + 1) % (self.entries.len() + 1);
                        }
                    },
                    KeyCode::Backspace => {
                        if !self.entries[self.cursor].is_empty() {
                            self.entries[self.cursor].pop();
                        }
                    },
                    KeyCode::Down => {
                        self.cursor = (self.cursor + 1) % (self.entries.len() + 1);
                    },
                    KeyCode::Up => {
                        if self.cursor == 0 {
                            self.cursor = self.entries.len(); // Wrap to submit option
                        } else {
                            self.cursor -= 1;
                        }
                    },
                    KeyCode::Esc => {
                        return Ok(CreateResults::Quit);
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        terminal.draw(|frame| {
            let items: Vec<_> = self.labels.iter().zip(self.entries.iter()).map(|(label, entry)| {
                ListItem::new(format!("{}: {}", label, entry))
            }).collect();

            let items = items.into_iter().chain(std::iter::once(ListItem::new("Submit"))).collect::<Vec<_>>();

            let list = List::new(items)
                .block(Block::default().title("Create or Edit Record:").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
                .highlight_symbol(">>");

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(frame.size());

            let mut state = ListState::default();
            state.select(Some(self.cursor));
            frame.render_stateful_widget(list, chunks[0], &mut state);
        })?;

        Ok(())
    }
}