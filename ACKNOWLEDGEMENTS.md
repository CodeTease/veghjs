# Acknowledgements and Technical Notes for VeghJS

This document serves to acknowledge contributions and transparently communicate technical implementation choices and experimental features utilized by the VeghJS project.

## 1. Experimental Features Notice (Node.js)

VeghJS leverages WebAssembly (WASM) ES Module integration for high-performance data processing in Node.js environments. Since this feature is still considered experimental by Node.js, users might encounter the following warning upon execution:

> `(node:XXXXX) ExperimentalWarning: Importing WebAssembly module instances is an experimental feature and might change at any time`

**Resolution**

This warning **does not affect the stability or correctness** of VeghJS's core functionality.

If you need to suppress this warning in non-interactive environments (such as CI/CD pipelines), you can launch your Node.js application using the official flag:
```bash
node --no-warnings your_app.js
```

## 2. Dependencies

VeghJS is built upon the incredible work of the following pure Rust crates, ensuring no external C dependencies are required for compilation:

* **ruzstd:** Pure Rust implementation of the Zstandard compression algorithm.
* **sha2:** Pure Rust implementation for SHA-256 integrity checks.
* **tar:** For parsing the Tar archive structure.