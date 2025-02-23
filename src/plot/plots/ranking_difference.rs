use std::{fs, iter, path::Path};

use anyhow::{anyhow, Result};
use plotters::{
    chart::ChartBuilder,
    prelude::{BitMapBackend, BitMapElement, IntoDrawingArea, Polygon},
    series::LineSeries,
};
use tracing::info;

use crate::{
    data::{Data, RatingKind, LOGO_FILENAME},
    plot::{
        color::{Color, ColorIterator},
        font::Font,
        img,
    },
};

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 1556;
const COLOR_SPACING: usize = 10;
const CURVE_POINTS: usize = (WIDTH - 2 * (MARGIN + Y_LABEL_AREA_SIZE)) as usize;
const MARGIN: u32 = 64;
const LOGO_MARGIN: i32 = 16;
const Y_LABEL_AREA_SIZE: u32 = 384;

fn ease_in_out_cubic(x: f64) -> f64 {
    if x < 0.5 {
        4.0 * x.powi(3)
    } else {
        1.0 - (-2.0f64).mul_add(x, 2.0).powi(3) / 2.0
    }
}

pub fn ranking_difference<P>(path: P, kind: RatingKind, data: &Data) -> Result<()>
where
    P: AsRef<Path>,
{
    info!(
        "Generating visualization {}",
        path.as_ref().to_string_lossy()
    );

    let latest_list = data
        .latest()
        .ok_or_else(|| anyhow!("Latest list doesn't exist"))?;
    let num_games = latest_list.0.len();
    let igdb_list = data.igdb_list(kind);

    let root = BitMapBackend::new(&path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&Color::BG_PRIMARY)?;

    let logo = img::load(&fs::read(LOGO_FILENAME)?, 170, 90, Color::BG_PRIMARY)?;
    root.draw(&BitMapElement::from(((LOGO_MARGIN, LOGO_MARGIN), logo)))?;

    let mut chart = ChartBuilder::on(&root)
        .y_label_area_size(Y_LABEL_AREA_SIZE)
        .right_y_label_area_size(Y_LABEL_AREA_SIZE)
        .margin(MARGIN)
        .build_cartesian_2d(0.0..1.0, ((num_games - 1) as f64)..0.0)?
        .set_secondary_coord(0..0, (igdb_list.len() - 1)..0);

    chart
        .configure_mesh()
        .disable_mesh()
        .y_labels(num_games)
        .y_label_formatter(&|i| {
            data.metas.0[&latest_list.0[i.round() as usize]]
                .name
                .clone()
        })
        .y_desc("Bonus Point Ranking")
        .label_style(Font::default())
        .axis_style(Color::FONT_PRIMARY)
        .draw()?;

    chart
        .configure_secondary_axes()
        .y_labels(igdb_list.len())
        .y_label_formatter(&|i| {
            format!("({:.0}) {}", igdb_list[*i].0.round(), igdb_list[*i].1.name)
        })
        .y_desc(kind.to_string())
        .label_style(Font::default())
        .axis_style(Color::FONT_PRIMARY)
        .draw()?;

    chart.draw_series(iter::once(Polygon::new(
        vec![
            (1.0, 0.0),
            (0.0, 0.0),
            (0.0, (num_games - 1) as f64),
            (1.0, (num_games - 1) as f64),
        ],
        Color::BG_SECONDARY,
    )))?;

    let mut colors = ColorIterator::new(COLOR_SPACING, num_games);

    for (i, id) in latest_list.0.iter().enumerate() {
        let color = colors.next().unwrap();
        if let Some(igdb_pos) = igdb_list.iter().position(|meta| meta.1.id == *id) {
            let start = i as f64;
            let end = igdb_pos as f64 * (num_games - 1) as f64 / (igdb_list.len() - 1) as f64;

            chart.draw_series(LineSeries::new(
                (0..CURVE_POINTS).map(|i| {
                    let x = i as f64 / CURVE_POINTS as f64;
                    (x, ease_in_out_cubic(x).mul_add(end - start, start))
                }),
                color,
            ))?;
        }
    }

    root.present()?;

    info!(
        "Generated visualization {}",
        path.as_ref().to_string_lossy()
    );

    Ok(())
}
