#![feature(no_coverage)]
mod game;
mod random;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use dotenvy::dotenv;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A science fiction game inspired by Aurora4X.",
    long_about = "A science fiction 4x written in rust."
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Run a turn of the game.")]
    Run,
}

#[no_coverage]
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    dotenv()?;

    let args = Args::parse();
    match args.command.unwrap() {
        Commands::Run => game::run().await,
    }
}
