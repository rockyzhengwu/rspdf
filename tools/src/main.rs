use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use log::info;

use pdf::document::Document;
mod commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, value_name = "FILE")]
    filename: PathBuf,

    #[arg(short, long)]
    start: Option<u32>,
    #[arg(short, long)]
    end: Option<u32>,
}

#[derive(Parser, Debug)]
enum Commands {
    Pdftotext(commands::pdftotext::Config),
    Pdftopng(commands::pdftopng::Config),
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let filename = cli.filename;
    let start = cli.start.unwrap_or(0);
    let end = cli.end.unwrap_or(1);
    let command = cli.command;
    info!(
        "Process {:?},from page {} to {}, {:?}",
        filename, start, end, command
    );
    let file = File::open(filename).unwrap();
    let doc = Document::open(file).unwrap();

    match command {
        Commands::Pdftotext(cfg) => commands::pdftotext::command(doc, start, end, cfg).unwrap(),
        Commands::Pdftopng(cfg) => commands::pdftopng::command(doc, start, end, cfg).unwrap(),
    }
}
