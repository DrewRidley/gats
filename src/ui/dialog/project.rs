use crate::{crud::delete_project_by_id, Project};
use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Terminal,
};
use sqlx::MySqlPool;

pub struct CreateProjectDialog {
    cursor: usize,
    name: String,
    desc: String,
}

impl CreateProjectDialog {
    fn new() -> Self {
        Self {
            cursor: 0,
            name: String::new(),
            desc: String::new(),
        }
    }

    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
    ) -> std::io::Result<()> {
        let mut diag = CreateProjectDialog::new();

        loop {
            diag.draw(&mut terminal)?;

            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(c) => match diag.cursor {
                            0 => diag.name.push(c),
                            1 => diag.desc.push(c),
                            _ => {}
                        },
                        KeyCode::Enter => {
                            if diag.cursor == 2 {
                                //Actually create the project and return.
                                let query =
                                    "INSERT INTO Project (Title, Description) VALUES (?, ?)";
                                sqlx::query(query)
                                    .bind(&diag.name)
                                    .bind(&diag.desc)
                                    .execute(pool)
                                    .await
                                    .expect("Failed to insert project into database!");

                                return Ok(());
                            }
                        }
                        KeyCode::Backspace => match diag.cursor {
                            0 => {
                                diag.name.pop();
                            }
                            1 => {
                                diag.desc.pop();
                            }
                            _ => {}
                        },
                        KeyCode::Down => {
                            diag.cursor = (diag.cursor + 1) % 5;
                        }
                        KeyCode::Up => {
                            diag.cursor = if diag.cursor > 0 { diag.cursor - 1 } else { 3 };
                        }
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|frame| {
            let list = List::new(vec![
                ListItem::new(format!("Project Name: {}", self.name)),
                ListItem::new(format!("Description: {}", self.desc)),
                ListItem::new("Create Project"),
            ]);

            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(frame.size());

            let mut list_state = ListState::default();
            list_state.select(Some(self.cursor));

            let action_list = list
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Create new Project")
                        .title_alignment(ratatui::layout::Alignment::Center),
                )
                .highlight_symbol(">")
                .highlight_style(
                    ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                );

            frame.render_stateful_widget(action_list, chunks[0], &mut list_state);
        })?;

        Ok(())
    }
}
