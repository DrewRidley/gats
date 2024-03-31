use ratatui::{
    backend::Backend, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, Paragraph, Widget, Wrap}, Terminal
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[derive(Clone, PartialEq, Eq)]
enum TaskManagerCursor {
    ViewTasks,
    AddTask,
    UpdateTask,
    DeleteTask,
    BackToMainMenu,
}

impl TaskManagerCursor {
    fn next(&mut self) {
        *self = match *self {
            TaskManagerCursor::ViewTasks => TaskManagerCursor::AddTask,
            TaskManagerCursor::AddTask => TaskManagerCursor::UpdateTask,
            TaskManagerCursor::UpdateTask => TaskManagerCursor::DeleteTask,
            TaskManagerCursor::DeleteTask => TaskManagerCursor::BackToMainMenu,
            TaskManagerCursor::BackToMainMenu => TaskManagerCursor::ViewTasks,
        }
    }

    fn prev(&mut self) {
        *self = match *self {
            TaskManagerCursor::ViewTasks => TaskManagerCursor::BackToMainMenu,
            TaskManagerCursor::AddTask => TaskManagerCursor::ViewTasks,
            TaskManagerCursor::UpdateTask => TaskManagerCursor::AddTask,
            TaskManagerCursor::DeleteTask => TaskManagerCursor::UpdateTask,
            TaskManagerCursor::BackToMainMenu => TaskManagerCursor::DeleteTask,
        }
    }
}

pub struct TaskManager {
    cursor: TaskManagerCursor,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            cursor: TaskManagerCursor::ViewTasks,
        }
    }

    fn get_menu_lines(&self) -> Vec<Span> {
        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD);

        let menu_items = vec![
            TaskManagerCursor::ViewTasks,
            TaskManagerCursor::AddTask,
            TaskManagerCursor::UpdateTask,
            TaskManagerCursor::DeleteTask,
            TaskManagerCursor::BackToMainMenu,
        ];

        menu_items
            .iter()
            .map(|item| {
                let name = match item {
                    TaskManagerCursor::ViewTasks => "View Tasks",
                    TaskManagerCursor::AddTask => "Add Task",
                    TaskManagerCursor::UpdateTask => "Update Task",
                    TaskManagerCursor::DeleteTask => "Delete Task",
                    TaskManagerCursor::BackToMainMenu => "Back to Main Menu",
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
                                TaskManagerCursor::ViewTasks => {
                                    // TODO: Implement view tasks functionality
                                }
                                TaskManagerCursor::AddTask => {
                                    // TODO: Implement add task functionality
                                }
                                TaskManagerCursor::UpdateTask => {
                                    // TODO: Implement update task functionality
                                }
                                TaskManagerCursor::DeleteTask => {
                                    // TODO: Implement delete task functionality
                                }
                                TaskManagerCursor::BackToMainMenu => return Ok(()),
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


impl Widget for &mut TaskManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::default()
            .title("Manage Tasks")
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