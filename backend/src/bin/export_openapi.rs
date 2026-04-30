//! Export the live OpenAPI spec to stdout (JSON).
//!
//! Run with: `cargo run --bin export_openapi > docs/api/openapi.json`
//!
//! The JSON payload is written to stdout so that the caller (Makefile or
//! shell pipeline) can redirect it to whichever file the host wants. This
//! is required because the binary runs inside the docker container where
//! the host repo root is not mounted — stdout is the universal channel.
//!
//! Progress messages (route count, schema count) are written to stderr so
//! they never corrupt the JSON pipe.
//!
//! Usage in the build pipeline:
//!   make openapi-export   # writes docs/api/openapi.json on the host
//!   make openapi-check    # also fails if the file is stale
//!   make types-sync       # runs openapi-export then openapi-typescript

use koprogo_api::infrastructure::openapi::ApiDoc;
use std::io::Write;
use utoipa::OpenApi;

fn main() -> std::io::Result<()> {
    let spec = ApiDoc::openapi();
    let json = spec
        .to_pretty_json()
        .expect("OpenAPI spec must serialize to JSON");

    // JSON to stdout (consumed by the host via redirection)
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(json.as_bytes())?;
    handle.write_all(b"\n")?;

    // Progress on stderr (never touches the pipe)
    let schemas = spec
        .components
        .as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    eprintln!(
        "✅ OpenAPI spec exported — {} routes, {} schemas",
        spec.paths.paths.len(),
        schemas
    );
    Ok(())
}
