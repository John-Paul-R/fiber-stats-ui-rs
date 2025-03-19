use std::fmt::Display;
use std::panic::Location;

use fibermc_sdk::apis::*;
use fibermc_sdk::models::*;
use leptos::logging::{error, log};

use crate::requests::config::REQUEST_CONFIG;

#[track_caller]
fn log<T, E, Ctx: Display>(
    fn_name: &str,
    result: &Result<T, Error<E>>,
    ctx: Ctx,
) {
    log!("{}({}) [{}] complete", fn_name, ctx, Location::caller());
    if let Err(err) = result.as_ref() {
        error!("{}({}) [{}]: {}", fn_name, ctx, Location::caller(), err);
    }
}

pub async fn get_stats(mod_id: String) -> Option<ModStatsResponse> {
    let id_str = mod_id.as_str();
    let result =
        mods_api::api_v10_mods_id_stats_get(&REQUEST_CONFIG, id_str).await;

    log("get_stats", &result, id_str);

    result.ok()
}

pub async fn get_mod(mod_id: String) -> Option<ModResponse> {
    let result =
        mods_api::api_v10_mods_id_get(&REQUEST_CONFIG, mod_id.as_str()).await;

    // there can be an errors at deserialization time that we are throwing away
    // here... There _must_ be a better way to propagate these errors through in
    // leptos...
    log("get_mod", &result, mod_id);

    result.ok()
}
