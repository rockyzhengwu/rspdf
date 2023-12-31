# rspdf

<img src="./doc/images/rspdf.png" alt="rspdf" style="width:40%;" />

A PDF library written in Rust work in progress. Focus on extract content and render at now.

# Motivation
Write a PDF processiong library in pure Rust.

# Usage

## extract text as xml

```
cargo run -- --filename <File> --start 0 --end 1 pdftotext --output page0.xml 
```

## render text
Can just render text on image now

```
cargo run -- --filename <File> --start 0 --end 1 pdftopng 
```

## extract  fonts
```
cargo run -- --filename pdffonts
```


## TODO
- [ ] fonts
  - [ ] refactor font, just two font Struct Simple Font and Type0 Font.
  - [ ] default built in font, encoding, tounicode
  - [ ] face optional , because face will never used when just extract text

- [ ] text
  - [ ] merge text chunk to line, program, and add label to text line like title ？

- [ ] render
  - [ ] path
  - [ ] text render
  - [ ] image
