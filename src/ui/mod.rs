use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap};

use crate::ui::projects::ProjectManager;




mod projects;
mod sprints;
mod members;
mod tasks;

#[derive(Clone, PartialEq, Eq)]
enum MainMenuCursor {
    ManageProjects,
    ManageSprints,
    ManageTasks,
    ManageMembers,
    Exit
}

impl MainMenuCursor {
    fn next(&mut self) {
        *self = match *self {
            MainMenuCursor::ManageProjects => MainMenuCursor::ManageSprints,
            MainMenuCursor::ManageSprints => MainMenuCursor::ManageTasks,
            MainMenuCursor::ManageTasks => MainMenuCursor::ManageMembers,
            MainMenuCursor::ManageMembers => MainMenuCursor::Exit,
            MainMenuCursor::Exit => MainMenuCursor::ManageProjects,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            MainMenuCursor::ManageProjects => MainMenuCursor::Exit,
            MainMenuCursor::ManageSprints => MainMenuCursor::ManageProjects,
            MainMenuCursor::ManageTasks => MainMenuCursor::ManageSprints,
            MainMenuCursor::ManageMembers => MainMenuCursor::ManageTasks,
            MainMenuCursor::Exit => MainMenuCursor::ManageMembers,
        }
    }
}


pub struct App {
    cursor: MainMenuCursor
}


impl App {
    pub fn new() -> Self {
        App {
            //Start the cursor at manage members.
            cursor: MainMenuCursor::ManageMembers
        }
    }

    fn get_main_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);
    
        let menu_items = vec![
            MainMenuCursor::ManageProjects,
            MainMenuCursor::ManageSprints,
            MainMenuCursor::ManageTasks,
            MainMenuCursor::ManageMembers,
            MainMenuCursor::Exit,
        ];
    
        menu_items.iter().map(|item| {
            let name = match item {
                MainMenuCursor::ManageProjects => "Manage Projects",
                MainMenuCursor::ManageSprints => "Manage Sprints",
                MainMenuCursor::ManageTasks => "Manage Tasks",
                MainMenuCursor::ManageMembers => "Manage Members",
                MainMenuCursor::Exit => "Exit",
            };
    
            let is_selected = self.cursor == *item;
            if is_selected {
                Span::styled(name, highlight_style)
            } else {
                Span::raw(name)
            }
        }).collect()
    }

    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> std::io::Result<()> {
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
                                MainMenuCursor::ManageProjects => projects::ProjectManager::run(&mut terminal),
                                MainMenuCursor::ManageSprints => sprints::SprintManager::run(&mut terminal),
                                MainMenuCursor::ManageTasks => tasks::TaskManager::run(&mut terminal),
                                MainMenuCursor::ManageMembers => members::MemberManager::run(&mut terminal),
                                MainMenuCursor::Exit => {
                                    return Ok(());
                                },
                            };
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

    //Renders the list of selections available on this menu.
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {


        let text_lines: Vec<Line> = self.get_main_menu_lines().into_iter().map(|span| Line::from(span)).collect();

        let paragraph = Paragraph::new(text_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}


fn render_title(area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!("GATs v{}", env!("CARGO_PKG_VERSION")))
        .bold()
        .centered()
        .render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new(format!("GATs 2024© All Rights Reserved. Developed exclusively by Drew Ridley."))
    .bold()
    .centered()
    .render(area, buf);
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized 
    {
         // Create a space for header, todo list and the footer.
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ]);

        let [header_area, rest_area, footer_area] = vertical.areas(area);
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [upper_item_list_area, lower_item_list_area] = vertical.areas(rest_area);

        render_title(header_area, buf);
        render_footer(footer_area, buf);

        self.render_list(rest_area, buf);
    }   
    
}
