use std::{error::Error, io::stdout, panic};

use dotenv::dotenv;
use log::LevelFilter;
use sqlx::mysql::MySqlPoolOptions;

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

mod crud;
mod models;
mod ui;

use models::*;

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let _ = execute!(
            stdout(),
            EnterAlternateScreen, // Move to an alternate screen buffer if available
            Clear(ClearType::All),
            MoveTo(0, 0) // Move cursor to the top-left corner
        );

        // Ensure the panic message is visible
        println!("{}", info);

        // Give some time to read the message
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Return to the original screen buffer
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    {
        simple_logging::log_to_file("tats.log", LevelFilter::Trace)
            .expect("Failed to initialize logger!");
    }

    dotenv().ok();

    let db_url = std::env::args().nth(1).expect("You must provide the db url as a cli argument. Example: [mariadb://user:password@ip:port/tats]");
    println!("Url: {}", db_url);
    let pool = MySqlPoolOptions::new()
        .connect(db_url.as_str())
        .await
        .unwrap();

    let terminal = init_terminal()?;

    setup_panic_hook();

    // Clear the terminal at the start of the program
    execute!(stdout(), Clear(ClearType::All))?;

    crate::ui::prelude::App::new().run(terminal, &pool).await?;
    Ok(())
}
