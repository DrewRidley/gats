use std::default;

use log::trace;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
    Terminal,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use sqlx::MySqlPool;

use crate::{fetch_projects, Project};

use super::project_dialog::ProjectDialog;

#[derive(PartialEq, Eq, Copy, Clone)]
enum ManagementCursor {
    AddProject,
    DeleteProject,
    MainMenu,
}

impl ManagementCursor {
    fn next(&mut self) {
        *self = match *self {
            ManagementCursor::AddProject => ManagementCursor::DeleteProject,
            ManagementCursor::DeleteProject => ManagementCursor::MainMenu,
            ManagementCursor::MainMenu => ManagementCursor::AddProject,
        };
    }

    fn prev(&mut self) {
        *self = match *self {
            ManagementCursor::AddProject => ManagementCursor::MainMenu,
            ManagementCursor::DeleteProject => ManagementCursor::AddProject,
            ManagementCursor::MainMenu => ManagementCursor::DeleteProject,
        };
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
enum ProjectCursorDepth {
    //The user is navigating between projects with Up/Down,
    Project,
    // The user is navigating between sprints for this particular project with Up/Down.
    Sprint,
    // The user is navigating between tasks for the current sprint with Up/Down.
    Task,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct ProjectCursor {
    depth: ProjectCursorDepth,
    project: Option<u8>,
    sprint: Option<u8>,
    task: Option<u8>,
}

impl ProjectCursor {
    fn next(&mut self) {
        match self.depth {
            ProjectCursorDepth::Project => {
                self.project = self.project.map_or(Some(0), |p| Some(p + 1));
                // Reset the lower levels when changing the project
                self.sprint = None;
                self.task = None;
            }
            ProjectCursorDepth::Sprint => {
                // Only advance sprint if project is selected
                if self.project.is_some() {
                    self.sprint = self.sprint.map_or(Some(0), |s| Some(s + 1));
                    // Reset task level when changing the sprint
                    self.task = None;
                }
            }
            ProjectCursorDepth::Task => {
                // Only advance task if project and sprint are selected
                if self.project.is_some() && self.sprint.is_some() {
                    self.task = self.task.map_or(Some(0), |t| Some(t + 1));
                }
            }
        }
    }

    fn prev(&mut self) {
        match self.depth {
            ProjectCursorDepth::Project => {
                if let Some(p) = self.project {
                    if p > 0 {
                        self.project = Some(p - 1);
                    }
                }
                // Reset the lower levels when changing the project
                self.sprint = None;
                self.task = None;
            }
            ProjectCursorDepth::Sprint => {
                // Only decrement sprint if project is selected
                if self.project.is_some() {
                    if let Some(s) = self.sprint {
                        if s > 0 {
                            self.sprint = Some(s - 1);
                        }
                    }
                }
                // Reset task level when changing the sprint
                self.task = None;
            }
            ProjectCursorDepth::Task => {
                // Only decrement task if project and sprint are selected
                if self.project.is_some() && self.sprint.is_some() {
                    if let Some(t) = self.task {
                        if t > 0 {
                            self.task = Some(t - 1);
                        }
                    }
                }
            }
        }
    }

    fn increase_depth(&mut self) {
        self.depth = match self.depth {
            ProjectCursorDepth::Project => {
                self.sprint = Some(0); // Set sprint to 0 when increasing depth from Project
                self.task = None; // Reset task when leaving Project depth
                ProjectCursorDepth::Sprint
            }
            ProjectCursorDepth::Sprint => {
                self.task = Some(0); // Set task to 0 when increasing depth from Sprint
                ProjectCursorDepth::Task
            }
            ProjectCursorDepth::Task => ProjectCursorDepth::Task, // Max depth, stays at Task
        };
    }

    fn decrease_depth(&mut self) {
        self.depth = match self.depth {
            ProjectCursorDepth::Project => ProjectCursorDepth::Project, // Min depth, stays at Project
            ProjectCursorDepth::Sprint => {
                self.sprint = None; // Reset sprint when decreasing depth to Project
                self.task = None; // Reset task when decreasing depth to Project
                ProjectCursorDepth::Project
            }
            ProjectCursorDepth::Task => {
                self.task = None; // Reset task when decreasing depth to Sprint
                ProjectCursorDepth::Sprint
            }
        };
    }
}

impl Default for ProjectCursor {
    fn default() -> Self {
        Self {
            depth: ProjectCursorDepth::Project,
            project: Some(0),
            sprint: None,
            task: None,
        }
    }
}

#[derive(PartialEq, Eq)]
enum Cursor {
    Project(ProjectCursor),
    Manage(ManagementCursor),
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::Manage(ManagementCursor::AddProject)
    }
}

pub struct ProjectManager {
    cursor: Cursor,
    projects: Vec<Project>,
}

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager {
            cursor: Default::default(),
            projects: vec![],
        }
    }

    //Renders all of the lines in the project menu.
    fn project_lines(&self) -> Vec<Span> {
        let mut lines = Vec::new();
        let selected_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let pc = match &self.cursor {
            Cursor::Project(pc) => pc,
            Cursor::Manage(_) => &ProjectCursor {
                depth: ProjectCursorDepth::Project,
                project: None,
                sprint: None,
                task: None,
            },
        };

        for (project_index, project) in self.projects.iter().enumerate() {
            let project_is_selected = pc.project == Some(project_index as u8);

            let project_span = if project_is_selected {
                Span::styled(
                    format!("◆ Project #{}: {}", project.ProjectID, project.Title),
                    selected_style,
                )
            } else {
                Span::raw(format!(
                    "  Project #{}: {}",
                    project.ProjectID, project.Title
                ))
            };
            lines.push(project_span);

            if project_is_selected
                && (pc.depth == ProjectCursorDepth::Sprint || pc.depth == ProjectCursorDepth::Task)
            {
                for (sprint_index, sprint) in project.Sprints.iter().enumerate() {
                    let sprint_is_selected = pc.sprint == Some(sprint_index as u8);

                    let sprint_span = if sprint_is_selected {
                        Span::styled(
                            format!(
                                "  ◆ Sprint #{}: {} ({} to {})",
                                sprint.SprintID, sprint.Title, sprint.startDate, sprint.endDate
                            ),
                            selected_style,
                        )
                    } else {
                        Span::raw(format!(
                            "    Sprint #{}: {} ({} to {})",
                            sprint.SprintID, sprint.Title, sprint.startDate, sprint.endDate
                        ))
                    };
                    lines.push(sprint_span);

                    if pc.depth == ProjectCursorDepth::Task && sprint_is_selected {
                        for (task_index, task) in sprint.Tasks.iter().enumerate() {
                            let task_is_selected = pc.task == Some(task_index as u8);
                            let emoji = match task.Status.as_str() {
                                "NotStarted" => "⏳",
                                "InProgress" => "🚧",
                                "Completed" => "✅",
                                _ => "❓",
                            };
                            let task_span = if task_is_selected {
                                Span::styled(
                                    format!(
                                        "    ◆ Task #{}: {} - {} {} | {}h estimated, {}h completed",
                                        task.TaskID,
                                        task.Title,
                                        task.Status,
                                        emoji,
                                        task.estimatedHours,
                                        task.commitedHours
                                    ),
                                    selected_style,
                                )
                            } else {
                                Span::raw(format!(
                                    "      Task #{}: {} - {} {} | {}h estimated, {}h completed",
                                    task.TaskID,
                                    task.Title,
                                    task.Status,
                                    emoji,
                                    task.estimatedHours,
                                    task.commitedHours
                                ))
                            };
                            lines.push(task_span);
                        }
                    }
                }
            }
        }

        lines
    }

    //Renders all of the lines in the 'management' menu.
    fn management_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            ManagementCursor::AddProject,
            ManagementCursor::DeleteProject,
            ManagementCursor::MainMenu,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    ManagementCursor::AddProject => "Add Project",
                    ManagementCursor::DeleteProject => "Delete Project",
                    ManagementCursor::MainMenu => "Back to Main Menu",
                };

                let is_selected = match &self.cursor {
                    Cursor::Manage(manage_cursor) => manage_cursor == item,
                    Cursor::Project(_) => false, // ProjectCursor doesn't match ManagementCursor items
                };

                if is_selected {
                    Span::styled(name, highlight_style)
                } else {
                    Span::raw(name)
                }
            })
            .collect()
    }

    //The main event loop responsible for processing events and rendering every frame.
    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
    ) -> std::io::Result<()> {
        let mut mgr = Self::new();
        mgr.projects = fetch_projects(pool).await.unwrap();

        loop {
            mgr.draw(&mut terminal)?;

            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match &mut mgr.cursor {
                            Cursor::Project(proj_cursor) => {
                                match key.code {
                                    KeyCode::Right => {
                                        proj_cursor.increase_depth();
                                        trace!(
                                            "Increased depth of cursor. New cursor: {:?}",
                                            proj_cursor
                                        );
                                    }
                                    KeyCode::Left => {
                                        if let ProjectCursorDepth::Project = proj_cursor.depth {
                                            mgr.cursor =
                                                Cursor::Manage(ManagementCursor::AddProject);
                                        } else {
                                            proj_cursor.decrease_depth();
                                        }
                                    }
                                    KeyCode::Down => {
                                        proj_cursor.next(); // Moving to the next project, sprint, or task in the list
                                    }
                                    KeyCode::Up => {
                                        proj_cursor.prev(); // Moving to the previous project, sprint, or task in the list
                                    }
                                    KeyCode::Enter => {
                                        match proj_cursor.depth {
                                            ProjectCursorDepth::Project => {
                                                trace!("Attempted to open a project's dialog!");
                                                ProjectDialog::run(
                                                    mgr.projects
                                                        [proj_cursor.project.unwrap() as usize]
                                                        .clone(),
                                                    terminal,
                                                    pool,
                                                )
                                                .await?;
                                                //Refresh the state after the dialog since its likely now stale.
                                                mgr.projects = fetch_projects(pool).await.unwrap();
                                            }
                                            ProjectCursorDepth::Sprint => todo!(),
                                            ProjectCursorDepth::Task => todo!(),
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Cursor::Manage(manage_cursor) => {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                    KeyCode::Down => manage_cursor.next(),
                                    KeyCode::Up => manage_cursor.prev(),
                                    KeyCode::Enter => {
                                        match manage_cursor {
                                            ManagementCursor::AddProject => {
                                                // TODO: Implement add project functionality
                                            }
                                            ManagementCursor::DeleteProject => {
                                                // TODO: Implement delete project functionality
                                            }
                                            ManagementCursor::MainMenu => return Ok(()),
                                        }
                                    }
                                    KeyCode::Right => {
                                        mgr.cursor = Cursor::Project(ProjectCursor::default());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    //Responsible for drawing the page using existing widget logic.
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
        //The left and right side of the screen as 'chunks'.
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10), // 30% of the screen for the menu
                Constraint::Percentage(90), // 70% of the screen for project details
            ])
            .split(area);

        let management_block = Block::default()
            .title("Manage Projects")
            .borders(Borders::ALL);

        let management_lines: Vec<Line> = self
            .management_menu_lines()
            .into_iter()
            .map(|span| Line::from(span))
            .collect();
        let management_list = List::new(management_lines)
            .block(management_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        management_list.render(chunks[0], buf);

        let proj_block = Block::default()
            .title("Projects List")
            .borders(Borders::ALL);

        let proj_lines: Vec<Line> = self
            .project_lines()
            .into_iter()
            .map(|span| Line::from(span))
            .collect();
        let proj_list = List::new(proj_lines).block(proj_block).highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        proj_list.render(chunks[1], buf);
    }
}
