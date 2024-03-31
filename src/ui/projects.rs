use ratatui::{
    backend::Backend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap}, Terminal
};


use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use sqlx::MySqlPool;

use crate::{Project};

#[derive(Clone, PartialEq, Eq)]
enum ProjectManagerCursor {
    AddProject,
    UpdateProject,
    DeleteProject,
    BackToMainMenu,
}

impl ProjectManagerCursor {
    fn next(&mut self) {
        *self = match *self {
            ProjectManagerCursor::AddProject => ProjectManagerCursor::UpdateProject,
            ProjectManagerCursor::UpdateProject => ProjectManagerCursor::DeleteProject,
            ProjectManagerCursor::DeleteProject => ProjectManagerCursor::BackToMainMenu,
            ProjectManagerCursor::BackToMainMenu => ProjectManagerCursor::AddProject,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            ProjectManagerCursor::AddProject => ProjectManagerCursor::BackToMainMenu,
            ProjectManagerCursor::UpdateProject => ProjectManagerCursor::AddProject,
            ProjectManagerCursor::DeleteProject => ProjectManagerCursor::UpdateProject,
            ProjectManagerCursor::BackToMainMenu => ProjectManagerCursor::DeleteProject,
        }
    }
}

pub struct ProjectManager {
    cursor: ProjectManagerCursor,
    projects: Vec<Project>
}

async fn fetch_projects(pool: &MySqlPool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT ProjectID, Title, Description FROM Project")
        .fetch_all(pool)
        .await
}

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager {
            cursor: ProjectManagerCursor::AddProject,
            projects: vec![]
        }
    }

    fn get_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            ProjectManagerCursor::AddProject,
            ProjectManagerCursor::UpdateProject,
            ProjectManagerCursor::DeleteProject,
            ProjectManagerCursor::BackToMainMenu,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    ProjectManagerCursor::AddProject => "Add Project",
                    ProjectManagerCursor::UpdateProject => "Update Project",
                    ProjectManagerCursor::DeleteProject => "Delete Project",
                    ProjectManagerCursor::BackToMainMenu => "Back to Main Menu",
                };

                let is_selected = self.cursor == *item;

                if is_selected {
                    Span::styled(name, highlight_style)
                } else {
                    Span::raw(name)
                }
            })
            .collect()
    }

    pub async fn run(mut terminal: &mut Terminal<impl Backend>, pool: &MySqlPool) -> std::io::Result<()> {
        let mut mgr = Self::new();
        mgr.projects = fetch_projects(pool).await.unwrap();
        loop {
            mgr.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down => mgr.cursor.next(),
                        KeyCode::Up => mgr.cursor.prev(),
                        KeyCode::Enter => {
                            match mgr.cursor {
                                ProjectManagerCursor::AddProject => {
                                    // TODO: Implement add project functionality
                                }
                                ProjectManagerCursor::UpdateProject => {
                                    // TODO: Implement update project functionality
                                }
                                ProjectManagerCursor::DeleteProject => {
                                    // TODO: Implement delete project functionality
                                }
                                ProjectManagerCursor::BackToMainMenu => return Ok(()),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl Widget for &mut ProjectManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // 30% of the screen for the menu
            Constraint::Percentage(50), // 70% of the screen for project details
        ])
        .split(area);

        let block = Block::default()
            .title("Manage Projects")
            .borders(Borders::ALL);

        let text_lines: Vec<Line> = self.get_menu_lines().into_iter().map(|span| Line::from(span)).collect();

        let list = List::new(text_lines)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        list.render(chunks[0], buf);

        let lines: Vec<Line> = self.projects.iter().map(|proj| {
            Line::from(Span::from(format!("Project #{} | {} | {}", proj.ProjectID, proj.Title, proj.Description)))
        }).collect();

        let proj_block = Block::default()
        .title("Projects List")
        .borders(Borders::ALL);


        let list = List::new(lines)
        .block(proj_block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        list.render(chunks[1], buf);
    }
}