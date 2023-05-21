use std::fmt::{Debug, Formatter};
use std::future;
use std::future::{Future, IntoFuture};
use std::num::ParseIntError;
use std::string::ParseError;
use std::sync::Arc;

use fibermc_sdk::apis::configuration::Configuration;
use fibermc_sdk::apis::mods_api::ApiV10ModsIdStatsGetError;
use fibermc_sdk::models::{ModResponse, ModStatsResponse};
use futures::{FutureExt, TryFutureExt};
use futures::future::OptionFuture;
use leptos::*;
use leptos_router::{IntoParam, Params, ParamsError, use_params, use_params_map};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::my_uuid::MyUuid;

#[derive(Params)]
#[derive(PartialEq)]
pub struct StatsPageParams {
    mod_id: MyUuid,
}

#[component]
pub fn StatsPage(
    cx: Scope,
    params: Memo<Result<StatsPageParams, ParamsError>>
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

    let mod_id_str = mod_id().ok().unwrap().get_uuid().hyphenated().to_string();
    let on_click = move |_| set_count.update(|count| *count += 1);

    log!("render, kinda!");

    let pretty_mod_id = move || mod_id().ok().map(|id| id.to_pretty_string());
    let requestConfig:&'static Configuration = &REQUEST_CONFIG;
    let modResponse = create_local_resource(
        cx,
        pretty_mod_id,
        move |id| async move {
            match id {
                Some(id) => get_mod(&requestConfig, id).await,
                None => None
            }
        }
    );

    let statsResponse = create_local_resource(
        cx,
        pretty_mod_id,
        move |id| async move {
            match id {
                Some(id) => get_stats(&requestConfig, id).await,
                None => None
            }
        }
    );

    let mod_downloads_over_time_str = move || statsResponse.with(cx, | res| res
        .as_ref()
        .map(|r| r
            .overall_stats
            .iter()
            .map(|el| format!("({}, {})", el.downloads, el.timestamp))
            .intersperse("\n".to_owned())
            .collect::<String>())
    );

    let mod_name = move || modResponse.with(cx, |res| res
        .as_ref()
        .map(|m| m.name.to_owned())
    );

    return view! { cx,
        <h1>"Stats for " {mod_name}</h1>
        <h2>"("{mod_id}")"</h2>
        <button on:click=on_click>"Click Me: " {count}</button>
        <div>{mod_downloads_over_time_str}</div>
    }
}

static REQUEST_CONFIG: Lazy<Configuration> = Lazy::new(|| Configuration {
    // base_path: "https://www.fibermc.com".to_owned(),
    base_path: "https://dev.fibermc.com".to_owned(),
    // base_path: "https://localhost:5001".to_owned(),
    user_agent: Some("OpenAPI-Generator/0.0.1/rust".to_owned()),
    client: reqwest::Client::new(),
    basic_auth: None,
    oauth_access_token: None,
    bearer_access_token: None,
    api_key: None,
});

async fn get_stats(requestConfig: &Configuration, mod_id: String) -> Option<ModStatsResponse> {
    let id_str = mod_id.as_str();
    // log!("request fn! {}", id_ref.unwrap_or("NO VALUE FOR MOD ID"));
    let result = fibermc_sdk::apis::mods_api::api_v10_mods_id_stats_get(
        &requestConfig,
        id_str)
        .await
        .ok();

    log!("after result! {}", id_str);
    return result;
}

async fn get_mod(requestConfig: &Configuration, mod_id: String) -> Option<ModResponse> {
    let result = fibermc_sdk::apis::mods_api::api_v10_mods_id_get(
        &requestConfig,
        mod_id.as_str())
        .await
        // .ok()
        ;
    // there is an error at deserialization time that we are throwing away here...
    // There _must_ be a better way to propagate these errors through in leptos...
    log!("after mod result! {}", mod_id);
    if let Err(err) = result.as_ref() {
        log!("after mod result! {}", err);
    }
    return result.ok();

}
