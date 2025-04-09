use std::path::PathBuf;
mod commands;
use clap::Parser;
use pdf::document::Document;
mod device;

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
    Pdffonts(commands::pdffonts::Config),
    Trace(commands::trace::Config),
    Images(commands::pdfimages::Config),
    Pdftotext(commands::pdftotext::Config),
}

fn main() {
    let cli = Cli::parse();
    let filename = cli.filename;
    let command = cli.command;
    let doc = Document::new_from_file(filename, None).unwrap();
    let start = match cli.start {
        Some(s) => s,
        None => 1,
    };
    let total_page = doc.total_page().unwrap();
    let end = match cli.end {
        Some(v) => {
            if v > total_page {
                total_page
            } else {
                v
            }
        }
        None => total_page,
    };

    match command {
        Commands::Trace(cfg) => {
            commands::trace::command(&doc, cfg, start, end);
        }
        Commands::Pdffonts(cfg) => {
            commands::pdffonts::command(&doc, cfg, start, end);
        }
        Commands::Images(cfg) => {
            commands::pdfimages::command(&doc, cfg, start, end);
        }
        Commands::Pdftotext(cfg) => {
            commands::pdftotext::command(&doc, cfg, start, end);
        }
    }
}
