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
        terminal
            .draw(|f| {
                let project_name = &self.project.title;

                let action_items = vec![
                    ListItem::new(format!("Add Sprint to {}", project_name)),
                    ListItem::new(format!("Add Member to {}", project_name)),
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
                            .title(format!("Project Actions | {}", self.project.title)),
                    )
                    .highlight_symbol(">>");

                f.render_stateful_widget(action_list, chunks[0], &mut list_state);
            })
            .unwrap();

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
