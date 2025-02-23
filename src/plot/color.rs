use image::Rgba;
use palette::{FromColor, Hsv, Srgb};
use plotters_backend::BackendColor;

#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const FONT_PRIMARY: Self = Self(0xf9, 0xf9, 0xf9);
    pub const BG_PRIMARY: Self = Self(0x4e, 0x2f, 0x63);
    pub const BG_SECONDARY: Self = Self(0x71, 0x50, 0x7c);
    pub const ACCENT_PINK: Self = Self(0xed, 0x0d, 0x7f);
    pub const ACCENT_BLUE: Self = Self(0x42, 0xbc, 0xec);
    pub const ACCENT_YELLOW: Self = Self(0xfa, 0xe6, 0x16);

    fn from_hsv(angle: f64) -> Self {
        let color = Srgb::from_color(Hsv::new_srgb(angle, 1.0, 1.0));
        Self(
            (color.red * 255.0) as u8,
            (color.green * 255.0) as u8,
            (color.blue * 255.0) as u8,
        )
    }
}

impl plotters::style::Color for Color {
    fn to_backend_color(&self) -> BackendColor {
        BackendColor {
            alpha: 1.0,
            rgb: (self.0, self.1, self.2),
        }
    }
}

impl From<Color> for Rgba<u8> {
    fn from(value: Color) -> Self {
        Self([value.0, value.1, value.2, 0xff])
    }
}

#[derive(Debug)]
pub struct ColorIterator {
    dangle: f64,
    i: f64,
}

impl Iterator for ColorIterator {
    type Item = Color;

    /// Guaranteed to return `Some`
    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1.0;
        Some(Color::from_hsv(self.dangle * self.i))
    }
}

impl ColorIterator {
    #[must_use]
    pub fn new(spacing: usize, elements: usize) -> Self {
        Self {
            dangle: (360.0 - (360.0 / elements as f64)) / spacing as f64,
            i: -1.0,
        }
    }
}
