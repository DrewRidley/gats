use ratatui::{backend::Backend, widgets::{Paragraph, Widget}, Terminal};


#[derive(Clone, Copy)]
pub struct SprintManager {

}

impl SprintManager {
    pub fn new() -> Self {
        SprintManager {}
    }

    /// Will show this menu until the user decides to exit. If the user decides to exit the entire program (CTRL C), it will not return to the callee.
    pub fn run(mut terminal: &mut Terminal<impl Backend>) -> std::io::Result<()>  {
        let mgr = Self::new();
        
        loop {
            terminal.draw(|f| f.render_widget(mgr, f.size()))?;
        }
    }
}

impl Widget for SprintManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        
        let para = Paragraph::new("Hello Sprints!");

        para.render(area, buf);
    }
}