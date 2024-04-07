use std::fs::File;
use std::io::{Cursor, Read};
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
    Pdffonts(commands::pdffonts::Config),
    Pdftrace(commands::pdftrace::Config),
    Pdfimages(commands::pdfimages::Config),
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let filename = cli.filename;
    let command = cli.command;

    let start = cli.start.unwrap_or(0);
    let mut file = File::open(filename.as_path()).unwrap();
    // load all to memory or just use file
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let cursor = Cursor::new(buffer);
    let doc = Document::open(cursor).unwrap();
    let end = cli.end.unwrap_or(doc.page_count().unwrap());
    info!(
        "Process {:?} for page:{} to page {}",
        filename.display(),
        start,
        end
    );

    match command {
        Commands::Pdftotext(cfg) => {
            commands::pdftotext::command(doc, start, end, cfg, filename).unwrap()
        }
        Commands::Pdftopng(cfg) => {
            commands::pdftopng::command(doc, start, end, cfg, filename).unwrap()
        }
        Commands::Pdffonts(cfg) => {
            commands::pdffonts::command(doc, start, end, cfg, filename).unwrap()
        }
        Commands::Pdftrace(cfg) => {
            commands::pdftrace::command(doc, start, end, cfg, filename).unwrap()
        }
        Commands::Pdfimages(cfg) => {
            commands::pdfimages::command(doc, start, end, cfg, filename).unwrap()
        }
    }
}
