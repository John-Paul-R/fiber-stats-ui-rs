use std::fmt::{Debug, Formatter};
use std::future;
use std::future::{Future, IntoFuture};
use std::num::ParseIntError;
use std::string::ParseError;
use std::sync::Arc;
use fibermc_sdk::apis::configuration::Configuration;
use fibermc_sdk::apis::mods_api::ApiV10ModsIdStatsGetError;
use fibermc_sdk::models::ModStatsResponse;
use futures::{FutureExt, TryFutureExt};

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

    let requestConfig:&'static Configuration = &REQUEST_CONFIG;
    let statsResponse = create_local_resource(
        cx,
        move || mod_id().ok().map(|id| id.to_pretty_string()),
        move |id| async move {
            get_stats(&requestConfig, id).await
        }
    );

    // let mod_name: &str = "Essential Commands";

    let mod_downloads_over_time_str = move || statsResponse
        .read(cx)
        // .as_ref()
        .and_then(|v| v)
        .map(|r| r.0
            .overall_stats
            .iter()
            .cloned()
            .map(|el| format!("({}, {})", el.downloads, el.timestamp))
            .intersperse("\n".to_owned())
            .collect::<String>());

    return view! { cx,
        <h1>"Stats for " <b>{mod_id}</b></h1>
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

#[derive(Serialize, Deserialize, Clone)]
struct MyModStatsResponse(ModStatsResponse);

impl MyModStatsResponse {
    pub fn get(self) -> ModStatsResponse {
        return self.0
    }
}

async fn get_stats(requestConfig: &Configuration, mod_id: Option<String>) -> Option<MyModStatsResponse> {
    let id_ref = mod_id.as_ref().map(|s| s.as_str());
    log!("request fn! {}", id_ref.unwrap_or("NO VALUE FOR MOD ID"));
    let result = match id_ref {
        Some(id_str) => fibermc_sdk::apis::mods_api::api_v10_mods_id_stats_get(
            &requestConfig,
            id_str)
            .await.ok()
            .map(MyModStatsResponse),
        None => None,
    };

    log!("after result! {}", id_ref.unwrap_or("NO VALUE FOR MOD ID"));
    return result;
}

async fn parse_stats(
    fut_res: Box<Result<ModStatsResponse, fibermc_sdk::apis::Error<ApiV10ModsIdStatsGetError>>>
) -> MyModStatsResponse
{
    // Box::into_pin(fut_res)
        // .await
    fut_res
        .map(MyModStatsResponse)
        .unwrap_or_else(|_| todo!())
}
