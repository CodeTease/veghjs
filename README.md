# 🥬 VeghJS

**VeghJS** is the high-performance WebAssembly (WASM) binding for the Vegh snapshot core, providing **near-native speed** for reading and verifying `.vegh` files directly in the browser or Node.js.

The core logic is implemented in **Pure Rust (ruzstd, sha2)** and compiled to WASM, ensuring reliability and bypassing the need for C compilers. This version is locked to the stable Vegh core logic (**v0.3.0**), using **Format Version 2** (with backward compatibility for FV 1).

## Features

* **WASM Speed:** Decompress Zstd-compressed Tar archives instantly via `ruzstd`.
* **FV 2 Integrity:** Uses BLAKE3 for lightning-fast hash verification (replaces SHA256).
* **Zero Dependencies (JS):** Once the WASM is loaded, the module runs independently.
* **Metadata Read:** Quickly read snapshot metadata (Author, Comment, Format Version) without full decompression.
* **Caching Schema:** Full support for Vegh's `FileCacheEntry` and `VeghCache` structures, allowing interoperability with CLI cache files.
* **Worker Offloading:** Includes a robust `worker.js` to run heavy operations (Hashing, Metadata) off the main thread, keeping your UI responsive.

## Usage

VeghJS performs optimally in modern JavaScript/TypeScript environments that support ES Modules.

1. **Basic Inspection (Node.js/Browser)**
```javascript
import * as vegh from 'veghjs';

await vegh.default(); // Initialize WASM

// Check capabilities
const info = vegh.get_library_info();
console.log(`VeghJS v${info.version} (Core: ${info.core_version})`);
// > VeghJS v0.3.0 (Core: 0.3.0)

// ... Read file to uint8Array ...
const metadata = vegh.get_metadata(uint8Array);
console.log(`Author: ${metadata.author}, Format: v${metadata.format_version}`);
```

2. **Caching Support (FV 2)**

VeghJS allows you to interact with the Vegh CLI caching logic. This is useful if you are building a tool that needs to verify if a file has changed compared to a cache index.
```javascript
// Example: Creating a cache check
const cacheObj = vegh.create_empty_cache();
// Assume we populated cacheObj.files['my-file.txt'] from a previous run

const isHit = vegh.check_cache_hit(
    cacheObj, 
    'my-file.txt', 
    BigInt(1024),      // Size
    BigInt(1678900000) // Modified Time
);

if (isHit) {
    console.log("File is unchanged (Cache Hit)");
}
```
3. **Worker Offloading (Recommended for UI)**

For the best user experience, use the included `worker.js` to process large snapshots without blocking the main thread.
```javascript
const worker = new Worker('path/to/node_modules/veghjs/worker.js', { type: 'module' });

worker.onmessage = (e) => {
    if (e.data.type === 'RESULT_INTEGRITY') {
        console.log('Hash:', e.data.payload);
    }
};

// Send a large file to be hashed (Streamed via BLAKE3)
worker.postMessage({
    command: 'CHECK_INTEGRITY_STREAM',
    payload: { file: myFileObj }
});
```

## Development & Building

If you wish to contribute or rebuild the WASM module:

1. **Clone the Repository:**
```bash
git clone https://github.com/CodeTease/veghjs
cd veghjs
```

2. **Install Dependencies:**
```bash
npm ci
# Or npm install (npm i) if you want
```

3. **Build the WASM Module:**
```bash
npm run build
```
(This command executes `wasm-pack build --target bundler --out-dir pkg --release`)

## Acknowledgements

Take a look at [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md) for technical notes about VeghJS.

## License

This project is under the **MIT License**.
