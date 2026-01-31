mod api;
mod app;
mod components;
mod sound;
mod state;
mod utils;

use app::App;
use wasm_bindgen::prelude::*;
use yew::Renderer;

#[wasm_bindgen(start)]
pub fn run() {
    console_error_panic_hook::set_once();
    let root = gloo::utils::document()
        .get_element_by_id("app")
        .expect("missing #app");
    Renderer::<App>::with_root(root).render();
}
