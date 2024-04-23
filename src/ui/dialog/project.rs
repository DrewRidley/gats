use std::default;

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{self, Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use sqlx::{MySql, MySqlPool, Pool};

use crate::{crud::fetch_members_by_project_id, Member};

use super::error::{self, DisplayWindow};

pub struct ProjectMembersDialog {
    cursor: usize, // Vertical cursor index only
    members: Vec<Member>,
    new_id: String,
}

impl ProjectMembersDialog {
    fn new() -> Self {
        Self {
            cursor: 0,
            members: vec![],
            new_id: String::new(),
        }
    }
    async fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        pool: &MySqlPool,
        project_id: i32,
    ) -> std::io::Result<bool> {
        let refresh_needed = match key_event.code {
            KeyCode::Down => {
                self.cursor = (self.cursor + 1) % (self.members.len() + 1);
                false
            }
            KeyCode::Up => {
                self.cursor = if self.cursor > 0 {
                    self.cursor - 1
                } else {
                    self.members.len()
                };
                false
            }
            KeyCode::Char(c) if self.cursor == self.members.len() => {
                self.new_id.push(c);
                false
            }
            KeyCode::Backspace if self.cursor == self.members.len() => {
                self.new_id.pop();
                false
            }
            KeyCode::Enter if self.cursor == self.members.len() => {
                let new_member = self.new_id.clone();
                let _ = sqlx::query!(
                    "INSERT INTO ContributesTo (MemberID, ProjectID) VALUES (?, ?)",
                    new_member,
                    project_id
                )
                .execute(pool)
                .await;
                self.new_id.clear();
                true // Indicates the need for refresh
            }
            KeyCode::Char('d') if self.cursor < self.members.len() => {
                let member_id = self.members[self.cursor].member_id;
                let _ = sqlx::query!(
                    "DELETE FROM ContributesTo WHERE MemberID = ? AND ProjectID = ?",
                    member_id,
                    project_id
                )
                .execute(pool)
                .await;
                true // Indicates the need for refresh
            }
            KeyCode::Esc => return Ok(false),
            _ => false,
        };

        if refresh_needed {
            self.members = fetch_members_by_project_id(pool, project_id)
                .await
                .expect("Failed to update members after change was applied...");
            self.cursor = 0;
        }

        Ok(true)
    }

    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
        project_id: i32,
    ) -> std::io::Result<()> {
        let mut diag = ProjectMembersDialog::new();
        diag.members = fetch_members_by_project_id(pool, project_id)
            .await
            .expect("Failed to fetch members!");

        loop {
            diag.draw(&mut terminal)?;

            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    if !diag.handle_key_event(key_event, pool, project_id).await? {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|frame| {
            let items: Vec<ListItem> = self
                .members
                .iter()
                .map(|member| {
                    let text = format!(
                        "{} {} - {}",
                        member.first_name, member.last_name, member.email
                    );
                    ListItem::new(text)
                })
                .collect();

            let new_member_item = ListItem::new(format!("New Member (Enter ID): {}", self.new_id))
                .style(Style::default().fg(Color::Yellow));

            let list = List::new(
                items
                    .into_iter()
                    .chain(std::iter::once(new_member_item))
                    .collect::<Vec<_>>(),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Project Members"),
            )
            .highlight_style(Style::default().fg(Color::Yellow));

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(frame.size());

            let mut list_state = ListState::default();
            list_state.select(Some(self.cursor));
            frame.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;

        Ok(())
    }
}

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

                                match sqlx::query(query)
                                    .bind(&diag.name)
                                    .bind(&diag.desc)
                                    .execute(pool)
                                    .await
                                {
                                    Ok(_) => {
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        DisplayWindow::run(
                                            terminal,
                                            format!("Failed to create project: {}", e),
                                        )
                                        .await;
                                        return Ok(());
                                    }
                                }
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
