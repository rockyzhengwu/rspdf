use std::fs::File;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use log::info;

use pdf::canvas::processor::Processor;
use pdf::device::image_device::ImageDevice;
use pdf::device::text::TextDevice;
use pdf::document::Document;
use pdf::errors::PDFResult;

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
    command: Commands,
}
#[derive(Args, Debug)]
struct TextArgs {
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Args, Debug)]
struct RenderArgs {
    #[arg(short, long, default_value_t = 300.0)]
    resulotion: f64,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Text(TextArgs),
    Render(RenderArgs),
}

fn extract_text(doc: Document<File>, start: u32, end: u32, cfg: TextArgs) -> PDFResult<()> {
    let device = Box::new(TextDevice::new(cfg.output));
    let mut processor = Processor::new(&doc, device.clone());
    for p in start..end {
        let page = doc.page(p).unwrap();
        processor.process_page_content(page).unwrap();
    }
    processor.close()?;
    Ok(())
}

fn render(doc: Document<File>, start: u32, end: u32, cfg: RenderArgs) -> PDFResult<()> {
    let device = ImageDevice::new(cfg.resulotion, cfg.resulotion);
    let mut processor = Processor::new(&doc, Box::new(device));
    for p in start..end {
        let page = doc.page(p).unwrap();
        processor.process_page_content(page).unwrap();
    }
    Ok(())
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let filename = cli.filename;
    let start = cli.start.unwrap_or(0);
    let end = cli.end.unwrap_or(0);
    let command = cli.command;
    info!(
        "Process {:?},from page {} to {}, {:?}",
        filename, start, end, command
    );
    let file = File::open(filename).unwrap();
    let doc = Document::open(file).unwrap();

    match command {
        Commands::Text(cfg) => extract_text(doc, start, end, cfg).unwrap(),
        Commands::Render(cfg) => render(doc, start, end, cfg).unwrap(),
    }
}
