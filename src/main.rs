use leptos::*;
use wasm_bindgen::prelude::*;

mod components;
mod util;

fn main() {
    // _ = console_log::init_with_level(log::Level::Debug);
    leptos::mount_to_body(|| view! { <components::app::App /> })
}
