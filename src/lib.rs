#![feature(iter_intersperse)]

use cfg_if::cfg_if;
use leptos::mount::mount_to_body;

pub mod app;
mod my_uuid;
mod param_helper;
mod requests;
mod stats_page;

cfg_if! {
if #[cfg(feature = "hydrate")] {

  use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn hydrate() {
      use app::*;
      use leptos::*;

      console_error_panic_hook::set_once();

      mount_to_body(move || {
          view! { <App/> }
      });
    }
}
}
