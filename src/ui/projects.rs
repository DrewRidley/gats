use ratatui::{
    backend::Backend, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap}, Terminal
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[derive(Clone, PartialEq, Eq)]
enum ProjectManagerCursor {
    ViewProjects,
    AddProject,
    UpdateProject,
    DeleteProject,
    BackToMainMenu,
}

impl ProjectManagerCursor {
    fn next(&mut self) {
        *self = match *self {
            ProjectManagerCursor::ViewProjects => ProjectManagerCursor::AddProject,
            ProjectManagerCursor::AddProject => ProjectManagerCursor::UpdateProject,
            ProjectManagerCursor::UpdateProject => ProjectManagerCursor::DeleteProject,
            ProjectManagerCursor::DeleteProject => ProjectManagerCursor::BackToMainMenu,
            ProjectManagerCursor::BackToMainMenu => ProjectManagerCursor::ViewProjects,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            ProjectManagerCursor::ViewProjects => ProjectManagerCursor::BackToMainMenu,
            ProjectManagerCursor::AddProject => ProjectManagerCursor::ViewProjects,
            ProjectManagerCursor::UpdateProject => ProjectManagerCursor::AddProject,
            ProjectManagerCursor::DeleteProject => ProjectManagerCursor::UpdateProject,
            ProjectManagerCursor::BackToMainMenu => ProjectManagerCursor::DeleteProject,
        }
    }
}

pub struct ProjectManager {
    cursor: ProjectManagerCursor,
}

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager {
            cursor: ProjectManagerCursor::ViewProjects,
        }
    }

    fn get_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            ProjectManagerCursor::ViewProjects,
            ProjectManagerCursor::AddProject,
            ProjectManagerCursor::UpdateProject,
            ProjectManagerCursor::DeleteProject,
            ProjectManagerCursor::BackToMainMenu,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    ProjectManagerCursor::ViewProjects => "View Projects",
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

    pub fn run(mut terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        let mut mgr = Self::new();
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
                                ProjectManagerCursor::ViewProjects => {
                                    // TODO: Implement view projects functionality
                                }
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

        list.render(area, buf);
    }
}