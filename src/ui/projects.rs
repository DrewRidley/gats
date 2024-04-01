use log::trace;
use ratatui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, List, Widget},
    Terminal,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use sqlx::MySqlPool;

use crate::{crud::{delete_project_by_id, fetch_projects}, Project};

//Import all dialogs.
use super::dialog::prelude::*;

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

pub struct ProjectManager {
    cursor: ProjectCursor,
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

        let pc = &self.cursor;

        for (project_index, project) in self.projects.iter().enumerate() {
            let project_is_selected = pc.project == Some(project_index as u8);

            let project_span = if project_is_selected {
                Span::styled(
                    format!("◆ Project #{}: {}", project.proj_id, project.title),
                    selected_style,
                )
            } else {
                Span::raw(format!("  Project #{}: {}", project.proj_id, project.title))
            };
            lines.push(project_span);

            if project_is_selected
                && (pc.depth == ProjectCursorDepth::Sprint || pc.depth == ProjectCursorDepth::Task)
            {
                for (sprint_index, sprint) in project.sprints.iter().enumerate() {
                    let sprint_is_selected = pc.sprint == Some(sprint_index as u8);

                    let sprint_span = if sprint_is_selected {
                        Span::styled(
                            format!(
                                "  ◆ Sprint #{}: {} ({} to {})",
                                sprint.sprint_id, sprint.title, sprint.start_date, sprint.end_date
                            ),
                            selected_style,
                        )
                    } else {
                        Span::raw(format!(
                            "    Sprint #{}: {} ({} to {})",
                            sprint.sprint_id, sprint.title, sprint.start_date, sprint.end_date
                        ))
                    };
                    lines.push(sprint_span);

                    if pc.depth == ProjectCursorDepth::Task && sprint_is_selected {
                        for (task_index, task) in sprint.tasks.iter().enumerate() {
                            let task_is_selected = pc.task == Some(task_index as u8);
                            let emoji = match task.status.as_str() {
                                "NotStarted" => "⏳",
                                "InProgress" => "🚧",
                                "Completed" => "✅",
                                _ => "❓",
                            };
                            let task_span = if task_is_selected {
                                Span::styled(
                                    format!(
                                        "    ◆ Task #{}: {} - {} {} | {}h estimated, {}h completed",
                                        task.task_id,
                                        task.title,
                                        task.status,
                                        emoji,
                                        task.estimated_hours,
                                        task.commited_hours
                                    ),
                                    selected_style,
                                )
                            } else {
                                Span::raw(format!(
                                    "      Task #{}: {} - {} {} | {}h estimated, {}h completed",
                                    task.task_id,
                                    task.title,
                                    task.status,
                                    emoji,
                                    task.estimated_hours,
                                    task.commited_hours
                                ))
                            };
                            lines.push(task_span);
                        }
                    }
                }
            }
        }

        // Check if we're at the Project level without sub-levels like Sprint or Task
        let at_project_level = matches!(self.cursor.depth, ProjectCursorDepth::Project);

        // After listing all the projects, append the "Create New Project +" entry at the bottom
        if at_project_level {
            lines.push(Span::styled(
                " Create New Project +",
                Style::default().fg(Color::Green).add_modifier(
                    if pc.project == Some(lines.len() as u8) {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ),
            ));
        }

        lines
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
                        let proj_cursor = &mut mgr.cursor;

                        match key.code {
                            KeyCode::Right => {
                                proj_cursor.increase_depth();
                                trace!("Increased depth of cursor. New cursor: {:?}", proj_cursor);
                            }
                            KeyCode::Left => {
                                proj_cursor.decrease_depth();
                            }
                            KeyCode::Down => {
                                proj_cursor.next(); // Moving to the next project, sprint, or task in the list
                            }
                            KeyCode::Up => {
                                proj_cursor.prev(); // Moving to the previous project, sprint, or task in the list
                            }
                            KeyCode::Enter => {
                                //Create new project button.
                                if proj_cursor.depth == ProjectCursorDepth::Project {
                                    //If the cursor is on the last project, create a new project.
                                    if proj_cursor.project == Some(mgr.projects.len() as u8) {
                                        CreateProjectDialog::run(&mut terminal, pool).await?;
                                        mgr.projects = fetch_projects(pool).await.unwrap();
                                    }
                                }
                            }
                            KeyCode::Char('r') => {
                                return Ok(());
                            }
                            KeyCode::Char('c') => {
                                // Create a new sprint
                                if proj_cursor.depth == ProjectCursorDepth::Project {
                                    if let Some(project_idx) = proj_cursor.project {
                                        let proj_id = mgr.projects[project_idx as usize].proj_id;

                                        if project_idx < mgr.projects.len() as u8 {
                                            CreateSprintDialog::run(&mut terminal, pool, proj_id)
                                                .await?;
                                            mgr.projects = fetch_projects(pool).await.unwrap();
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('d') => {
                                match proj_cursor.depth {
                                    ProjectCursorDepth::Project => {
                                        if let Some(project_idx) = proj_cursor.project {
                                            if ConfirmDelete::run(&mut terminal).await {
                                                delete_project_by_id(pool, mgr.projects[project_idx as usize].proj_id)
                                                    .await
                                                    .expect("Failed to delete project");
                                                mgr.projects = fetch_projects(pool).await.unwrap();
                                            }   
                                        }
                                    },
                                    ProjectCursorDepth::Sprint => {
                                        if let Some(project_idx) = proj_cursor.project {
                                            if let Some(sprint_idx) = proj_cursor.sprint {
                                                let sprint_id = mgr.projects[project_idx as usize].sprints[sprint_idx as usize].sprint_id;
                                                if ConfirmDelete::run(&mut terminal).await {
                                                    // Begin a transaction
                                                    let mut tx = pool.begin().await.expect("Failed to begin transaction");
                                    
                                    
                                                    sqlx::query("DELETE FROM PartOf WHERE SprintID = ?")
                                                        .bind(sprint_id)
                                                        .execute(&mut *tx)
                                                        .await
                                                        .expect("Failed to delete PartOf records associated with the sprint");
                                
                                                    //Remove association between sprint and project.                             
                                                    sqlx::query("DELETE FROM ProjectSprint WHERE SprintID = ?")
                                                    .bind(sprint_id)
                                                    .execute(&mut *tx)
                                                    .await
                                                    .expect("Failed to delete sprint");

                                                    sqlx::query("DELETE FROM Sprint WHERE SprintID = ?")
                                                        .bind(sprint_id)
                                                        .execute(&mut *tx)
                                                        .await
                                                        .expect("Failed to delete sprint");
                                    
                                                    // Commit the transaction
                                                    tx.commit().await.expect("Failed to commit transaction");
                                    
                                                    // Refetch projects to update the manager's state
                                                    mgr.projects = fetch_projects(pool).await.unwrap();
                                                }
                                            }
                                        }
                                    },
                                    ProjectCursorDepth::Task => {}
                                }
                            }
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            _ => {}
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
        //Rendering the tooltip hints.
        let (create_text, show_instructions) = match self.cursor.depth {
            //If we are at project depth, tooltip must show 'create sprint'.
            ProjectCursorDepth::Project => {
                if self.cursor.project == Some(self.projects.len() as u8) {
                    // Cursor is on "Create New Project", don't show create text or instructions
                    (None, false)
                } else if let Some(project_idx) = self.cursor.project {
                    if project_idx < self.projects.len() as u8 {
                        // Cursor is within bounds, show "Create Sprint for 'Project'" text and instructions
                        (
                            Some(format!(
                                " Create Sprint for '{}' ",
                                self.projects[project_idx as usize].title
                            )),
                            true,
                        )
                    } else {
                        // Cursor is out of bounds, don't show create text or instructions
                        (None, false)
                    }
                } else {
                    // Cursor is out of bounds or unspecified, don't show create text or instructions
                    (None, false)
                }
            }
            //If we are at sprint depth, the tooltip must show 'create task'.
            ProjectCursorDepth::Sprint => {
                if let Some(project_idx) = self.cursor.project {
                    if let Some(sprint_idx) = self.cursor.sprint {
                        if project_idx < self.projects.len() as u8
                            && sprint_idx < self.projects[project_idx as usize].sprints.len() as u8
                        {
                            // Cursor is within bounds, show "Create Task for 'Sprint'" text and instructions
                            let sprint =
                                &self.projects[project_idx as usize].sprints[sprint_idx as usize];
                            (Some(format!(" Create Task for '{}' ", sprint.title)), true)
                        } else {
                            // Cursor is out of bounds, don't show create text or instructions
                            (None, false)
                        }
                    } else {
                        // Sprint index is unspecified, don't show create text or instructions
                        (None, false)
                    }
                } else {
                    // Project index is unspecified, don't show create text or instructions
                    (None, false)
                }
            }
            //At task depth, there is no create tooltip (you cannot create anything when highlighting a task.)
            ProjectCursorDepth::Task => {
                // We are at the task depth; no creation text should be shown, but other instructions can be
                (None, true)
            }
        };

        //A list of all the instructions.
        let mut instruction_spans = Vec::new();

        if show_instructions {
            if let Some(text) = create_text {
                instruction_spans.push(Span::raw(text));
                instruction_spans.push(Span::styled(
                    "<C> ",
                    Style::default().fg(Color::Rgb(255, 165, 0)),
                ));
            }

            instruction_spans.extend(vec![
                Span::raw("Edit "),
                Span::styled("<E> ", Style::default().fg(Color::Rgb(255, 165, 0))),
                Span::raw("Delete "),
                Span::styled("<D> ", Style::default().fg(Color::Rgb(255, 165, 0))),
            ]);

            // Only include 'Manage Members' if the depth is not Sprint
            if self.cursor.depth != ProjectCursorDepth::Sprint {
                instruction_spans.push(Span::raw("Manage Members "));
                instruction_spans.push(Span::styled(
                    "<M> ",
                    Style::default().fg(Color::Rgb(255, 165, 0)),
                ));
            }

            instruction_spans.extend(vec![
                Span::raw("Return "),
                Span::styled("<R> ", Style::default().fg(Color::Rgb(255, 165, 0))),
            ]);
        }

        let instructions = Title::from(Line::from(instruction_spans));

        let proj_block = Block::default()
            .title("Projects")
            .title(
                instructions
                    .alignment(ratatui::layout::Alignment::Center)
                    .position(ratatui::widgets::block::Position::Bottom),
            )
            .borders(Borders::ALL)
            .title_alignment(ratatui::layout::Alignment::Center);

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

        proj_list.render(area, buf);
    }
}
