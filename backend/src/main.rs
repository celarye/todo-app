use std::{process::ExitCode, sync::Arc};

use tracing::{error, info};
use tracing_subscriber;

mod app;
mod database;
mod logic;
use app::App;
use database::Database;
use logic::Logic;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt::init();

    info!("Creating a database connection pool");
    let Ok(database) = Database::connect("database.sqlite3", 5).await else {
        error!("Exiting the program");
        return ExitCode::from(1);
    };

    info!("Starting the web API");
    if let Err(_) = App::run(
        "0.0.0.0",
        8080,
        String::from("https://todo.celarye.dev"),
        Arc::new(Logic::new(database)),
    )
    .await
    {
        error!("Exiting the program");
        return ExitCode::from(1);
    };

    info!("Exiting the program");
    ExitCode::from(0)
}
