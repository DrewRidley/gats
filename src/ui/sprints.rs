use ratatui::{
    backend::Backend, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap}, Terminal
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};


#[derive(Clone, PartialEq, Eq)]
enum SprintManagerCursor {
    ViewSprints,
    AddSprint,
    UpdateSprint,
    DeleteSprint,
    BackToMainMenu,
}

impl SprintManagerCursor {
    fn next(&mut self) {
        *self = match *self {
            SprintManagerCursor::ViewSprints => SprintManagerCursor::AddSprint,
            SprintManagerCursor::AddSprint => SprintManagerCursor::UpdateSprint,
            SprintManagerCursor::UpdateSprint => SprintManagerCursor::DeleteSprint,
            SprintManagerCursor::DeleteSprint => SprintManagerCursor::BackToMainMenu,
            SprintManagerCursor::BackToMainMenu => SprintManagerCursor::ViewSprints,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            SprintManagerCursor::ViewSprints => SprintManagerCursor::BackToMainMenu,
            SprintManagerCursor::AddSprint => SprintManagerCursor::ViewSprints,
            SprintManagerCursor::UpdateSprint => SprintManagerCursor::AddSprint,
            SprintManagerCursor::DeleteSprint => SprintManagerCursor::UpdateSprint,
            SprintManagerCursor::BackToMainMenu => SprintManagerCursor::DeleteSprint,
        }
    }
}

pub struct SprintManager {
    cursor: SprintManagerCursor,
}

impl SprintManager {
    pub fn new() -> Self {
        SprintManager {
            cursor: SprintManagerCursor::ViewSprints,
        }
    }

    fn get_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            SprintManagerCursor::ViewSprints,
            SprintManagerCursor::AddSprint,
            SprintManagerCursor::UpdateSprint,
            SprintManagerCursor::DeleteSprint,
            SprintManagerCursor::BackToMainMenu,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    SprintManagerCursor::ViewSprints => "View Sprints",
                    SprintManagerCursor::AddSprint => "Add Sprint",
                    SprintManagerCursor::UpdateSprint => "Update Sprint",
                    SprintManagerCursor::DeleteSprint => "Delete Sprint",
                    SprintManagerCursor::BackToMainMenu => "Back to Main Menu",
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
                                SprintManagerCursor::ViewSprints => {
                                    // TODO: Implement view sprints functionality
                                }
                                SprintManagerCursor::AddSprint => {
                                    // TODO: Implement add sprint functionality
                                }
                                SprintManagerCursor::UpdateSprint => {
                                    // TODO: Implement update sprint functionality
                                }
                                SprintManagerCursor::DeleteSprint => {
                                    // TODO: Implement delete sprint functionality
                                }
                                SprintManagerCursor::BackToMainMenu => return Ok(()),
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

impl Widget for &mut SprintManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .title("Manage Sprints")
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