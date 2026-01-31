use wasm_bindgen::JsCast;
use web_sys::HtmlAudioElement;

const SOUNDS: &[&str] = &["capture", "castle", "check", "checkmate", "move", "start"];

/// Preload all sounds (call on first user gesture so later play() starts immediately).
pub fn preload() {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };
    for name in SOUNDS {
        if let Ok(el) = document.create_element("audio") {
            if let Ok(audio) = el.dyn_into::<HtmlAudioElement>() {
                let _ = audio.set_src(&format!("/assets/{}.mp3", name));
                let _ = audio.set_preload("auto");
            }
        }
    }
}

pub fn play(name: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };
    let audio = match document.create_element("audio") {
        Ok(el) => el,
        Err(_) => return,
    };
    let audio: HtmlAudioElement = match audio.dyn_into() {
        Ok(a) => a,
        Err(_) => return,
    };
    let _ = audio.set_src(&format!("/assets/{}.mp3", name));
    let _ = audio.play();
}
