use crate::font::composite_font::CompositeFont;
use crate::font::simple_font::SimpleFont;

#[derive(Debug, Clone)]
pub enum Font {
    Simple(SimpleFont),
    Composite(CompositeFont),
}

// TODO impl type0

impl Font {}
