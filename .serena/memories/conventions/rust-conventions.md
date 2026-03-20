# Rust Conventions

- **Shared state**: `Arc<Mutex<T>>` / `Arc<RwLock<T>>`
- **Errors**: `thiserror` for per-crate error enums
- **Logging**: `tracing` crate — never `println!` in library code
- **Tests**: inline `#[cfg(test)]` blocks within the same file
- **Async**: Tokio runtime (edition 2021)
- **Aggregate roots**: own consistency boundaries; cross-context communication via domain events
- **Cross-device types**: shared between napi/wasm must live in `rlmx-kernel`
- **LifeDomain**: closed enum with 12 fixed variants (Finance, Health, Legal, Career, Education, Home, Shopping, Travel, Social, Government, Automotive, Pet)
