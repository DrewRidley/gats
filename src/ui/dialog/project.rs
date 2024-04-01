

use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend, layout::{Alignment, Constraint, Layout}, text::Text, widgets::{Block, Borders, List, ListItem, ListState, Widget}, Terminal
};
use sqlx::MySqlPool;
use crate::{crud::delete_project_by_id, Project};

pub struct CreateProjectDialog {
    cursor: usize,
    name: String,
    desc: String,
    start_date: String,
    end_date: String,
}

impl CreateProjectDialog {
    fn new() -> Self {
        Self { cursor: 0, 
            name: String::new(), 
            desc: String::new(), 
            start_date: String::new(), 
            end_date: String::new() 
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
                            2 => diag.start_date.push(c),
                            3 => diag.end_date.push(c),
                            _ => {}
                        },
                        KeyCode::Enter => {
                            if diag.cursor == 4 {
                                //Actually create the project and return.

                                return Ok(());
                            }
                        }
                        KeyCode::Backspace => match diag.cursor {
                            0 => { diag.name.pop(); }
                            1 => { diag.desc.pop(); }
                            2 => { diag.start_date.pop(); }
                            3 => { diag.end_date.pop(); }
                            _ => {}
                        },
                        KeyCode::Down => {
                            diag.cursor = (diag.cursor + 1) % 5;
                        }
                        KeyCode::Up => {
                            diag.cursor = if diag.cursor > 0 {
                                diag.cursor - 1
                            } else {
                                3
                            };
                        }
                        KeyCode::Esc => {
                            return Ok(())
                        }
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
                ListItem::new(format!("Start Date: {}", self.start_date)),
                ListItem::new(format!("End Date: {}", self.end_date)),
                ListItem::new("Create Project")
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
                        .title("Create new Project").title_alignment(ratatui::layout::Alignment::Center),
                )
                .highlight_symbol(">")
                .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
    
            frame.render_stateful_widget(action_list, chunks[0], &mut list_state);
        })?;

        Ok(())
    }
}


pub struct ProjectDialog {
    //Where the cursor is on the current dialog.
    cursor: usize,
    project: Project,
}

impl ProjectDialog {
    fn new(proj: Project) -> Self {
        Self {
            cursor: 0,
            project: proj,
        }
    }

    pub async fn run(
        proj: Project,
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
    ) -> std::io::Result<()> {
        let mut diag = ProjectDialog::new(proj.clone());

        loop {
            diag.draw(&mut terminal)?;

            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Down => {
                            diag.cursor += 1;
                        }
                        KeyCode::Up => {
                            if diag.cursor > 0 {
                                diag.cursor -= 1;
                            } else {
                                diag.cursor = 3;
                            }
                        }
                        KeyCode::Enter => {
                            match diag.cursor {
                                //Add sprint
                                0 => {}
                                //Add member
                                1 => {
                                    todo!()
                                }
                                //Modify certain fields of the project.
                                2 => {}
                                //Delete the project here.
                                3 => {
                                    delete_project_by_id(pool, proj.proj_id)
                                        .await
                                        .expect("Received an error while deleting project!");
                                    return Ok(());
                                }
                                //Return to previous menu.
                                4 => return Ok(()),
                                _ => {}
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| {
            let project_name = &self.project.title;
    
            
    
            let action_items = vec![
                ListItem::new(Text::from(format!("✎ Modify {}", project_name)).alignment(Alignment::Left)),
                ListItem::new(Text::from(format!("+ Create Sprint")).alignment(Alignment::Left)),
                ListItem::new(Text::from("⚙ Manage members").alignment(Alignment::Left)),  // Modified to show members
                ListItem::new(Text::from(format!("🗑 Delete {} ", project_name)).alignment(Alignment::Left)),
                ListItem::new(Text::from("Return ⏎").alignment(Alignment::Left)),
            ];
    
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());
    
            let mut list_state = ListState::default();
            list_state.select(Some(self.cursor));
    
            let action_list = List::new(action_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Manage Project: {}", project_name)).title_alignment(ratatui::layout::Alignment::Center),
                )
                .highlight_symbol(">")
                .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
    
            f.render_stateful_widget(action_list, chunks[0], &mut list_state);
        })?;
    
        Ok(())
    }
}

impl Widget for ProjectDialog {
    fn render(self, _area: ratatui::prelude::Rect, _buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
    }
}
