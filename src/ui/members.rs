use crossterm::event::{read, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{block::Title, Block, Borders, List},
    Terminal,
};
use sqlx::MySqlPool;

use crate::Member;

use super::dialog::prelude::{ConfirmDelete, CreateRecordDialog, CreateResults};

pub struct MemberManager {
    members: Vec<Member>,
    cursor: usize,
}

impl MemberManager {
    pub async fn new(pool: &MySqlPool) -> Self {
        let mut manager = MemberManager {
            members: vec![],
            cursor: 0,
        };

        manager.fetch_members(pool).await;
        manager
    }

    pub async fn fetch_members(&mut self, pool: &MySqlPool) {
        let fetch_query = "SELECT MemberID, firstName, lastName, email, phone FROM Member";
        let members = sqlx::query_as::<_, Member>(fetch_query)
            .fetch_all(pool)
            .await
            .expect("Failed to fetch members");
        self.members = members;
    }

    pub async fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        pool: &MySqlPool,
    ) -> std::io::Result<CreateResults> {
        loop {
            self.draw(terminal)?;

            if self.members.len() == 0 {
                self.fetch_members(pool).await;
            }

            if let Event::Key(key_event) = read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('c') => {
                            // Create a new member
                            match CreateRecordDialog::new(
                                vec![
                                    "First Name".into(),
                                    "Last Name".into(),
                                    "Email".into(),
                                    "Phone".into(),
                                ],
                                |d: &CreateRecordDialog| true,
                            )
                            .run(terminal)
                            .await?
                            {
                                CreateResults::Create(data) => {
                                    let insert_query = "INSERT INTO Member (firstName, lastName, email, phone) VALUES (?, ?, ?, ?)";
                                    let result = sqlx::query(insert_query)
                                        .bind(&data[0]) // First Name
                                        .bind(&data[1]) // Last Name
                                        .bind(&data[2]) // Email
                                        .bind(&data[3]) // Phone
                                        .execute(pool)
                                        .await;

                                    match result {
                                        Ok(_) => {
                                            // Fetch members again to update the list with new member
                                            self.fetch_members(pool).await;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to create member: {}", e);
                                            // Handle error accordingly
                                        }
                                    }
                                }
                                CreateResults::Quit => {
                                    // Handle quitting the create dialog
                                }
                            };
                        }
                        KeyCode::Char('e') => {
                            // Edit an existing member
                            let current_member = &self.members[self.cursor];
                            let current_data = vec![
                                current_member.first_name.clone(),
                                current_member.last_name.clone(),
                                current_member.email.clone(),
                                current_member.phone.clone(),
                            ];

                            match CreateRecordDialog::new_edit(
                                vec![
                                    "First Name".into(),
                                    "Last Name".into(),
                                    "Email".into(),
                                    "Phone".into(),
                                ],
                                current_data,
                                |d: &CreateRecordDialog| true,
                            )
                            .run(terminal)
                            .await?
                            {
                                CreateResults::Create(data) => {
                                    let update_query = "UPDATE Member SET firstName = ?, lastName = ?, email = ?, phone = ? WHERE MemberID = ?";
                                    let result = sqlx::query(update_query)
                                        .bind(&data[0]) // First Name
                                        .bind(&data[1]) // Last Name
                                        .bind(&data[2]) // Email
                                        .bind(&data[3]) // Phone
                                        .bind(current_member.member_id) // MemberID
                                        .execute(pool)
                                        .await;

                                    match result {
                                        Ok(_) => {
                                            // Fetch members again to update the list with edited member
                                            self.fetch_members(pool).await;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to update member: {}", e);
                                        }
                                    }
                                }
                                CreateResults::Quit => {
                                    return Ok(CreateResults::Quit);
                                }
                            };
                        }
                        KeyCode::Char('d') => {
                            if ConfirmDelete::run(terminal).await {
                                let current_member = &self.members[self.cursor];

                                let delete_contributions_query =
                                    "DELETE FROM ContributesTo WHERE MemberID = ?";
                                let _ = sqlx::query(delete_contributions_query)
                                    .bind(current_member.member_id)
                                    .execute(pool)
                                    .await;

                                let delete_query = "DELETE FROM Member WHERE MemberID = ?";
                                let result = sqlx::query(delete_query)
                                    .bind(current_member.member_id)
                                    .execute(pool)
                                    .await;

                                match result {
                                    Ok(_) => {
                                        self.fetch_members(pool).await;
                                    }
                                    Err(_) => {
                                        panic!("Failed to delete member!");
                                    }
                                }
                            }
                        }
                        KeyCode::Down => {
                            self.cursor += 1;
                        }
                        KeyCode::Up => {
                            self.cursor -= 1;
                        }
                        KeyCode::Esc => {
                            return Ok(CreateResults::Quit);
                        }
                        //Unknown input...
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&self, terminal: &mut Terminal<impl Backend>) -> std::io::Result<()> {
        terminal.draw(|f| {
            // Assuming a full screen size
            let size = f.size();

            let instructions_line = Line::from(vec![
                Span::styled("Create ", Style::default().fg(Color::Rgb(255, 165, 0))),
                Span::raw("<C> "),
                Span::styled("Edit ", Style::default().fg(Color::Rgb(255, 165, 0))),
                Span::raw("<E> "),
                Span::styled("Delete ", Style::default().fg(Color::Rgb(255, 165, 0))),
                Span::raw("<D> "),
            ]);

            let instructions_title = Title::from(instructions_line);

            let title_block = Block::default()
                .title(
                    instructions_title
                        .alignment(ratatui::layout::Alignment::Center)
                        .position(ratatui::widgets::block::Position::Bottom),
                )
                .borders(Borders::ALL)
                .title_alignment(ratatui::layout::Alignment::Center);

            // Create a list of member entries
            let members: Vec<Line> = self
                .members
                .iter()
                .enumerate()
                .map(|(index, member)| {
                    let style = if index == self.cursor {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };

                    // Create a single line with member details
                    let line = Line::from(vec![
                        Span::styled(format!("{} ", member.member_id), style),
                        Span::styled(
                            format!("{} {} ", member.first_name, member.last_name),
                            style,
                        ),
                        Span::styled(format!("{} ", member.email), style),
                        Span::styled(format!("{}", member.phone), style),
                    ]);

                    line
                })
                .collect();

            // Creating the List widget with member entries
            let member_list = List::new(members)
                .block(title_block) // Use the title block with instructions
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD),
                );

            // Render the list widget
            f.render_widget(member_list, size);
        })?;

        Ok(())
    }
}
