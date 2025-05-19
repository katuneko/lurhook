#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Start function for the WebAssembly build.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    game_core::run().map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start() {}
