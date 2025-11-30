use anyhow::Result;
use image::{DynamicImage, imageops::FilterType};

use super::color::Color;

pub fn load(img: &[u8], nwidth: u32, nheight: u32, bg: Color) -> Result<DynamicImage> {
    let mut img = image::load_from_memory(img)?
        .resize(nwidth, nheight, FilterType::Nearest)
        .into_rgba8();
    for pixel in img.pixels_mut() {
        if pixel.0[3] == 0x00 {
            *pixel = bg.into();
        }
    }
    Ok(DynamicImage::ImageRgba8(img))
}
