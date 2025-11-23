// CodeTease - VeghJS Worker
// Handles WASM operations in a background thread to keep UI responsive.

import init, { VeghStreamingHasher, get_metadata, list_files } from "./pkg/vegh_js.js";

let isReady = false;

// Initialize WASM immediately upon worker creation
(async () => {
    try {
        await init();
        isReady = true;
        postMessage({ type: 'READY' });
    } catch (err) {
        console.error("VeghJS Worker Init Failed:", err);
        postMessage({ type: 'ERROR', payload: `WASM Init Failed: ${err}` });
    }
})();

self.onmessage = async (e) => {
    if (!isReady) {
        postMessage({ type: 'ERROR', payload: 'Core not ready yet. Please wait.' });
        return;
    }

    const { command, file, chunk_size = 5 * 1024 * 1024 } = e.data; // 5MB chunks

    try {
        switch (command) {
            case 'CHECK_INTEGRITY_STREAM':
                if (!file) throw new Error("File is required");
                await handleIntegrityStream(file, chunk_size);
                break;

            case 'GET_METADATA':
                if (!file) throw new Error("File is required");
                await handleMetadata(file);
                break;

            case 'LIST_FILES':
                if (!file) throw new Error("File is required");
                await handleListFiles(file);
                break;

            default:
                postMessage({ type: 'ERROR', payload: `Unknown command: ${command}` });
        }
    } catch (err) {
        postMessage({ type: 'ERROR', payload: err.message || String(err) });
    }
};

/**
 * DOUBLE-CHECKED: Safe Memory Management
 * Uses VeghStreamingHasher to process file in chunks.
 * Explicitly frees Rust memory to prevent leaks.
 */
async function handleIntegrityStream(file, chunkSize) {
    const totalBytes = file.size;
    let loadedBytes = 0;
    
    // Create Rust instance
    const hasher = new VeghStreamingHasher();
    
    // Use standard Web Stream API
    const reader = file.stream().getReader();

    try {
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;

            // 'value' is Uint8Array, passed directly to WASM memory
            hasher.update(value);
            
            loadedBytes += value.length;
            
            // Send progress update
            const progress = (loadedBytes / totalBytes) * 100;
            postMessage({ 
                type: 'PROGRESS', 
                payload: { task: 'integrity', progress } 
            });
        }

        // Finalize and get hash string
        const hash = hasher.finalize();
        postMessage({ type: 'RESULT_INTEGRITY', payload: hash });
        
    } catch (err) {
        throw err;
    } finally {
        // CRITICAL: Explicitly free Rust memory
        // Even though finalize() consumes self in Rust, the JS wrapper might stick around.
        // calling .free() is the safest bet in wasm-bindgen.
        if (hasher && hasher.free) {
            hasher.free();
        }
    }
}

async function handleMetadata(file) {
    // Note: Still loads to memory, but in Worker thread so UI won't freeze.
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);
    const meta = get_metadata(uint8Array);
    postMessage({ type: 'RESULT_METADATA', payload: meta });
}

async function handleListFiles(file) {
    const arrayBuffer = await file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);
    const list = list_files(uint8Array);
    postMessage({ type: 'RESULT_FILES', payload: list });
}