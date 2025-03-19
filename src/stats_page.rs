use std::io::Cursor;
use std::ops::Range;
use std::time::Duration;

use chrono::*;
use fibermc_sdk::models::{ModResponse, ModStatsResponse, TimestampedModStats};
use leptos::control_flow::{For, Show};
use leptos::html::{Div, Script};
use leptos::logging::log;
use leptos::prelude::{
    set_timeout, ElementChild, GlobalAttributes, LocalResource, Memo, NodeRef,
    NodeRefAttribute, With,
};
use leptos::suspense::Suspense;
use leptos::*;
use leptos_router::params::{Params, ParamsError};
use plotters::prelude::*;
use plotters::style::full_palette::{BLUE_600, GREEN_600, ORANGE_600};
use quick_xml::events::Event;
use quick_xml::Writer;
use wasm_bindgen::UnwrapThrowExt;

use crate::my_uuid::MyUuid;
use crate::requests::mods::{get_mod, get_stats};

#[derive(Params, PartialEq)]
pub struct StatsPageParams {
    mod_id: MyUuid,
}

#[component]
#[allow(non_snake_case)]
pub fn StatsPage(
    params: Memo<Result<StatsPageParams, ParamsError>>,
) -> impl IntoView {
    let mod_id = move || {
        params.clone().with(|p_outer| match p_outer {
            Ok(value) => Ok(value.mod_id.clone()),
            Err(err) => Err(err.clone()),
        })
    };

    log!("render, kinda!");

    let pretty_mod_id = move || mod_id().ok().map(|id| id.to_pretty_string());
    let mod_response = LocalResource::new(move || async move {
        match pretty_mod_id() {
            Some(id) => get_mod(id).await,
            None => None,
        }
    });

    let stats_response = LocalResource::new(move || async move {
        match pretty_mod_id() {
            Some(id) => get_stats(id).await,
            None => None,
        }
    });

    let ModDownloadsOverTimeView = move || {
        let stats = move || {
            stats_response.with(|res| {
                res.as_ref()
                    .and_then(|r| r.as_ref())
                    .map(|r| r.overall_stats.clone())
            })
        };

        view! {
            <Show
                when=move || stats().is_some()
                fallback=move || view! { <p>"No stats available"</p> }
            >
                <For
                    each=move || stats().unwrap_or_default()
                    key=|el| el.timestamp.clone()
                    let:el
                >
                    <div>{format!("({}, {})", el.downloads, el.timestamp)}</div>
                </For>
            </Show>
        }
    };

    let ModOverviewView = move || {
        mod_response.with(|res| {
            res.as_ref()
                .and_then(|res| res.as_ref())
                .map(|m| view! {<StatsPageModSummary mod_response=m.clone()/>})
        })
    };

    let ModStatsView = move || {
        let maybe_stats = move || {
            stats_response
                .with(|res| res.as_ref().and_then(|msr| msr.as_ref()).cloned())
        };

        view! {
            <Show
                when=move || maybe_stats().is_some()
                fallback=move || view! { <p>"Err!"</p> }
            >
                <ModStatsSection mod_stats=maybe_stats().unwrap()/>
            </Show>
        }
    };

    view! {
        <Suspense
            fallback=move || view! { <p>"Loading..."</p> }
        >
            {ModOverviewView}
            <details>
                <summary>"View Data Points List"</summary>
                <div>{ModDownloadsOverTimeView}</div>
            </details>
            {ModStatsView}
        </Suspense>
    }
}

#[component]
#[allow(non_snake_case)]
fn StatsPageModSummary(mod_response: ModResponse) -> impl IntoView {
    view! {
        <title>{format!("Stats for {}", mod_response.name)}</title>
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
    DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis()
}

fn parse_to_timestamps(s: &[TimestampedModStats]) -> Vec<i64> {
    s.iter()
        .map(|s| s.timestamp.as_str())
        .map(parse_to_timestamp)
        .collect()
}

fn line_series_from_mod_stats<S>(
    s: &[TimestampedModStats],
    style: S,
) -> LineSeries<SVGBackend, (i64, i64)>
where
    S: Into<ShapeStyle>,
{
    LineSeries::new(
        s.iter()
            .cloned()
            .map(|s| (parse_to_timestamp(&s.timestamp), s.downloads)),
        style,
    )
}

#[component]
#[allow(non_snake_case)]
fn ModStatsSection(mod_stats: ModStatsResponse) -> impl IntoView {
    let container_ref = NodeRef::<Div>::new();
    let script_ref = NodeRef::<Script>::new();

    set_timeout(
        move || render_chart(&mod_stats, container_ref),
        Duration::from_secs(0),
    );

    script_ref.on_load(|el| el.set_src("../assets/js/circles.js"));

    view! {
        <div>
            <style>
                """
                #my_plot {
                    background-color: unset; /*var(--color-base-1);*/
                }
                #my_plot text {
                    fill: var(--color-text) !important;
                }
                #my_plot line {
                    stroke: var(--color-text) !important;
                }
                #my_plot polyline[stroke=\"#000000\"] {
                    stroke: var(--color-text) !important;
                }
                #my_plot circle::after {
                    content: attr(data-content);
                    width: 32px;
                    height: 32px;
                    background-color: red;
                }
                #my_plot_tooltip {
                    background-color: var(--color-element-1);
                }
                """
            </style>

            <script node_ref=script_ref />
            <h3>"Stats"</h3>
            <div id="my_plot" node_ref=container_ref />
            <div id="my_plot_tooltip" />
        </div>
    }
}

fn render_chart(mod_stats: &ModStatsResponse, container_ref: NodeRef<Div>) {
    if mod_stats.overall_stats.is_empty() {
        return;
    }
    let padding_frac = 0.3f32;
    let max_downloads = mod_stats
        .overall_stats
        .iter()
        .map(|s| s.downloads)
        .max()
        .unwrap();
    let upper_downloads_axis_bound =
        ((max_downloads as f32) * (1f32 + padding_frac)) as i64;
    let timestamps = parse_to_timestamps(&mod_stats.overall_stats);
    let min_date = *timestamps.iter().min().unwrap();
    let max_date = *timestamps.iter().max().unwrap();

    let overall_series =
        line_series_from_mod_stats(&mod_stats.overall_stats, BLUE_600);
    let modrinth_series =
        line_series_from_mod_stats(&mod_stats.modrinth_stats, GREEN_600);
    let curse_series =
        line_series_from_mod_stats(&mod_stats.curse_forge_stats, ORANGE_600);

    log!(
        "Overall data points count: {}",
        &mod_stats.overall_stats.len()
    );
    log!(
        "Modrinth data points count: {}",
        &mod_stats.modrinth_stats.len()
    );
    log!(
        "CurseForge data points count: {}",
        &mod_stats.curse_forge_stats.len()
    );

    let svg_string = draw_series(
        min_date..max_date,
        0i64..upper_downloads_axis_bound,
        vec![overall_series, modrinth_series, curse_series],
    )
    .unwrap();

    let mut svg_reader = ::quick_xml::reader::Reader::from_str(&svg_string);
    let all_points: Vec<TimestampedModStats> = {
        let mut pts = mod_stats.overall_stats.clone();
        pts.append(&mut mod_stats.modrinth_stats.clone());
        pts.append(&mut mod_stats.curse_forge_stats.clone());
        pts
    };

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut idx = 0;
    loop {
        match svg_reader.read_event() {
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Empty(e)) if e.name().as_ref() == b"circle" => {
                log!("circle {}", idx);
                let point = &all_points[idx];
                idx += 1;
                let mut elem = e.into_owned();
                elem.push_attribute(("data-y", &*point.downloads.to_string()));
                elem.push_attribute(("data-x", &*point.timestamp.to_string()));
                assert!(writer.write_event(Event::Empty(elem)).is_ok());
            }
            Ok(e) => {
                assert!(writer.write_event(e).is_ok())
            }
            Err(e) => panic!(
                "Error at position {}: {:?}",
                svg_reader.buffer_position(),
                e
            ),
        }
    }
    let svg_blob = writer.into_inner().into_inner();
    let res = std::str::from_utf8(&svg_blob)
        .map(|data| data.to_string())
        .expect_throw(
            "Failed to generate a download counts svg from the mod_response",
        );

    container_ref.on_load(move |f| f.set_inner_html(&res));
}

/// Type alias for the result of a drawing function.
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn draw_series(
    x_range: Range<i64>,
    y_range: Range<i64>,
    series: Vec<LineSeries<SVGBackend, (i64, i64)>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut svg_content: String = "".to_string();
    {
        let backend = SVGBackend::with_string(&mut svg_content, (800, 600));
        let root = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();

        let mut chart = ChartBuilder::on(&root)
            .margin(64u32)
            .caption("Downloads Over Time", font)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .x_labels(5)
            .y_labels(8)
            .x_label_formatter(&|v| {
                DateTime::from_timestamp_millis(*v)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string()
            })
            .draw()?;

        for s in series {
            chart.draw_series(s.point_size(2))?;
        }

        root.present()?;
    }
    Ok(svg_content)
}
