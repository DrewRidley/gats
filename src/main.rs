use std::{error::Error, io::stdout};

use dotenv::dotenv;
use log::LevelFilter;
use sqlx::mysql::MySqlPoolOptions;

mod models;
use models::*;

mod ui;

use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
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
    simple_logging::log_to_file("tats.log", LevelFilter::Trace)
        .expect("Failed to initialize logger!");
    dotenv().ok();
    let database_url = "mariadb://root:password1@localhost:3306/gats";
    let pool = MySqlPoolOptions::new().connect(database_url).await.unwrap();

    let terminal = init_terminal()?;
    ui::App::new().run(terminal, &pool).await?;

    Ok(())
}
