use std::{fs, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use plotters::{
    coord::Shift,
    prelude::{BitMapBackend, BitMapElement, DrawingArea, IntoDrawingArea, Rectangle},
    style::{IntoTextStyle, ShapeStyle},
};
use plotters_backend::{
    text_anchor::{HPos, Pos, VPos},
    DrawingBackend,
};
use tokio::task::{JoinSet, LocalSet};
use tracing::info;

use crate::{
    data::{Data, LOGO_FILENAME},
    join_local,
    plot::{color::Color, font::Font, img},
    request::resource::{ImageSize, ResourceRequestor},
};

const WIDTH: u32 = 4096;
const HEIGHT: u32 = 1556;
const NUM_SEGMENTS: u32 = 7;
const SEGMENT_WIDTH: u32 = WIDTH / NUM_SEGMENTS;
const MARGIN: u32 = 16;
const TITLE_HEIGHT: u32 = 80;
const ITEM_GAP: u32 = 16;
const ITEM_TITLE_HEIGHT: u32 = 32;
const NUM_OVERRATED: usize = 5;
const NUM_UNDERRATED: usize = 5;
const NUM_GAME_ENGINES: usize = 4;
const NUM_COMPANIES: usize = 7;
const NUM_PLATFORMS: usize = 5;
const LOGO_WIDTH: u32 = 170;
const LOGO_HEIGHT: u32 = 90;
const TITLE_FONT_SIZE: u32 = 96;
const FONT_SIZE: u32 = 32;

#[allow(clippy::too_many_lines)]
pub async fn summary<P>(path: &'static P, data: Arc<Data>) -> Result<()>
where
    P: AsRef<Path> + ?Sized,
{
    info!(
        "Generating visualization {}",
        path.as_ref().to_string_lossy()
    );

    let root = BitMapBackend::new(path, (WIDTH, HEIGHT)).into_drawing_area();

    let roots = root.split_evenly((1, NUM_SEGMENTS as usize));

    let mut tasks = JoinSet::new();
    let local_tasks = LocalSet::new();

    {
        let root = roots[0].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                draw_segment(
                    root,
                    "List Toppers",
                    data.extrema(true)
                        .iter()
                        .map(|(id, duration)| {
                            let meta = &data.metas.0[id];
                            (
                                meta.cover.as_ref().map(|url_field| url_field.url.as_str()),
                                format!("{} days", duration.whole_days()),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_PRIMARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    {
        let root = roots[1].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                draw_segment(
                    root,
                    "Barrel Bottoms",
                    data.extrema(false)
                        .iter()
                        .map(|(id, duration)| {
                            let meta = &data.metas.0[id];
                            (
                                meta.cover.as_ref().map(|url_field| url_field.url.as_str()),
                                format!("{} days", duration.whole_days()),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_SECONDARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    {
        let root = roots[2].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                draw_segment(
                    root,
                    "Overrated",
                    data.igdb_diffs()
                        .ok_or_else(|| anyhow!("Could not generate IGDB rating differences."))?
                        [..NUM_OVERRATED]
                        .iter()
                        .map(|(diff, meta)| {
                            (
                                meta.cover.as_ref().map(|url_field| url_field.url.as_str()),
                                format!("{diff:+} positions"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_PRIMARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    {
        let root = roots[3].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                let igdb_diffs = data
                    .igdb_diffs()
                    .ok_or_else(|| anyhow!("Could not generate IGDB rating differences."))?;
                draw_segment(
                    root,
                    "Underrated",
                    igdb_diffs[igdb_diffs.len() - NUM_UNDERRATED..]
                        .iter()
                        .rev()
                        .map(|(diff, meta)| {
                            (
                                meta.cover.as_ref().map(|url_field| url_field.url.as_str()),
                                format!("{diff:+} positions"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_SECONDARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    {
        let root = roots[4].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                draw_segment(
                    root,
                    "Game Engines",
                    data.most_common(
                        |meta| meta.game_engines.iter(),
                        |game_engine| game_engine.name.as_str(),
                    )[..NUM_GAME_ENGINES]
                        .iter()
                        .map(|(count, game_engine)| {
                            (
                                game_engine
                                    .logo
                                    .as_ref()
                                    .map(|url_field| url_field.url.as_str()),
                                format!("{count} games"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_PRIMARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    {
        let root = roots[5].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                draw_segment(
                    root,
                    "Companies",
                    data.most_common(
                        |meta| meta.involved_companies.iter(),
                        |involved_company| involved_company.company.name.as_str(),
                    )[..NUM_COMPANIES]
                        .iter()
                        .map(|(count, involved_company)| {
                            (
                                involved_company
                                    .company
                                    .logo
                                    .as_ref()
                                    .map(|url_field| url_field.url.as_str()),
                                format!("{count} games"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_SECONDARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    {
        let root = roots[6].clone();
        let data = data.clone();
        tasks.spawn_local_on(
            async move {
                draw_segment(
                    root,
                    "Platforms",
                    data.most_common(
                        |meta| meta.platforms.iter(),
                        |platform| platform.name.as_str(),
                    )[..NUM_PLATFORMS]
                        .iter()
                        .map(|(count, platform)| {
                            (
                                platform
                                    .platform_logo
                                    .as_ref()
                                    .map(|url_field| url_field.url.as_str()),
                                format!("{count} games"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &data.res,
                    Color::BG_PRIMARY,
                )
                .await
            },
            &local_tasks,
        );
    }

    join_local(tasks, local_tasks).await?;

    let logo = img::load(
        &fs::read(LOGO_FILENAME)?,
        LOGO_WIDTH,
        LOGO_HEIGHT,
        Color::BG_PRIMARY,
    )?;
    root.draw(&BitMapElement::from((
        (
            (WIDTH - MARGIN - LOGO_WIDTH) as i32,
            (HEIGHT - MARGIN - LOGO_HEIGHT) as i32,
        ),
        logo,
    )))?;

    root.present()?;

    info!(
        "Generated visualization {}",
        path.as_ref().to_string_lossy()
    );

    Ok(())
}

async fn draw_segment<DB>(
    root: DrawingArea<DB, Shift>,
    title: &str,
    items: &[(Option<&str>, String)],
    res: &ResourceRequestor,
    bg: Color,
) -> Result<()>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    root.fill(&bg)?;
    let root = root.margin(MARGIN, MARGIN, MARGIN, MARGIN);

    root.draw_text(
        title,
        &Font::new(TITLE_FONT_SIZE)
            .with_anchor::<Color>(Pos {
                h_pos: HPos::Center,
                v_pos: VPos::Top,
            })
            .into_text_style(&root),
        (SEGMENT_WIDTH as i32 / 2, 0),
    )?;
    root.draw(&Rectangle::new(
        [
            (MARGIN as i32, (TITLE_HEIGHT - 2) as i32),
            ((SEGMENT_WIDTH - MARGIN) as i32, TITLE_HEIGHT as i32),
        ],
        ShapeStyle::from(Color::FONT_PRIMARY).filled(),
    ))?;

    let image_height =
        (HEIGHT - 2 * MARGIN - TITLE_HEIGHT) / items.len() as u32 - ITEM_GAP - ITEM_TITLE_HEIGHT;

    for (i, (url, text)) in items.iter().enumerate() {
        let y = TITLE_HEIGHT + i as u32 * (image_height + ITEM_GAP + ITEM_TITLE_HEIGHT) + ITEM_GAP;

        if let Some(url) = url {
            let image = res.get(ImageSize::Hd, url).await?;
            let image = img::load(&image, SEGMENT_WIDTH - 2 * MARGIN, image_height, bg)?;
            root.draw(&BitMapElement::from((
                (
                    (((SEGMENT_WIDTH - 2 * MARGIN) - image.width()) / 2) as i32,
                    (y + ITEM_TITLE_HEIGHT + (image_height - image.height() as u32) / 2) as i32,
                ),
                image,
            )))?;
        }

        root.draw_text(
            text,
            &Font::new(FONT_SIZE)
                .with_anchor::<Color>(Pos {
                    h_pos: HPos::Center,
                    v_pos: VPos::Top,
                })
                .into_text_style(&root),
            ((SEGMENT_WIDTH / 2) as i32, y as i32),
        )?;
    }

    Ok(())
}
