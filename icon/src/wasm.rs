// WASM bindings for browser usage
use crate::engine::IconSearchEngine;
use crate::index::IconIndex;
use crate::search::SearchResult;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
pub struct WasmSearchResult {
    pub name: String,
    pub pack: String,
    pub score: f32,
    pub match_type: String,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmIconSearch {
    engine: IconSearchEngine,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmIconSearch {
    #[wasm_bindgen(constructor)]
    pub fn new(index_data: &[u8]) -> Result<WasmIconSearch, JsValue> {
        // Deserialize index from bytes
        let index: IconIndex = rkyv::from_bytes(index_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize index: {}", e)))?;

        let engine = IconSearchEngine::from_index(index)
            .map_err(|e| JsValue::from_str(&format!("Failed to create engine: {}", e)))?;

        Ok(WasmIconSearch { engine })
    }

    #[wasm_bindgen]
    pub fn search(&self, query: &str, limit: usize) -> Result<JsValue, JsValue> {
        let results = self.engine.search(query, limit);

        let wasm_results: Vec<WasmSearchResult> = results
            .into_iter()
            .map(|r| WasmSearchResult {
                name: r.icon.name,
                pack: r.icon.pack,
                score: r.score,
                match_type: format!("{:?}", r.match_type),
            })
            .collect();

        serde_wasm_bindgen::to_value(&wasm_results)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen]
    pub fn total_icons(&self) -> usize {
        self.engine.total_icons()
    }

    #[wasm_bindgen]
    pub fn clear_cache(&mut self) {
        self.engine.clear_cache();
    }
}
