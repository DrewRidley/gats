use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};
use sqlx::MySqlPool;

use crate::ui::{members, projects};

/// An enum describing the possible cursor positions in the main menu.
#[derive(Clone, PartialEq, Eq)]
enum MainMenuCursor {
    ManageProjects,
    ManageMembers,
    Exit,
}

impl MainMenuCursor {
    /// Advances the main menu cursor to the next element.
    fn next(&mut self) {
        *self = match *self {
            MainMenuCursor::ManageProjects => MainMenuCursor::ManageMembers,
            MainMenuCursor::ManageMembers => MainMenuCursor::Exit,
            MainMenuCursor::Exit => MainMenuCursor::ManageProjects,
        }
    }

    /// Moves the cursor to the previous element on the menu.
    fn prev(&mut self) {
        *self = match *self {
            MainMenuCursor::ManageProjects => MainMenuCursor::Exit,
            MainMenuCursor::ManageMembers => MainMenuCursor::ManageProjects,
            MainMenuCursor::Exit => MainMenuCursor::ManageMembers,
        }
    }
}

/// The app and its encompassed state.
/// Since this application uses recursion as the basis to handle control flow,
/// The main app only has the cursor for its state.
pub struct App {
    cursor: MainMenuCursor,
}

impl App {
    /// Create a new app.
    pub fn new() -> Self {
        App {
            cursor: MainMenuCursor::ManageProjects,
        }
    }

    /// Returns a rendering of all of the lines for the main menu, with the selected one highlighted.
    fn get_main_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            MainMenuCursor::ManageProjects,
            MainMenuCursor::ManageMembers,
            MainMenuCursor::Exit,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    MainMenuCursor::ManageProjects => "Manage Projects",
                    MainMenuCursor::ManageMembers => "Manage Members",
                    MainMenuCursor::Exit => "Exit",
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

    /// Runs this menu until it terminates.
    pub async fn run(
        &mut self,
        mut terminal: Terminal<impl Backend>,
        pool: &MySqlPool,
    ) -> std::io::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            let cur = self.cursor.clone();

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Down => self.cursor.next(),
                        Up => self.cursor.prev(),
                        Enter => {
                            let _ = match cur {
                                MainMenuCursor::ManageProjects => {
                                    projects::ProjectManager::run(&mut terminal, pool.clone())
                                        .await?
                                }
                                MainMenuCursor::ManageMembers => {
                                    let mut mgr = members::MemberManager::new(&pool.clone()).await;
                                    mgr.run(&mut terminal, &pool.clone()).await?;
                                }
                                MainMenuCursor::Exit => {
                                    return Ok(());
                                }
                            };
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Draws this menu to the terminal.
    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    //Renders the list of selections available on this menu.
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let text_lines: Vec<Line> = self
            .get_main_menu_lines()
            .into_iter()
            .map(|span| Line::from(span))
            .collect();

        let paragraph = Paragraph::new(text_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}

/// Renders the title of the application to the main menu.
fn render_title(area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!("TATs v{}", env!("CARGO_PKG_VERSION")))
        .bold()
        .centered()
        .render(area, buf);
}

/// Renders the footer of the application to the main menu.
fn render_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!(
        "TATs 2024© All Rights Reserved. Developed exclusively by Drew Ridley."
    ))
    .bold()
    .centered()
    .render(area, buf);
}

/// Implement the main menu as a widget for ratatui.
impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // Create a space for header, todo list and the footer.
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ]);

        let [header_area, rest_area, footer_area] = vertical.areas(area);
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [_upper_item_list_area, _lower_item_list_area] = vertical.areas(rest_area);

        render_title(header_area, buf);
        render_footer(footer_area, buf);

        self.render_list(rest_area, buf);
    }
}
