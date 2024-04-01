use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Terminal,
};
use sqlx::MySqlPool;

use crate::{delete_project_by_id, Project};

#[derive(Debug)]
enum ProjectAction {
    AddSprint,
    AddMember,
    Modify,
    Delete,
}

pub struct ProjectDialog {
    //Where the cursor is on the current dialog.
    cursor: usize,
    action: Option<ProjectAction>,
    project: Project,
}

impl ProjectDialog {
    fn new(proj: Project) -> Self {
        Self {
            cursor: 0,
            action: None,
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
    
            let members_list = self.project.members.iter()
                .map(|member| format!("{} {}", member.first_name, member.last_name))
                .collect::<Vec<_>>()
                .join(" X, ") + " X";  // Add 'X' after each member for removal option
    
            let members_line = if !members_list.is_empty() {
                format!("Members: | {} | +", members_list)  // Add '+' at the end for adding a new member
            } else {
                "Members: | +".to_string()  // Only show '+' if there are no members
            };
    
            let action_items = vec![
                ListItem::new(format!("Add Sprint to {}", project_name)),
                ListItem::new(members_line),  // Modified to show members
                ListItem::new(format!("Modify {}", project_name)),
                ListItem::new(format!("DELETE {}", project_name)),
                ListItem::new("Return"),
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
                        .title(format!("Project Actions | {}", project_name)),
                )
                .highlight_symbol(" >>");
    
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
