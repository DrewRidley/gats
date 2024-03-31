use ratatui::{
    backend::Backend, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, Paragraph, Widget, Wrap}, Terminal
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[derive(Clone, PartialEq, Eq)]
enum MemberManagerCursor {
    ViewMembers,
    AddMember,
    UpdateMember,
    DeleteMember,
    BackToMainMenu,
}

impl MemberManagerCursor {
    fn next(&mut self) {
        *self = match *self {
            MemberManagerCursor::ViewMembers => MemberManagerCursor::AddMember,
            MemberManagerCursor::AddMember => MemberManagerCursor::UpdateMember,
            MemberManagerCursor::UpdateMember => MemberManagerCursor::DeleteMember,
            MemberManagerCursor::DeleteMember => MemberManagerCursor::BackToMainMenu,
            MemberManagerCursor::BackToMainMenu => MemberManagerCursor::ViewMembers,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            MemberManagerCursor::ViewMembers => MemberManagerCursor::BackToMainMenu,
            MemberManagerCursor::AddMember => MemberManagerCursor::ViewMembers,
            MemberManagerCursor::UpdateMember => MemberManagerCursor::AddMember,
            MemberManagerCursor::DeleteMember => MemberManagerCursor::UpdateMember,
            MemberManagerCursor::BackToMainMenu => MemberManagerCursor::DeleteMember,
        }
    }
}

pub struct MemberManager {
    cursor: MemberManagerCursor,
}

impl MemberManager {
    pub fn new() -> Self {
        MemberManager {
            cursor: MemberManagerCursor::ViewMembers,
        }
    }

    fn get_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            MemberManagerCursor::ViewMembers,
            MemberManagerCursor::AddMember,
            MemberManagerCursor::UpdateMember,
            MemberManagerCursor::DeleteMember,
            MemberManagerCursor::BackToMainMenu,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    MemberManagerCursor::ViewMembers => "View Members",
                    MemberManagerCursor::AddMember => "Add Member",
                    MemberManagerCursor::UpdateMember => "Update Member",
                    MemberManagerCursor::DeleteMember => "Delete Member",
                    MemberManagerCursor::BackToMainMenu => "Back to Main Menu",
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
                                MemberManagerCursor::ViewMembers => {
                                    // TODO: Implement view members functionality
                                }
                                MemberManagerCursor::AddMember => {
                                    // TODO: Implement add member functionality
                                }
                                MemberManagerCursor::UpdateMember => {
                                    // TODO: Implement update member functionality
                                }
                                MemberManagerCursor::DeleteMember => {
                                    // TODO: Implement delete member functionality
                                }
                                MemberManagerCursor::BackToMainMenu => return Ok(()),
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

impl Widget for &mut MemberManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .title("Manage Members")
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