use crate::errors::{PDFError, PDFResult};

pub fn dct_decode(buf: &[u8]) -> PDFResult<Vec<u8>> {
    let d = mozjpeg::Decompress::new_mem(buf)
        .map_err(|e| PDFError::Filter(format!("DCT create decoder error:{:?}", e)))?;
    let image = d
        .image()
        .map_err(|e| PDFError::Filter(format!("DCT decoder read decoder error:{:?}", e)))?;
    let res = match image {
        mozjpeg::Format::RGB(mut rgb) => {
            let pixels: Vec<[u8; 3]> = rgb.read_scanlines().unwrap();
            pixels.concat()
        }
        mozjpeg::Format::Gray(mut g) => {
            let pixels: Vec<u8> = g.read_scanlines().unwrap();
            pixels
        }
        mozjpeg::Format::CMYK(mut cmyk) => {
            let pixels: Vec<[u8; 4]> = cmyk.read_scanlines().unwrap();
            pixels.concat()
        }
    };
    Ok(res)
}
