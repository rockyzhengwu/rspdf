use pdf::device::Device;
use std::fs::File;
use std::io::Write;

pub struct ImageDevice {
    num: u32,
}

impl ImageDevice {
    pub fn new() -> Self {
        ImageDevice { num: 0 }
    }
}

impl Device for ImageDevice {
    fn draw_image(
        &mut self,
        image: pdf::page::image::PdfImage,
        _state: &pdf::page::graphics_state::GraphicsState,
    ) -> pdf::error::Result<()> {
        let width = image.width() as u32;
        let height = image.height() as u32;
        let rgb = image.rgb_image()?;
        let mut data = Vec::new();

        for h in 0..height {
            for w in 0..width {
                let pos = h * width + w;
                let c = rgb.get(pos as usize).unwrap();
                data.push((c.r * 255.0) as u8);
                data.push((c.g * 255.0) as u8);
                data.push((c.b * 255.0) as u8);
            }
        }
        let name = image.name();
        let fname = format!("{}_{}.ppm", self.num, name);
        self.num += 1;
        save_as_ppm(fname.as_str(), width, height, data).unwrap();

        Ok(())
    }
    fn vdpi(&self) -> f32 {
        72.0
    }

    fn hdpi(&self) -> f32 {
        72.0
    }
}

pub fn save_as_ppm(filename: &str, width: u32, height: u32, data: Vec<u8>) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "P6\n{} {}\n255", width, height)?;
    file.write_all(data.as_slice())?;

    Ok(())
}
