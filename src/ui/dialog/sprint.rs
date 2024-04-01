

use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use sqlx::MySqlPool;

pub struct CreateSprintDialog {
    cursor: usize,
    title: String,
    start_date: String,
    end_date: String,
}

impl CreateSprintDialog {
    fn new() -> Self {
        Self {
            cursor: 0,
            title: String::new(),
            start_date: String::new(),
            end_date: String::new(),
        }
    }

    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
        id: i32,
    ) -> std::io::Result<()> {
        let mut diag = CreateSprintDialog::new();

        loop {
            diag.draw(&mut terminal)?;

            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(c) => match diag.cursor {
                            0 => diag.title.push(c),
                            1 => diag.start_date.push(c),
                            2 => diag.end_date.push(c),
                            _ => {}
                        },
                        KeyCode::Enter => {
                            if diag.cursor == 3 {
                                // CREATE SPRINT
                                // Insert the sprint into the SPRINT table
                                let sprint_query = "INSERT INTO SPRINT (Title, startDate, endDate) VALUES (?, ?, ?)";
                                let sprint_result = sqlx::query(sprint_query)
                                    .bind(&diag.title)
                                    .bind(&diag.start_date)
                                    .bind(&diag.end_date)
                                    .execute(pool)
                                    .await;

                                    let last_sprint_id = match sprint_result {
                                        Ok(result) => result.last_insert_id(),
                                        Err(e) => {
                                            eprintln!("Failed to insert sprint: {}", e);
                                            return Ok(()); // Or return an error indicating that the insertion failed
                                        }
                                    };

                                    let project_sprint_query = "INSERT INTO ProjectSprint (ProjectID, SprintID) VALUES (?, ?)";
                                    sqlx::query(project_sprint_query)
                                        .bind(id)
                                        .bind(last_sprint_id)
                                        .execute(pool)
                                        .await.expect("Failed to associated sprint with project! Sprint is now oprhaned...");      

                                return Ok(())
                            }
                        }
                        KeyCode::Backspace => match diag.cursor {
                            0 => {
                                diag.title.pop();
                            }
                            1 => {
                                diag.start_date.pop();
                            }
                            2 => {
                                diag.end_date.pop();
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
                ListItem::new(format!("Sprint Name: {}", self.title)),
                ListItem::new(format!("Start Date: (YYYY-MM-DD): {}", self.start_date)),
                ListItem::new(format!("End Date: (YYYY-MM-DD): {}", self.end_date)),
                ListItem::new("Create Sprint"),
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
