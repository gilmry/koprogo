//! Export the live OpenAPI spec to `docs/api/openapi.json`.
//!
//! Run with: `cargo run --bin export_openapi`
//!
//! Usage in the build pipeline:
//!   make openapi-export   # regenerates docs/api/openapi.json
//!   make types-sync       # also runs frontend openapi-typescript
//!
//! The JSON output is consumed by `frontend` via `npm run types:generate`.

use koprogo_api::infrastructure::openapi::ApiDoc;
use std::fs;
use std::path::PathBuf;
use utoipa::OpenApi;

fn main() -> std::io::Result<()> {
    let spec = ApiDoc::openapi();
    let json = spec
        .to_pretty_json()
        .expect("OpenAPI spec must serialize to JSON");

    // Resolve docs/api/openapi.json relative to the repo root (one level above backend/)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out = PathBuf::from(manifest_dir)
        .join("..")
        .join("docs")
        .join("api")
        .join("openapi.json");

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, &json)?;

    println!("✅ OpenAPI spec written to {}", out.display());
    println!("   Routes: {}", spec.paths.paths.len());
    if let Some(components) = &spec.components {
        println!("   Schemas: {}", components.schemas.len());
    }
    Ok(())
}
