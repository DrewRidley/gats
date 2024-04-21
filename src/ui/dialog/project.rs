
use std::default;

use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend, layout::{self, Alignment, Constraint, Direction, Layout}, style::{Color, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, ListState}, Terminal
};
use sqlx::{MySql, MySqlPool, Pool};

use crate::Member;



async fn fetch_members_by_project_id(pool: &MySqlPool, project_id: i32) -> Result<Vec<Member>, sqlx::Error> {
    let members = sqlx::query_as::<_, Member>(
        r#"
        SELECT Member.MemberID, Member.firstName, Member.lastName, Member.email, Member.phone
        FROM Member
        INNER JOIN ContributesTo ON Member.MemberID = ContributesTo.MemberID
        WHERE ContributesTo.ProjectID = ?
        "#
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(members)
}


pub struct ProjectMembersDialog {
    //Cursor vertical and horizontal index.
    cursor: (usize, usize),
    members: Vec<Member>,
    new_id: String
}

impl ProjectMembersDialog {
    fn new() -> Self {
        Self {
            cursor: (0,0),
            members: vec![],
            new_id: Default::default()
        }
    }

    fn handle_key_event(
        key_event: KeyEvent, 
        cursor: &mut (usize, usize), 
        members_len: usize, 
        input_name: &mut String, 
        submit_action: &dyn Fn(String)  // A callback function for submitting the name
    ) -> (bool, bool) {
        match key_event.code {
            KeyCode::Down => {
                cursor.0 = (cursor.0 + 1) % (members_len + 3); // +3 for the two input areas and submit button
            },
            KeyCode::Up => {
                cursor.0 = if cursor.0 > 0 { cursor.0 - 1 } else { members_len + 2 };
            },
            KeyCode::Right if cursor.0 >= members_len => {
                cursor.1 = (cursor.1 + 1) % 2; // Toggle between input and submit button
            },
            KeyCode::Left if cursor.0 >= members_len => {
                cursor.1 = if cursor.1 > 0 { cursor.1 - 1 } else { 1 };
            },
            KeyCode::Char(c) if cursor.0 == members_len => {
                input_name.push(c); // Append character to input field if on input field
            },
            KeyCode::Backspace if cursor.0 == members_len => {
                input_name.pop(); // Remove last character from input field
            },
            KeyCode::Enter if cursor.0 == members_len => {
                submit_action(input_name.clone()); // Call the submit action with the entered name
                input_name.clear(); // Clear input field after submission

                //Continue running and refresh.
                return (true, true);
            },
            KeyCode::Esc => return (false, false), // Stop running
            _ => {}
        }
        (true, false) // Continue running but do not refresh.
    }

    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
        project_id: i32
    ) -> std::io::Result<()> {
        let mut diag = ProjectMembersDialog::new();
        diag.members = fetch_members_by_project_id(pool, project_id).await.expect("Failed to fetch members!");

        let submit_action = move |name: String| {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                let result = sqlx::query!(
                    "INSERT INTO ContributesTo (MemberID, ProjectID) VALUES (?, ?)",
                    name,  // Assuming `name` is the MemberID you want to add; adjust accordingly if it's not
                    project_id
                )
                .execute(&pool_clone)
                .await;
    
                match result {
                    Ok(_) => {},
                    Err(e) => panic!("Failed to add member to project: {}", e),
                }
            });
        };

        loop {
            diag.draw(&mut terminal)?;
    
            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    let (cont, refresh) = Self::handle_key_event(key_event, &mut diag.cursor, diag.members.len(), &mut diag.new_id, &submit_action);

                    if refresh {
                        diag.members = fetch_members_by_project_id(pool, project_id).await.expect("Failed to update project members list...");
                    }

                    if !cont {
                        return Ok(())
                    }
                }
            }
        }
    }
    
    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|frame| {
            let mut items: Vec<ListItem> = Vec::new();
            for (i, member) in self.members.iter().enumerate() {
                // Determine if the current item should be highlighted based on the cursor position
                let highlight = i == self.cursor.0 && self.cursor.1 == 1;
                
                
                let member_info = Span::raw(format!("{} {} - {} | ", member.first_name, member.last_name, member.email));
                let (trash_can_icon, trash_can_style) = if highlight {
                    ("❌", Style::default().bold())  
                } else {
                    ("🗑", Style::default().fg(Color::White))  
                };
                
                let trash_can = Span::styled(trash_can_icon, trash_can_style);
                
                
                let line = Line::from(vec![member_info, trash_can]);
                items.push(ListItem::new(line));
            }

            let new_member_line = Line::from(vec![
                Span::from("New Member (Enter ID): "),
                Span::styled(
                    self.new_id.clone(),
                    Style::default().fg(Color::Yellow) 
                )
            ]);

            items.push(ListItem::new(new_member_line));
    

            let list = List::new(items)
                .block(
                    Block::default()
                    .borders(Borders::ALL)
                    .title("Project Members")
                    .title_alignment(layout::Alignment::Center)
                )
                .highlight_style(
                    Style::default().fg(Color::Yellow)  // Default highlight for the whole line
                );
    
           // Define layout constraints
           let chunks = Layout::default()
           .direction(Direction::Vertical)
           .constraints([
               Constraint::Percentage(100),
           ])
           .split(frame.size());

        // List state for handling selection
        let mut list_state = ListState::default();
        list_state.select(Some(self.cursor.0));

        // Render the list widget with the state
        frame.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;
        Ok(())
    }
}


pub struct CreateProjectDialog {
    cursor: usize,
    name: String,
    desc: String,
}

impl CreateProjectDialog {
    fn new() -> Self {
        Self {
            cursor: 0,
            name: String::new(),
            desc: String::new(),
        }
    }

    pub async fn run(
        mut terminal: &mut Terminal<impl Backend>,
        pool: &MySqlPool,
    ) -> std::io::Result<()> {
        let mut diag = CreateProjectDialog::new();

        loop {
            diag.draw(&mut terminal)?;

            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(c) => match diag.cursor {
                            0 => diag.name.push(c),
                            1 => diag.desc.push(c),
                            _ => {}
                        },
                        KeyCode::Enter => {
                            if diag.cursor == 2 {
                                //Actually create the project and return.
                                let query =
                                    "INSERT INTO Project (Title, Description) VALUES (?, ?)";
                                sqlx::query(query)
                                    .bind(&diag.name)
                                    .bind(&diag.desc)
                                    .execute(pool)
                                    .await
                                    .expect("Failed to insert project into database!");

                                return Ok(());
                            }
                        }
                        KeyCode::Backspace => match diag.cursor {
                            0 => {
                                diag.name.pop();
                            }
                            1 => {
                                diag.desc.pop();
                            }
                            _ => {}
                        },
                        KeyCode::Down => {
                            diag.cursor = (diag.cursor + 1) % 5;
                        }
                        KeyCode::Up => {
                            diag.cursor = if diag.cursor > 0 { diag.cursor - 1 } else { 3 };
                        }
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|frame| {
            let list = List::new(vec![
                ListItem::new(format!("Project Name: {}", self.name)),
                ListItem::new(format!("Description: {}", self.desc)),
                ListItem::new("Create Project"),
            ]);

            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(frame.size());

            let mut list_state = ListState::default();
            list_state.select(Some(self.cursor));

            let action_list = list
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Create new Project")
                        .title_alignment(ratatui::layout::Alignment::Center),
                )
                .highlight_symbol(">")
                .highlight_style(
                    ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                );

            frame.render_stateful_widget(action_list, chunks[0], &mut list_state);
        })?;

        Ok(())
    }
}
