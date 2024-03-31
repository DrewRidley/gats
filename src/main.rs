use std::{default, error::Error, io::stdout};

use sqlx::{mysql::MySqlPoolOptions, prelude::FromRow, MySqlPool};
use dotenv::dotenv;

mod models;
use models::*;

mod ui;



async fn fetch_projects(pool: &MySqlPool) -> Result<Vec<Project>, sqlx::Error> {
    let projects = sqlx::query_as::<_, Project>("SELECT * FROM Project")
    .fetch_all(pool)
    .await?;
    Ok(projects)
}

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen}, ExecutableCommand};
use ratatui::prelude::*;

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let database_url = "mariadb://root:password1@localhost:3306/gats";
    let pool = MySqlPoolOptions::new().connect(database_url).await.unwrap();

    println!("Connected to db successfully??");

    let projects = fetch_projects(&pool).await.unwrap();

    println!("Total projects: {}", projects.len());

    let terminal = init_terminal()?;
    ui::App::new().run(terminal, &pool).await?;

    Ok(())
}