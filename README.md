# rspdf


<img src="./doc/images/rspdf.png" alt="rspdf" style="width:40%;" />

A PDF library written in Rust work in progress. Focus on extract content and render at now.

# Motivation
Write a PDF processiong library in pure Rust.

# Tool Usage

## extract text as xml
extract plain  text
```
cargo run -- --filename <File> pdftotext
```

## render text
Can just render text on image now

```
cargo run -- --filename <File> pdftopng 
```

## extract  fonts
extract font info
```
cargo run -- --filename <File> pdffonts
```


## trace
trace pdf object render info, char position , path , image
```
cargo run -- --filename <File> pdftrace
```
