use fibermc_sdk::models::ModResponse;
use leptos::*;
use leptos_router::{IntoParam, Params, ParamsError};

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

    let (count, set_count) = create_signal(cx, 0);

    let on_click = move |_| set_count.update(|count| *count += 1);

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
            .intersperse("\n".to_owned())
            .collect::<String>()),
    );

    let mod_overview = move || mod_response.with(cx, |res| res
        .as_ref()
        .map(|m| view! {cx, <StatsPageModSummary mod_response=m.clone()/>}),
    );

    view! { cx,
        {mod_overview}
        <button on:click=on_click>"Click Me: " {count}</button>
        <div>{mod_downloads_over_time_str}</div>
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
