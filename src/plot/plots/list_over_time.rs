use std::{fs, iter, path::Path};

use anyhow::{anyhow, Result};
use plotters::{
    chart::ChartBuilder,
    prelude::{BitMapBackend, BitMapElement, IntoDrawingArea, Polygon},
    series::LineSeries,
};
use tracing::info;

use crate::{
    data::{Data, LOGO_FILENAME},
    plot::{
        color::{Color, ColorIterator},
        font::Font,
        img,
        marker::{Marker, MarkerKind},
    },
};

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 1556;
const MARGIN: u32 = 64;
const X_LABEL_AREA_SIZE: u32 = 16;
const Y_LABEL_AREA_SIZE: u32 = 384;
const X_TICK_SPACING: usize = 10;
const LOGO_WIDTH_SCALE: u32 = 255;
const LOGO_WIDTH_NOSCALE: u32 = 510;
const LOGO_HEIGHT_SCALE: u32 = 235;
const LOGO_HEIGHT_NOSCALE: u32 = 270;
const LOGO_Y_SCALE: f64 = 0.01;
const LOGO_Y_NOSCALE: f64 = 0.8;

const COLOR_SPACING: usize = 4;

pub fn list_over_time<P>(path: P, scale: bool, data: &Data) -> Result<()>
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
    let dates = data.dates();

    let root = BitMapBackend::new(&path, (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&Color::BG_PRIMARY)?;

    let mut chart = ChartBuilder::on(&root)
        .top_x_label_area_size(X_LABEL_AREA_SIZE)
        .right_y_label_area_size(Y_LABEL_AREA_SIZE)
        .margin(MARGIN)
        .build_cartesian_2d(1..num_games, 1.0..0.0)?
        .set_secondary_coord(1..num_games, (num_games - 1)..0);

    chart
        .configure_secondary_axes()
        .y_labels(num_games)
        .y_label_formatter(&|i| data.metas.0[&latest_list.0[*i]].name.clone())
        .x_labels(num_games / X_TICK_SPACING)
        .x_label_formatter(&|i| format!("{} ({})", i, dates[*i - 1].0))
        .label_style(Font::default())
        .axis_style(Color::FONT_PRIMARY)
        .draw()?;

    chart.draw_series(iter::once(Polygon::new(
        if scale {
            vec![(1, 0.0), (num_games, 0.0), (num_games, 1.0), (1, 1.0)]
        } else {
            vec![(1, 0.0), (num_games, 1.0), (num_games, 0.0)]
        },
        Color::BG_SECONDARY,
    )))?;

    let logo = img::load(
        &fs::read(LOGO_FILENAME)?,
        if scale {
            LOGO_WIDTH_SCALE
        } else {
            LOGO_WIDTH_NOSCALE
        },
        if scale {
            LOGO_HEIGHT_SCALE
        } else {
            LOGO_HEIGHT_NOSCALE
        },
        if scale {
            Color::BG_SECONDARY
        } else {
            Color::BG_PRIMARY
        },
    )?;

    chart.draw_series(iter::once(BitMapElement::from((
        (2, if scale { LOGO_Y_SCALE } else { LOGO_Y_NOSCALE }),
        logo,
    ))))?;

    let mut colors = ColorIterator::new(COLOR_SPACING, num_games);

    for (i, id) in latest_list.0.iter().enumerate() {
        let color = colors.next().unwrap();
        let points = dates
            .iter()
            .enumerate()
            .filter_map(|(idx, date)| {
                let list = &data.lists.0[date];
                list.0.iter().position(|x| x == id).map(|position| {
                    (
                        idx + 1,
                        if scale {
                            position as f64 / idx as f64
                        } else {
                            position as f64 / (num_games - 1) as f64
                        },
                    )
                })
            })
            .collect::<Vec<_>>();
        chart.draw_series(points.iter().copied().map(|coord| {
            Marker::new(
                match (i / COLOR_SPACING) % MarkerKind::COUNT {
                    0 => MarkerKind::Triangle,
                    1 => MarkerKind::Circle,
                    2 => MarkerKind::Cross,
                    _ => unreachable!(),
                },
                coord,
                color,
            )
        }))?;
        chart.draw_series(LineSeries::new(points.iter().copied(), color))?;
    }

    root.present()?;

    info!(
        "Generated visualization {}",
        path.as_ref().to_string_lossy()
    );

    Ok(())
}
