use crate::{
    error::Result,
    font::CharCode,
    geom::path::Path,
    page::{
        graphics_state::{FillRule, GraphicsState},
        image::PdfImage,
    },
};

pub mod trace;

pub trait Device {
    fn start_page(
        &mut self,
        state: &GraphicsState,
        page_num: u32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        Ok(())
    }

    fn clip(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }

    fn end_page(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn draw_char(&mut self, char: &CharCode, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn begin_text(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn end_text(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn draw_image(&mut self, image: PdfImage, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn fill_path(&mut self, path: &Path, state: &GraphicsState, rule: FillRule) -> Result<()> {
        Ok(())
    }
    fn stroke_path(&mut self, path: &Path, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn fill_and_stroke_path(
        &mut self,
        path: &Path,
        state: &GraphicsState,
        rull: FillRule,
    ) -> Result<()> {
        Ok(())
    }
    fn update_font(&mut self, state: &GraphicsState) -> Result<()> {
        Ok(())
    }
    fn hdpi(&self) -> f32 {
        72.0
    }
    fn vdpi(&self) -> f32 {
        72.0
    }
}
