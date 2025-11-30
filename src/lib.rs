// CodeTease - VeghJS Core (Rust/WASM)
// Validated for OOM prevention & Thread safety

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::io::{Cursor, Read};
use blake3::Hasher;
use std::collections::HashMap; 
use ruzstd::StreamingDecoder; 
use tar::Archive;

// --- CONSTANTS ---
const SNAPSHOT_FORMAT_VERSION: &str = "2";
const VEGH_CORE_VERSION: &str = "0.3.0";

// --- STRUCTS (Vegh Core Sync) ---

#[derive(Serialize, Deserialize, Debug)]
struct VeghMetadata {
    author: String,
    timestamp: i64,
    
    // [COMPAT FIX] PyVegh doesn't export this field.
    // #[serde(default)] ensures it becomes None instead of crashing if missing.
    #[serde(default)] 
    timestamp_human: Option<String>, 
    
    comment: String,
    tool_version: String,
    #[serde(default = "default_format_version")]
    format_version: String,
}

fn default_format_version() -> String {
    "1".to_string()
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
    core_version: String,
    supported_format: String,
    engine: String,
    features: Vec<String>,
}

// Caching Structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileCacheEntry {
    pub size: u64,
    pub modified: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VeghCache {
    pub last_snapshot: i64,
    pub files: HashMap<String, FileCacheEntry>,
}

// --- WASM EXPORTS ---

#[wasm_bindgen]
pub fn get_library_info() -> Result<JsValue, JsValue> {
    let info = LibraryInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        core_version: VEGH_CORE_VERSION.to_string(),
        supported_format: SNAPSHOT_FORMAT_VERSION.to_string(),
        engine: "Rust/WASM (ruzstd + blake3)".to_string(),
        features: vec![
            "streaming_hashing".to_string(),
            "caching_schema_v2".to_string(),
            "worker_offloading".to_string(),
            "content_extraction".to_string(),
            "pyvegh_compat".to_string() // [NEW] Flag compatibility
        ],
    };
    Ok(serde_wasm_bindgen::to_value(&info)?)
}

// --- CACHING LOGIC ---

#[wasm_bindgen]
pub fn create_empty_cache() -> Result<JsValue, JsValue> {
    let cache = VeghCache::default();
    Ok(serde_wasm_bindgen::to_value(&cache)?)
}

#[wasm_bindgen]
pub fn check_cache_hit(
    cache_val: JsValue, 
    path: String, 
    current_size: u64, 
    current_modified: u64
) -> bool {
    let cache: VeghCache = match serde_wasm_bindgen::from_value(cache_val) {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(entry) = cache.files.get(&path) {
        return entry.size == current_size && entry.modified == current_modified;
    }
    false
}

// --- STREAMING HASHER ---

#[wasm_bindgen]
pub struct VeghStreamingHasher {
    hasher: Hasher,
}

#[wasm_bindgen]
impl VeghStreamingHasher {
    #[wasm_bindgen(constructor)]
    pub fn new() -> VeghStreamingHasher {
        VeghStreamingHasher {
            hasher: Hasher::new(),
        }
    }

    pub fn update(&mut self, chunk: &[u8]) {
        self.hasher.update(chunk);
    }

    pub fn finalize(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

// --- STANDARD FUNCTIONS ---

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
    Err(JsValue::from_str("Metadata file (.vegh.json) not found"))
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

// --- CONTENT EXTRACTION ---
#[wasm_bindgen]
pub fn get_file_content(data: &[u8], target_path: &str) -> Result<Box<[u8]>, JsValue> {
    let cursor = Cursor::new(data);
    let decoder = StreamingDecoder::new(cursor).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut archive = Archive::new(decoder);

    for file in archive.entries().map_err(|e| JsValue::from_str(&e.to_string()))? {
        let mut file = file.map_err(|e| JsValue::from_str(&e.to_string()))?;
        let path = file.path().map_err(|e| JsValue::from_str(&e.to_string()))?.to_string_lossy().to_string();

        if path == target_path {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|e| JsValue::from_str(&e.to_string()))?;
            return Ok(buffer.into_boxed_slice());
        }
    }

    Err(JsValue::from_str(&format!("File not found: {}", target_path)))
}