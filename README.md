# rspdf


A PDF library written in Rust work in progress. Focus on extract content and render at now.

# Introduction
- ./pdf the core pdf lib
- ./tools command tools
- ./cairo-render render use cairo

## Usage



### Render 
  Render use cairo for vector drawing, can implement by other graphic library.

```shell
cargo run --example render --release -- [pdf path]
```

### trace
trace every char, image, path information in pdf.
```shell
cargo run --filename [pdf path] trace

```
or 
```shell
cargo run --filename [pdf path] --start [start page num] --end [end page  num] trace



```
Some other command tools can found in ./tools


### Define Customer Device

Just Implement Device trait , find example in ./tools

### Warn

There are many feature are not implement, Postscript Function, JBIG2 Filter,  Pattern Render, Search Font on System ...






