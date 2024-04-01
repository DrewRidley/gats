use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend, layout::{Alignment, Constraint, Layout}, text::Text, widgets::{Block, Borders, List, ListItem, ListState, Widget}, Terminal
};
use sqlx::MySqlPool;
use crate::Sprint;
use crate::crud::delete_sprint_by_id;

pub struct SprintDialog {
    //Where the cursor is on the current dialog.
    cursor: usize,
    sprint: Sprint,
}

impl SprintDialog {
    fn new(sprint: Sprint) -> Self {
        Self {
            cursor: 0,
            sprint
        }
    }

    pub async fn run(
        sprint: Sprint,
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
    ) -> std::io::Result<()> {
        let mut diag = SprintDialog::new(sprint.clone());

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
                                //Add task
                                0 => {}
                                //Delete the sprint here.
                                1 => {
                                    delete_sprint_by_id(pool, sprint.sprint_id)
                                        .await
                                        .expect("Received an error while deleting sprint!");
                                    return Ok(());
                                }
                                //Return to previous menu.
                                2 => return Ok(()),
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
            let project_name = &self.sprint.title;
    
            
    
            let action_items = vec![
                ListItem::new(Text::from(format!("+ Create Task")).alignment(Alignment::Left)),
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
                        .title(format!("Manage Sprint: {}", project_name)).title_alignment(ratatui::layout::Alignment::Center),
                )
                .highlight_symbol(">")
                .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
    
            f.render_stateful_widget(action_list, chunks[0], &mut list_state);
        })?;
    
        Ok(())
    }
}
