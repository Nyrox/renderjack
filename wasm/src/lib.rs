mod utils;

use wasm_bindgen::prelude::*;
use renderjack;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct Vector3f {
    x: f32,
    y: f32,
    z: f32,
}

#[wasm_bindgen]
pub struct Tri3(Vector3f, Vector3f, Vector3f);

#[wasm_bindgen]
pub fn create_tri3(a: Vector3f) -> Tri3 {
    Tri3(a, a, a)
}

#[wasm_bindgen]
pub fn fullscreen_raster(width: usize, height: usize) -> Vec<u8> {

    vec![]
}