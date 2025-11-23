// CodeTease - VeghJS Core (Rust/WASM)
// Validated for OOM prevention & Thread safety

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::io::{Cursor, Read};
use sha2::{Digest, Sha256};
use ruzstd::StreamingDecoder; 
use tar::Archive;

// --- CONSTANTS ---
const SNAPSHOT_FORMAT_VERSION: &str = "1";

#[derive(Serialize, Deserialize, Debug)]
struct VeghMetadata {
    author: String,
    timestamp: i64,
    timestamp_human: Option<String>, 
    comment: String,
    tool_version: String, 
}

#[derive(Serialize)]
struct SnapEntry {
    path: String,
    size: u64,
    is_file: bool,
}

#[derive(Serialize)]
struct LibraryInfo {
    version: String,
    supported_format: String,
    engine: String,
}

// --- WASM EXPORTS ---

#[wasm_bindgen]
pub fn get_library_info() -> Result<JsValue, JsValue> {
    let info = LibraryInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        supported_format: SNAPSHOT_FORMAT_VERSION.to_string(),
        engine: "Rust/WASM (ruzstd)".to_string(),
    };
    Ok(serde_wasm_bindgen::to_value(&info)?)
}

// --- NEW FEATURE: STREAMING HASHER ---
// Prevents OOM by processing chunks instead of loading the full file.
// IMPORTANT: JS must call .free() on this object when done!
#[wasm_bindgen]
pub struct VeghStreamingHasher {
    hasher: Sha256,
}

#[wasm_bindgen]
impl VeghStreamingHasher {
    #[wasm_bindgen(constructor)]
    pub fn new() -> VeghStreamingHasher {
        VeghStreamingHasher {
            hasher: Sha256::new(),
        }
    }

    pub fn update(&mut self, chunk: &[u8]) {
        self.hasher.update(chunk);
    }

    // Consumes the hasher and returns the hex string
    pub fn finalize(self) -> String {
        hex::encode(self.hasher.finalize())
    }
}

// --- STANDARD FUNCTIONS (Worker Optimized) ---

#[wasm_bindgen]
pub fn get_metadata(data: &[u8]) -> Result<JsValue, JsValue> {
    let cursor = Cursor::new(data);
    let decoder = StreamingDecoder::new(cursor).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut archive = Archive::new(decoder);

    for file in archive.entries().map_err(|e| JsValue::from_str(&e.to_string()))? {
        let mut file = file.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let path = file.path().map_err(|e| JsValue::from_str(&e.to_string()))?.to_string_lossy().to_string();

        if path == ".vegh.json" {
            let mut s = String::new();
            file.read_to_string(&mut s).map_err(|e| JsValue::from_str(&e.to_string()))?;
            
            let meta: VeghMetadata = serde_json::from_str(&s).map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(serde_wasm_bindgen::to_value(&meta)?);
        }
    }

    Err(JsValue::from_str("Metadata file (.vegh.json) not found in snapshot"))
}

#[wasm_bindgen]
pub fn list_files(data: &[u8]) -> Result<JsValue, JsValue> {
    let cursor = Cursor::new(data);
    let decoder = StreamingDecoder::new(cursor).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut archive = Archive::new(decoder);
    
    let mut entries = Vec::new();

    for file in archive.entries().map_err(|e| JsValue::from_str(&e.to_string()))? {
        let file = file.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let path = file.path().map_err(|e| JsValue::from_str(&e.to_string()))?.to_string_lossy().to_string();
        let size = file.size();

        if path != ".vegh.json" {
            entries.push(SnapEntry {
                path,
                size,
                is_file: true,
            });
        }
    }

    Ok(serde_wasm_bindgen::to_value(&entries)?)
}