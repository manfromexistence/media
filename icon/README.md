# DX Icon Search

World's fastest icon search engine built with Rust.

## Run

cargo run --release --bin search_cli

## Features

- **FST-based prefix search** - <0.1ms search latency
- **Zero-copy rkyv metadata** - No deserialization overhead
- **Fuzzy matching** - Typo tolerance with Levenshtein distance
- **LZ4 compression** - Fast decompression for network transfer
- **WASM support** - Run in browser with near-native performance
- **Multi-strategy search** - Exact, prefix, and fuzzy matching
- **Smart caching** - LRU cache for repeated queries

## Architecture

```
┌─────────────────────────────────────────────────┐
│  TIER 1: FST Index (~1MB)                       │
│  - Finite State Transducer for prefix search   │
│  - O(k) lookup where k = query length          │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  TIER 2: rkyv Metadata (~2MB)                   │
│  - Zero-copy archived data                      │
│  - Direct memory access, no parsing             │
└─────────────────────────────────────────────────┘
```

## Usage

### Build Index

```bash
cargo run --bin build_index
```

### CLI Search

```bash
cargo run --bin search_cli
```

### WASM Build

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/dx_icon_search.wasm --out-dir pkg
```

## Performance

- **Index size**: ~3MB (compressed)
- **Load time**: <50ms
- **Search latency**: <0.1ms (cached), <1ms (uncached)
- **Memory usage**: ~5MB
- **Icons supported**: 100K+

## License

MIT
