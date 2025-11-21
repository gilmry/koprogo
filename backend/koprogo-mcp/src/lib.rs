// KoproGo MCP - Model Context Protocol implementation
// Decentralized AI ecosystem for property management

pub mod core;
pub mod ports;
pub mod adapters;

// Re-export commonly used types
pub use core::*;
pub use ports::*;
pub use adapters::*;

/// MCP version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// MCP API version
pub const API_VERSION: &str = "v1";
