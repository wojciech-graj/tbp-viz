use plotters::style::{HasDimension, IntoTextStyle, TextStyle};

use super::color::Color;

#[derive(Debug)]
pub struct Font {
    name: &'static str,
    size: u32,
    color: &'static Color,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            name: "Rubik",
            size: 24,
            color: &Color::FONT_PRIMARY,
        }
    }
}

impl Font {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            ..Default::default()
        }
    }
}

impl<'a> IntoTextStyle<'a> for Font {
    fn into_text_style<P>(self, parent: &P) -> TextStyle<'a>
    where
        P: HasDimension,
    {
        (self.name, self.size, self.color).into_text_style(parent)
    }
}
