#![feature(iter_intersperse)]

pub mod app;
mod my_uuid;
mod param_helper;
mod requests;
mod stats_page;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::*;
    logging::log!("Hydrating app...");

    leptos::mount::hydrate_body(App);
}
