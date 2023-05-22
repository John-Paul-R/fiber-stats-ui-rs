use std::fmt::Debug;
use std::time::Duration;

use chrono::*;
use fibermc_sdk::models::{ModResponse, ModStatsResponse, TimestampedModStats};
use leptos::*;
use leptos_router::{IntoParam, Params, ParamsError};
use plotters::chart::SeriesAnno;
use plotters::prelude::*;
use plotters::prelude::full_palette::GREEN_900;
use plotters::style::full_palette::{GREEN_200, GREEN_300, GREEN_600, ORANGE_200, ORANGE_300, ORANGE_600};
use plotters_canvas::CanvasBackend;

use crate::my_uuid::MyUuid;
use crate::requests::mods::{get_mod, get_stats};

#[derive(Params)]
#[derive(PartialEq)]
pub struct StatsPageParams {
    mod_id: MyUuid,
}

#[component]
#[allow(non_snake_case)]
pub fn StatsPage(
    cx: Scope,
    params: Memo<Result<StatsPageParams, ParamsError>>,
) -> impl IntoView {
    let mod_id = move || {
        params.clone().with(|p_outer| {
            match p_outer {
                Ok(value) => Ok(value.mod_id.clone()),
                Err(err) => Err(err.clone()),
            }
        })
    };

    log!("render, kinda!");

    let pretty_mod_id = move || mod_id().ok().map(|id| id.to_pretty_string());
    let mod_response = create_local_resource(
        cx,
        pretty_mod_id,
        move |id| async move {
            match id {
                Some(id) => get_mod(id).await,
                None => None
            }
        },
    );

    let stats_response = create_local_resource(
        cx,
        pretty_mod_id,
        move |id| async move {
            match id {
                Some(id) => get_stats(id).await,
                None => None
            }
        },
    );

    let mod_downloads_over_time_str = move || stats_response.with(cx, |res| res
        .as_ref()
        .map(|r| r
            .overall_stats
            .iter()
            .map(|el| format!("({}, {})", el.downloads, el.timestamp))
            .map(|s| view! { cx, <div>{s}</div> }.into_view(cx))
            .collect::<View>()),
    );

    let mod_overview = move || mod_response.with(cx, |res| res
        .as_ref()
        .map(|m| view! {cx, <StatsPageModSummary mod_response=m.clone()/>}),
    );

    let mod_stats_view = move || stats_response.with(cx, |res| res
        .as_ref()
        .map(|msr| view! {cx, <ModStatsSection mod_stats=msr.clone()/>}.into_view(cx))
        .unwrap_or_else(|| view! {cx, <p>"Err!"</p>}.into_view(cx)),
    );

    view! { cx,
        {mod_overview}
        <div>{mod_downloads_over_time_str}</div>
        {mod_stats_view}
    }
}

#[component]
#[allow(non_snake_case)]
fn StatsPageModSummary(
    cx: Scope,
    mod_response: ModResponse,
) -> impl IntoView {
    view! { cx,
        <h1>"Stats for " {mod_response.name}</h1>
        <div>"("{mod_response.id.hyphenated().to_string()}")"</div>
        <p>{mod_response.summary}</p>
        <div>
            <b>"Downloads: "</b>
            {mod_response.download_count}
        </div>
    }
}

fn parse_to_timestamp(s: &str) -> i64 {
    DateTime::parse_from_rfc3339(s)
        .unwrap()
        .timestamp_millis()
}

fn parse_to_timestamps(s: &Vec<TimestampedModStats>) -> Vec<i64> {
    s.iter()
        .map(|s| s.timestamp.as_str())
        .map(parse_to_timestamp)
        .collect()
}

fn line_series_from_mod_stats<S>(s: &Vec<TimestampedModStats>, style: S) -> LineSeries<CanvasBackend, (i64, i64)>
    where S: Into<ShapeStyle> {
    LineSeries::new(
        s.iter()
            .cloned()
            .map(|s| (s.downloads, parse_to_timestamp(&s.timestamp)))
            // .zip(timestamps)
            .map(|(dl, t)| (t, dl)),
        style,
    )
}

#[component]
#[allow(non_snake_case)]
fn ModStatsSection(
    cx: Scope,
    mod_stats: ModStatsResponse,
) -> impl IntoView {
    set_timeout(move || {
        if mod_stats.overall_stats.is_empty() {
            return;
        }
        let padding_frac = 0.3f32;
        let max_downloads = mod_stats.overall_stats.iter().map(|s| s.downloads).max().unwrap();
        let upper_downloads_axis_bound = ((max_downloads as f32) * (1f32 + padding_frac)) as i64;
        let timestamps = parse_to_timestamps(&mod_stats.overall_stats);
        let min_date = timestamps.iter().min().unwrap().clone();
        let max_date = timestamps.iter().max().unwrap().clone();

        let overall_series = line_series_from_mod_stats(&mod_stats.overall_stats, &BLUE);
        let modrinth_series = line_series_from_mod_stats(&mod_stats.modrinth_stats, &GREEN_600);
        let curse_series = line_series_from_mod_stats(&mod_stats.curse_forge_stats, &ORANGE_600);

        draw_series(
            "my_plot",
            (min_date, 0i64),
            (max_date, upper_downloads_axis_bound),
            vec![overall_series, modrinth_series, curse_series],
        ).unwrap();

        ()
    }, Duration::from_secs(0));

    view! { cx,
        <div>
            <h3>"Stats"</h3>
            <canvas id="my_plot" width=1024 height=1024/>
        </div>
    }
}

/// Type alias for the result of a drawing function.
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn draw_series(
    canvas_id: &str,
    min: (i64, i64),
    max: (i64, i64),
    series: Vec<LineSeries<CanvasBackend, (i64, i64)>>,
) -> DrawResult<impl Fn((i32, i32)) -> Option<(i64, i64)>> {
    let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(64u32)
        .caption(format!("Downloads Over Time"), font)
        .x_label_area_size(30u32)
        .y_label_area_size(30u32)
        .build_cartesian_2d(min.0..max.0, min.1..max.1)?;

    chart.configure_mesh().x_labels(3).y_labels(3)
        .x_label_formatter(&|v| NaiveDateTime::from_timestamp_millis(v.clone())
            .unwrap()
            .format("%Y-%m-%d")
            .to_string())
        .draw()?;

    for s in series {
        chart.draw_series(s.point_size(5))?;
    }

    root.present()?;
    return Ok(chart.into_coord_trans());
}
