use std::{f64::consts::PI, fs, path::Path, time::Duration};

use anyhow::{anyhow, Result};
use plotters::{
    chart::ChartBuilder,
    prelude::{BitMapBackend, BitMapElement, Circle, IntoDrawingArea},
    series::AreaSeries,
    style::ShapeStyle,
};
use tracing::info;

use crate::{
    data::{Data, LOGO_FILENAME},
    plot::{color::Color, font::Font, img, range::OffsetDateTimeRange},
};

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 389;
const MARGIN: u32 = 64;
const Y_MARGIN_LOGO: i32 = 16;
const LOGO_WIDTH: u32 = 425;
const LOGO_HEIGHT: u32 = 225;
const X_LABEL_AREA_SIZE: u32 = 56;
const BUCKET_WIDTH: Duration = Duration::from_secs(60 * 60 * 24);
const KERNEL_SIGMA: f64 = 150.0;

fn gaussian_kernel(sigma: f64) -> Vec<f64> {
    let num_points = (2 * (3.0 * sigma).ceil() as usize) + 1;
    (0..num_points)
        .map(|i| {
            (-0.5 * ((i as f64 - num_points as f64 / 2.0) / sigma).powi(2)).exp()
                / (sigma * (2.0 * PI).sqrt())
        })
        .collect()
}

pub fn release_dates<P>(path: P, data: &Data) -> Result<()>
where
    P: AsRef<Path>,
{
    info!(
        "Generating visualization {}",
        path.as_ref().to_string_lossy()
    );

    let kernel = gaussian_kernel(KERNEL_SIGMA);
    let (start_date, end_date) = data
        .release_date_range()
        .ok_or_else(|| anyhow!("Could not calculate release date range."))?;

    let root = BitMapBackend::new(&path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&Color::BG_PRIMARY)?;

    let logo = img::load(
        &fs::read(LOGO_FILENAME)?,
        LOGO_WIDTH,
        LOGO_HEIGHT,
        Color::BG_PRIMARY,
    )?;
    root.draw(&BitMapElement::from(((MARGIN as i32, Y_MARGIN_LOGO), logo)))?;

    let mut buckets = (0..((end_date - start_date) / BUCKET_WIDTH).ceil() as usize)
        .map(|i| (start_date + BUCKET_WIDTH * i as u32 + BUCKET_WIDTH / 2, 0.0))
        .collect::<Vec<_>>();

    for meta in data.metas.0.values() {
        let i = ((meta.first_release_date - start_date) / BUCKET_WIDTH - kernel.len() as f64 / 2.0)
            .round() as i32;
        for (d, &s) in buckets
            .iter_mut()
            .skip(i.max(0) as usize)
            .zip(kernel.iter().skip((-i).max(0) as usize))
        {
            d.1 += s;
        }
    }

    let max_bucket = buckets.iter().fold(0.0, |acc, (_, x)| x.max(acc));
    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .margin(MARGIN)
        .build_cartesian_2d(
            OffsetDateTimeRange {
                start: start_date,
                end: end_date,
            },
            0.0..max_bucket,
        )?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc("Release Date")
        .label_style(Font::default())
        .axis_style(Color::FONT_PRIMARY)
        .draw()?;

    chart.draw_series(
        AreaSeries::new(buckets, 0.0, Color::ACCENT_BLUE).border_style(Color::FONT_PRIMARY),
    )?;

    chart.draw_series(data.metas.0.values().map(|meta| {
        Circle::new(
            (meta.first_release_date, 0.0),
            4,
            ShapeStyle::from(Color::ACCENT_YELLOW).filled(),
        )
    }))?;

    info!(
        "Generated visualization {}",
        path.as_ref().to_string_lossy()
    );

    Ok(())
}
