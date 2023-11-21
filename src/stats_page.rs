use std::ops::Range;
use std::time::Duration;

use chrono::*;
use fibermc_sdk::models::{ModResponse, ModStatsResponse, TimestampedModStats};
use leptos::html::Div;
use leptos::*;
use leptos_meta::*;
use leptos_router::{IntoParam, Params, ParamsError};
use plotters::prelude::*;
use plotters::style::full_palette::{BLUE_600, GREEN_600, ORANGE_600};

use crate::my_uuid::MyUuid;
use crate::requests::mods::{get_mod, get_stats};

#[derive(Params, PartialEq)]
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
        params.clone().with(|p_outer| match p_outer {
            Ok(value) => Ok(value.mod_id.clone()),
            Err(err) => Err(err.clone()),
        })
    };

    log!("render, kinda!");

    let pretty_mod_id = move || mod_id().ok().map(|id| id.to_pretty_string());
    let mod_response =
        create_local_resource(cx, pretty_mod_id, move |id| async move {
            match id {
                Some(id) => get_mod(id).await,
                None => None,
            }
        });

    let stats_response =
        create_local_resource(cx, pretty_mod_id, move |id| async move {
            match id {
                Some(id) => get_stats(id).await,
                None => None,
            }
        });

    let mod_downloads_over_time_vws = move || {
        stats_response.with(cx, |res| {
            res.as_ref().map(|r| {
                r.overall_stats
                    .iter()
                    .map(|el| format!("({}, {})", el.downloads, el.timestamp))
                    .map(|s| view! { cx, <div>{s}</div> }.into_view(cx))
                    .collect::<View>()
            })
        })
    };

    let mod_overview = move || {
        mod_response.with(cx, |res| {
            res.as_ref().map(
                |m| view! {cx, <StatsPageModSummary mod_response=m.clone()/>},
            )
        })
    };

    let mod_stats_view = move || {
        stats_response.with(cx, |res| {
            res.as_ref()
                .map(|msr| {
                    (view! {cx, <ModStatsSection mod_stats=msr.clone()/>})
                        .into_view(cx)
                })
                .unwrap_or_else(|| view! {cx, <p>"Err!"</p>}.into_view(cx))
        })
    };

    view! { cx,
        {mod_overview}
        <details>
            <summary>"View Data Points List"</summary>
            <div>{mod_downloads_over_time_vws}</div>
        </details>
        {mod_stats_view}
    }
}

#[component]
#[allow(non_snake_case)]
fn StatsPageModSummary(cx: Scope, mod_response: ModResponse) -> impl IntoView {
    view! { cx,
        <Title text={format!("Stats for {}", mod_response.name)}/>
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
fn ModStatsSection(cx: Scope, mod_stats: ModStatsResponse) -> impl IntoView {
    let container_ref = create_node_ref::<Div>(cx);

    set_timeout(
        move || render_chart(&mod_stats, container_ref),
        Duration::from_secs(0),
    );

    view! { cx,
        <div>
            <Style>
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
                """
            </Style>
            <h3>"Stats"</h3>
            <div id="my_plot" _ref=container_ref />
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

    container_ref
        .get()
        .expect("Element should be loaded by the time this setTimeout runs")
        .set_inner_html(&svg_string);
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
                NaiveDateTime::from_timestamp_millis(*v)
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
