# 🥬 VeghJS

**VeghJS** is the high-performance WebAssembly (WASM) binding for the Vegh snapshot core, providing **near-native speed** for reading and verifying .snap files directly in the browser or Node.js.

The core logic is implemented in **Pure Rust (ruzstd, sha2)** and compiled to WASM, ensuring reliability and bypassing the need for C compilers. This version is intentionally locked to the stable Vegh core logic (v0.2.0), using Format Version 1.

## Features

* **WASM Speed:** Decompress Zstd-compressed Tar archives instantly on the client side.
* **Integrity Check:** Fast SHA256 verification of the entire snapshot file.
* **Zero Dependencies (JS):** Once the WASM is loaded, the module runs independently.
* **Metadata Read:** Quickly read snapshot metadata (Author, Comment, Format Version 1) without full decompression.
* **Format Compatibility:** Reads snapshots created by the original Vegh CLI and PyVegh.

## Usage

VeghJS performs optimally in modern JavaScript/TypeScript environments that support ES Modules.

1. **Node.js Example: Processing a Snapshot from Path**
```javascript
import { readFile } from 'fs/promises';
import * as vegh from 'veghjs'; // This assumes you run Node with ES Modules support

/**
 * Reads a .snap file from the file system, processes it using VeghJS (WASM), 
 * and prints its information.
 * @param {string} snapFilePath Path to the .snap file.
 */
async function inspectSnapshot(snapFilePath) {
    // 1. Initialize WASM module
    // This step is required in Node.js environments to load the WASM binary.
    await vegh.default();

    try {
        // Read file contents (returns a Node.js Buffer)
        const fileBuffer = await readFile(snapFilePath);
        
        // Convert Node.js Buffer to Uint8Array (required by WASM functions)
        const uint8Array = new Uint8Array(fileBuffer);
        
        // 2. Check Integrity
        const hash = vegh.check_integrity(uint8Array);
        console.log(`[SHA256 Integrity] ${hash}`);

        // 3. Get Metadata
        const metadata = vegh.get_metadata(uint8Array); 
        console.log('\n[Metadata]');
        console.log(`Author: ${metadata.author}`);
        console.log(`Comment: ${metadata.comment}`);

        // 4. List Contents
        const fileList = vegh.list_files(uint8Array);
        console.log(`\n[Contents (${fileList.length} files)]`);
        fileList.slice(0, 5).forEach(f => {
            console.log(`- ${f.path} (${(f.size / 1024).toFixed(1)} KB)`);
        });

    } catch (error) {
        console.error("An error occurred during snapshot processing:", error);
    }
}

// Example usage (replace with an actual path to a .snap file)
inspectSnapshot('path/to/your/backup.snap');
```

2. **Important Node.js Note: Initialization**

VeghJS exports an async default function (via `wasm-pack`) which must be called once before using any other functions:
```javascript
import * as vegh from 'veghjs';
await vegh.default(); // This loads the .wasm binary
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