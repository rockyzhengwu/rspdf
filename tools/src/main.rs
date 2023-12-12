use std::fs::File;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::info;

use pdf::canvas::processor::Processor;
use pdf::device::image_device::ImageDevice;
use pdf::document;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    filename: PathBuf,

    #[arg(short, long)]
    start: Option<u32>,
    #[arg(short, long)]
    end: Option<u32>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Text {},
    Image {},
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let filename = cli.filename;
    let start_page = cli.start.unwrap_or(0);
    let end_page = cli.end.unwrap_or(0);
    info!(
        "Process {:?},from page {} to {}",
        filename, start_page, end_page
    );

    let file = File::open(filename).unwrap();

    let device = ImageDevice::new(300.0, 300.0);

    let doc = document::Document::open(file).unwrap();
    let page = doc.page(0).unwrap();
    let mut processor = Processor::new(&doc, Box::new(device));
    processor.process_page_content(page).unwrap();
}
