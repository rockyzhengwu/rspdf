# rspdf

<img src="./doc/images/rspdf.png" alt="rspdf" style="width:40%;" />

A PDF library written in Rust work in progress.

# Motivation
Write a PDF processiong library in pure Rust.

# Usage

## extract text as xml
eg:

```
cargo run -- --filename <File> --start 0 --end 1 pdftotext --output page0.xml 
```

## render text
Can just render text on image now

```
cargo run -- --filename <File> --start 0 --end 1 pdftopng 
```




