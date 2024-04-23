use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, List, Widget},
    Terminal,
};
use sqlx::MySqlPool;

use crate::{
    crud::{delete_project_by_id, delete_sprint_by_id, delete_task_by_id, fetch_projects},
    Project,
};

// Import all dialogs.
use super::dialog::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord)]
enum ProjectCursorDepth {
    Project,
    Sprint,
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
                self.sprint = None;
                self.task = None;
            }
            ProjectCursorDepth::Sprint => {
                if self.project.is_some() {
                    self.sprint = self.sprint.map_or(Some(0), |s| Some(s + 1));
                    self.task = None;
                }
            }
            ProjectCursorDepth::Task => {
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
                self.sprint = None;
                self.task = None;
            }
            ProjectCursorDepth::Sprint => {
                if self.project.is_some() {
                    if let Some(s) = self.sprint {
                        if s > 0 {
                            self.sprint = Some(s - 1);
                        }
                    }
                }
                self.task = None;
            }
            ProjectCursorDepth::Task => {
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
        match self.depth {
            ProjectCursorDepth::Project => {
                self.sprint = Some(0);
                self.task = None;
                self.depth = ProjectCursorDepth::Sprint;
            }
            ProjectCursorDepth::Sprint => {
                self.task = Some(0);
                self.depth = ProjectCursorDepth::Task;
            }
            ProjectCursorDepth::Task => {}
        }
    }

    fn decrease_depth(&mut self) {
        match self.depth {
            ProjectCursorDepth::Project => {}
            ProjectCursorDepth::Sprint => {
                self.sprint = None;
                self.task = None;
                self.depth = ProjectCursorDepth::Project;
            }
            ProjectCursorDepth::Task => {
                self.task = None;
                self.depth = ProjectCursorDepth::Sprint;
            }
        }
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
    pool: MySqlPool,
}

#[derive(Default)]
enum RunResult {
    #[default]
    Continue,
    Return,
}

impl ProjectManager {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            cursor: Default::default(),
            projects: vec![],
            pool,
        }
    }

    async fn fetch_projects(&mut self) {
        self.projects = fetch_projects(&self.pool).await.unwrap();
    }

    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: MySqlPool,
    ) -> std::io::Result<()> {
        let mut mgr = Self::new(pool);
        mgr.fetch_projects().await;

        loop {
            mgr.draw(&mut terminal)?;

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match mgr.handle_key_press(terminal, key.code).await {
                        Ok(result) => match result {
                            RunResult::Continue => {}
                            RunResult::Return => {
                                return Ok(());
                            }
                        },
                        Err(e) => {
                            DisplayWindow::run(
                                terminal,
                                format!("An error occured. Changes were not saved: {}", e),
                            )
                            .await
                            .expect("Failed to show error screen.");
                        }
                    }
                }
                _ => {}
            }
        }
    }

    async fn edit_project(&self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        let current_proj = &self.projects[self.cursor.project.unwrap() as usize];
        let current_data = vec![current_proj.title.clone(), current_proj.desc.clone()];

        match CreateRecordDialog::new_edit(
            vec!["Title".into(), "Description".into()],
            current_data,
            |d: &CreateRecordDialog| true,
        )
        .run(terminal)
        .await?
        {
            CreateResults::Create(data) => {
                // Extract updated data
                let new_title = &data[0];
                let new_description = &data[1];

                // SQL statement to update the project
                let update_query =
                    "UPDATE Project SET Title = ?, Description = ? WHERE ProjectID = ?";
                let result = sqlx::query(update_query)
                    .bind(new_title)
                    .bind(new_description)
                    .bind(current_proj.proj_id)
                    .execute(&self.pool)
                    .await;

                match result {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        DisplayWindow::run(terminal, format!("Failed to update project: {}", e))
                            .await
                            .expect("Failed to show error screen.");
                        Ok(())
                    }
                }
            }
            CreateResults::Quit => {
                DisplayWindow::run(terminal, format!("Quit editor. No Changes Made..."))
                    .await
                    .expect("Failed to show error screen.");
                Ok(())
            }
        }
    }

    async fn edit_sprint(&self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        let current_proj = &self.projects[self.cursor.project.unwrap() as usize];
        let current_sprint = &current_proj.sprints[self.cursor.sprint.unwrap() as usize];
        let current_data = vec![
            current_sprint.title.clone(),
            current_sprint.start_date.to_string(),
            current_sprint.end_date.to_string(),
        ];

        match CreateRecordDialog::new_edit(
            vec!["Title".into(), "Start Date".into(), "End Date".into()],
            current_data,
            |d: &CreateRecordDialog| true,
        )
        .run(terminal)
        .await?
        {
            CreateResults::Create(data) => {
                let new_title = &data[0];
                let new_start_date = chrono::NaiveDate::parse_from_str(&data[1], "%Y-%m-%d").ok();
                let new_end_date = chrono::NaiveDate::parse_from_str(&data[2], "%Y-%m-%d").ok();

                if new_start_date.is_none() || new_end_date.is_none() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid date format",
                    ));
                }

                let update_query =
                    "UPDATE Sprint SET Title = ?, startDate = ?, endDate = ? WHERE SprintID = ?";
                sqlx::query(update_query)
                    .bind(new_title)
                    .bind(new_start_date.unwrap())
                    .bind(new_end_date.unwrap())
                    .bind(current_sprint.sprint_id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, "Failed to update sprint")
                    })?;

                Ok(())
            }
            CreateResults::Quit => Ok(()),
        }
    }

    async fn edit_task(&self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        // Access the current project and sprint using cursor indexes
        let current_proj = &self.projects[self.cursor.project.unwrap() as usize];
        let current_sprint = &current_proj.sprints[self.cursor.sprint.unwrap() as usize];
        // Access the specific task using the task cursor
        let current_task = &current_sprint.tasks[self.cursor.task.unwrap() as usize];
        // Prepare current data for editing
        let current_data = vec![
            current_task.title.clone(),
            current_task.status.clone(),
            current_task.description.clone(),
            current_task.estimated_hours.to_string(),
        ];

        // Create and run the dialog for editing task information
        match CreateRecordDialog::new_edit(
            vec![
                "Title".into(),
                "Status".into(),
                "Description".into(),
                "Estimated Hours".into(),
            ],
            current_data,
            |d: &CreateRecordDialog| true,
        )
        .run(terminal)
        .await?
        {
            CreateResults::Create(data) => {
                // Extract updated data from dialog
                let new_title = &data[0];
                let new_status = &data[1];
                let new_description = &data[2];
                let new_estimated_hours = data[3]
                    .parse::<i32>()
                    .unwrap_or(current_task.estimated_hours); // Use existing value as fallback

                // SQL statement to update the task
                let update_query = "UPDATE Task SET Title = ?, Status = ?, Description = ?, estimatedHours = ? WHERE TaskID = ?";
                let result = sqlx::query(update_query)
                    .bind(new_title)
                    .bind(new_status)
                    .bind(new_description)
                    .bind(new_estimated_hours)
                    .bind(current_task.task_id)
                    .execute(&self.pool)
                    .await;

                // Handle the result of the update operation
                match result {
                    Ok(_) => Ok(()),
                    Err(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to update task:",
                    )),
                }
            }
            CreateResults::Quit => Ok(()),
        }
    }

    async fn edit_entry(&mut self, term: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        match self.cursor.depth {
            ProjectCursorDepth::Project => self.edit_project(term).await?,
            ProjectCursorDepth::Sprint => self.edit_sprint(term).await?,
            ProjectCursorDepth::Task => self.edit_task(term).await?,
        }

        //Refresh after any possible updates.
        self.fetch_projects().await;
        Ok(())
    }

    async fn handle_key_press(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
        code: KeyCode,
    ) -> std::io::Result<RunResult> {
        match code {
            KeyCode::Right => self.cursor.increase_depth(),
            KeyCode::Left => self.cursor.decrease_depth(),
            KeyCode::Down => self.cursor.next(),
            KeyCode::Up => self.cursor.prev(),
            KeyCode::Enter => self.create_project_or_sprint(terminal).await?,
            KeyCode::Char('r') => return Ok(RunResult::Continue),
            KeyCode::Char('e') => self.edit_entry(terminal).await?,
            KeyCode::Char('c') => self.create_sprint_or_task(terminal).await?,
            KeyCode::Char('d') => self.delete_item(terminal).await,
            KeyCode::Char('m') => self.manage_members(terminal).await,
            KeyCode::Char('q') => return Ok(RunResult::Return),
            KeyCode::Esc => return Ok(RunResult::Return),
            _ => {}
        }
        Ok(RunResult::Continue)
    }

    async fn create_project_or_sprint(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
    ) -> std::io::Result<()> {
        if self.cursor.depth == ProjectCursorDepth::Project {
            if self.cursor.project == Some(self.projects.len() as u8) {
                CreateProjectDialog::run(terminal, &self.pool).await?;
                self.fetch_projects().await;

                return Ok(());
            }
        }

        Ok(())
    }

    async fn create_sprint_or_task(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
    ) -> std::io::Result<()> {
        match self.cursor.depth {
            ProjectCursorDepth::Project => {
                if let Some(project_idx) = self.cursor.project {
                    let proj_id = self.projects[project_idx as usize].proj_id;
                    if project_idx < self.projects.len() as u8 {
                        CreateSprintDialog::run(terminal, &self.pool, proj_id).await?;
                        self.fetch_projects().await;

                        return Ok(());
                    }
                }
            }
            ProjectCursorDepth::Sprint => {
                if let Some(project_idx) = self.cursor.project {
                    let proj_id = self.projects[project_idx as usize].proj_id;
                    if let Some(sprint_idx) = self.cursor.sprint {
                        let sprint_id = self.projects[project_idx as usize].sprints
                            [sprint_idx as usize]
                            .sprint_id;

                        match CreateRecordDialog::new(
                            vec![
                                String::from("Title"),
                                String::from("Status"),
                                String::from("Description"),
                                String::from("estimatedHours"),
                            ],
                            |diag: &CreateRecordDialog| true,
                        )
                        .run(terminal)
                        .await?
                        {
                            CreateResults::Create(data) => {
                                // Extract fields from the data vector
                                let title = &data[0];
                                let status = &data[1];
                                let description = &data[2];
                                let estimated_hours = data[3].parse::<i32>().unwrap_or(0); // Default to 0 if parsing fails

                                // SQL query to insert the new task
                                let insert_query = "INSERT INTO Task (Title, Status, Description, commitedHours, estimatedHours) VALUES (?, ?, ?, ?, ?)";
                                let task_row = sqlx::query(insert_query)
                                    .bind(title)
                                    .bind(status)
                                    .bind(description)
                                    .bind(0) // Setting commitedHours to 0 initially
                                    .bind(estimated_hours)
                                    .execute(&self.pool)
                                    .await;
                                match task_row {
                                    Ok(result) => {
                                        let last_insert_id = result.last_insert_id();
                                        let part_of_insert =
                                            "INSERT INTO PartOf (TaskID, SprintID) VALUES (?, ?)";
                                        sqlx::query(part_of_insert)
                                            .bind(last_insert_id)
                                            .bind(sprint_id)
                                            .execute(&self.pool)
                                            .await
                                            .expect("Failed to link task with sprint in the PartOf table!");

                                        self.fetch_projects().await;
                                    }
                                    Err(e) => {
                                        // Handle the error by displaying the custom error window
                                        DisplayWindow::run(
                                            terminal,
                                            format!(
                                                "An error occurred. Changes were not saved: {}",
                                                e
                                            ),
                                        )
                                        .await
                                        .expect("Failed to show error screen.");

                                        return Ok(());
                                    }
                                }
                            }
                            CreateResults::Quit => todo!(),
                        }

                        return Ok(());
                    }
                }
            }
            //You can't hit C on a task (theres nothing to create).
            ProjectCursorDepth::Task => {}
        }

        return Ok(());
    }

    async fn delete_item(&mut self, terminal: &mut Terminal<impl Backend>) {
        match self.cursor.depth {
            ProjectCursorDepth::Project => {
                if let Some(project_idx) = self.cursor.project {
                    if ConfirmDelete::run(terminal).await {
                        delete_project_by_id(
                            &self.pool,
                            self.projects[project_idx as usize].proj_id,
                        )
                        .await
                        .expect("Failed to delete project");
                        self.fetch_projects().await;
                    }
                }
            }
            ProjectCursorDepth::Sprint => {
                if let Some(project_idx) = self.cursor.project {
                    if ConfirmDelete::run(terminal).await {
                        delete_sprint_by_id(
                            &self.pool,
                            self.projects[project_idx as usize].sprints
                                [self.cursor.sprint.unwrap() as usize]
                                .sprint_id,
                        )
                        .await
                        .expect("Failed to delete sprint!");
                        self.fetch_projects().await;
                    }
                }
            }
            ProjectCursorDepth::Task => {
                if let Some(project_idx) = self.cursor.project {
                    if ConfirmDelete::run(terminal).await {
                        delete_task_by_id(
                            &self.pool,
                            self.projects[project_idx as usize].sprints
                                [self.cursor.sprint.unwrap() as usize]
                                .tasks[self.cursor.task.unwrap() as usize]
                                .task_id,
                        )
                        .await
                        .expect("Failed to delete sprint!");
                        self.fetch_projects().await;
                    }
                }
            }
        }
    }

    async fn manage_members(&mut self, terminal: &mut Terminal<impl Backend>) {
        if let Some(project_idx) = self.cursor.project {
            ProjectMembersDialog::run(
                terminal,
                &self.pool,
                self.projects[project_idx as usize].proj_id,
            )
            .await
            .expect("Error while managing project. Changes have not been saved.");
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

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

        let at_project_level = matches!(self.cursor.depth, ProjectCursorDepth::Project);
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
}

impl Widget for &mut ProjectManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut instruction_spans = Vec::new();
        let mut create_text = None;
        let mut show_instructions = true;

        match self.cursor.depth {
            ProjectCursorDepth::Project => {
                if self.cursor.project == Some(self.projects.len() as u8) {
                    show_instructions = false;
                } else if let Some(project_idx) = self.cursor.project {
                    if project_idx < self.projects.len() as u8 {
                        create_text = Some(format!(
                            " Create Sprint for '{}' ",
                            self.projects[project_idx as usize].title
                        ));
                    } else {
                        show_instructions = false;
                    }
                } else {
                    show_instructions = false;
                }
            }
            ProjectCursorDepth::Sprint => {
                if let Some(project_idx) = self.cursor.project {
                    if let Some(sprint_idx) = self.cursor.sprint {
                        if project_idx < self.projects.len() as u8
                            && sprint_idx < self.projects[project_idx as usize].sprints.len() as u8
                        {
                            let sprint =
                                &self.projects[project_idx as usize].sprints[sprint_idx as usize];
                            create_text = Some(format!(" Create Task for '{}' ", sprint.title));
                        } else {
                            show_instructions = false;
                        }
                    } else {
                        show_instructions = false;
                    }
                } else {
                    show_instructions = false;
                }
            }
            ProjectCursorDepth::Task => {
                show_instructions = true;
            }
        }

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
