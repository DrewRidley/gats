use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

pub struct ConfirmDelete {
    cursor: usize,
}

impl ConfirmDelete {
    pub async fn run(terminal: &mut Terminal<impl Backend>) -> bool {
        let mut diag = ConfirmDelete { cursor: 0 };

        loop {
            terminal
                .draw(|frame| {
                    let options = List::new(vec![
                        ListItem::new("Confirm delete"),
                        ListItem::new("Cancel"),
                    ]);

                    let chunks = Layout::default()
                        .constraints([Constraint::Percentage(100)].as_ref())
                        .split(frame.size());

                    let mut list_state = ListState::default();

                    let action_list = options
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Confirm DELETE")
                                .title_alignment(ratatui::layout::Alignment::Center),
                        )
                        .highlight_symbol(">")
                        .highlight_style(
                            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                        );
                    list_state.select(Some(diag.cursor));
                    frame.render_stateful_widget(action_list, chunks[0], &mut list_state);
                })
                .expect("Failed to render");

            if let Event::Key(key_event) = read().unwrap() {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Down => {
                            diag.cursor = (diag.cursor + 1) % 2;
                        }
                        KeyCode::Up => {
                            diag.cursor = if diag.cursor > 0 { diag.cursor - 1 } else { 1 };
                        }
                        KeyCode::Enter => {
                            let _ = match diag.cursor {
                                0 => {
                                    return true;
                                }
                                1 => {
                                    return false;
                                }
                                _ => {}
                            };
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
