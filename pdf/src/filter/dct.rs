use crate::{
    error::{PdfError, Result},
    object::dictionary::PdfDict,
};

pub fn dct_decode(input: &[u8], _params: Option<&PdfDict>) -> Result<Vec<u8>> {
    let d = mozjpeg::Decompress::new_mem(input)
        .map_err(|e| PdfError::Filter(format!("DCT create decoder error:{:?}", e)))?;
    let image = d
        .image()
        .map_err(|e| PdfError::Filter(format!("DCT decoder read decoder error:{:?}", e)))?;

    let res = match image {
        mozjpeg::Format::RGB(mut rgb) => {
            let mut res = Vec::new();
            let pixels: Vec<[u8; 3]> = rgb.read_scanlines().unwrap();
            for pixel in pixels {
                res.push(pixel[0]);
                res.push(pixel[1]);
                res.push(pixel[2]);
            }
            res
        }
        mozjpeg::Format::Gray(mut g) => {
            let pixels: Vec<u8> = g.read_scanlines().unwrap();
            pixels
        }
        mozjpeg::Format::CMYK(mut cmyk) => {
            let mut res = Vec::new();
            let pixels: Vec<[u8; 4]> = cmyk.read_scanlines().unwrap();
            // TODO why this order ?
            for pixel in pixels {
                res.push(pixel[0]);
                res.push(pixel[1]);
                res.push(pixel[2]);
                res.push(pixel[3]);
                //res.push(pixel[2]);
                //res.push(pixel[1]);
                //res.push(pixel[0]);
                //res.push(pixel[3]);
            }
            res
        }
    };

    Ok(res)
}
