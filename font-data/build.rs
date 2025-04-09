use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;

fn format_name(name: &str) -> String {
    let mut name = name.replace('-', "_");
    name.insert(0, '_');
    name = name.to_uppercase();
    name
}

fn read_cmap(path: &PathBuf) -> Vec<u8> {
    let f = std::fs::File::open(path).unwrap();
    let reader = BufReader::new(f);
    let lines = reader.lines();
    let mut content: Vec<u8> = Vec::new();
    for line in lines {
        let line = line.unwrap();
        if !line.starts_with('%') {
            content.extend(line.as_bytes());
            content.push(b'\n');
        }
    }
    content
}
fn format_content(content: &[u8]) -> String {
    let mut bytes = String::new();
    for v in content.iter() {
        bytes.push_str(format!("{},", v).as_str());
    }
    bytes
}

fn generate_cmap_data() {
    let cmaps = std::fs::read_dir("./cmaps/").unwrap();
    let mut outfile = std::fs::File::create("./src/cmap.rs").unwrap();
    let mut branches = String::new();
    for cmap in cmaps {
        let cmap = cmap.unwrap();
        let path = cmap.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let var = format_name(name.as_str());

        let content = read_cmap(&path);
        let bytes = format_content(content.as_slice());
        let v = format!("const {}:&[u8]=&[{}];\n", var, bytes);
        outfile.write_all(v.as_bytes()).unwrap();
        let code = format!("\"{}\"=>Some({}),\n", name, var);
        branches.push_str(code.as_str());
    }
    branches.push_str("_=>None");
    let match_code = format!(
        "pub fn get_predefine_cmap_data(name:&str)->Option<&[u8]>\n{{match name\n{{{}}} }}",
        branches
    );
    outfile.write_all(match_code.as_bytes()).unwrap();
}

fn read_data(path: &PathBuf) -> Vec<u8> {
    let f = std::fs::File::open(path).unwrap();
    let mut reader = BufReader::new(f);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();
    buf
}

fn generate_font_data() {
    let fonts = std::fs::read_dir("./fonts/").unwrap();
    let outdir = PathBuf::from("./src/fonts");
    if !outdir.exists() {
        std::fs::create_dir(outdir.as_path()).unwrap();
    }
    let mut all_font = Vec::new();
    for font in fonts {
        let font = font.unwrap();
        let name = font
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let content = read_data(&font.path());
        let bytes = format_content(content.as_slice());
        let v = format!("pub const DATA:&[u8]=&[{}];\n", bytes);
        let outpath = outdir.clone().join(name.replace(".pfb", ".rs"));
        let mut outfile = std::fs::File::create(outpath).unwrap();
        outfile.write_all(v.as_bytes()).unwrap();
        all_font.push(name.replace(".pfb", ""));
    }
    let mut modfile = std::fs::File::create("./src/fonts/mod.rs").unwrap();
    for f in all_font {
        let l = format!("pub mod {};\n", f);
        modfile.write_all(l.as_bytes()).unwrap();
    }
}

fn generate_afm_data() {
    let afms = std::fs::read_dir("./afm/").unwrap();
    let outdir = PathBuf::from("./src/afm");
    let mut all_afm = Vec::new();
    if !outdir.exists() {
        std::fs::create_dir(outdir.as_path()).unwrap();
    }
    for f in afms {
        let fp = f.unwrap();
        let name = fp.path().file_name().unwrap().to_string_lossy().to_string();
        let content = read_data(&fp.path());
        let bytes = format_content(content.as_slice());
        let v = format!("pub const DATA:&[u8]=&[{}];\n", bytes);
        let outpath = outdir
            .clone()
            .join(name.to_lowercase().replace(".afm", ".rs").replace("-", "_"));
        let mut outfile = std::fs::File::create(outpath).unwrap();
        outfile.write_all(v.as_bytes()).unwrap();
        all_afm.push(name.replace(".afm", "").to_lowercase().replace("-", "_"));
    }
    let mut modfile = std::fs::File::create("./src/afm/mod.rs").unwrap();
    for f in all_afm {
        let l = format!("pub mod {};\n", f);
        modfile.write_all(l.as_bytes()).unwrap();
    }
}

fn main() {
    // TODO just exec once
    println!("cargo::rerun-if-changed=cmaps");
    generate_cmap_data();
    println!("cargo::rerun-if-changed=fonts");
    generate_font_data();
    println!("cargo::rerun-if-changed=afm");
    generate_afm_data();
}
