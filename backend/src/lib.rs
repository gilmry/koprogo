//! KoproGo API - Property Management Platform
//!
//! This crate provides the backend API for KoproGo, a Belgian property management platform.

// Allow some clippy lints for cleaner code during active development
#![allow(clippy::too_many_arguments)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::useless_format)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_cast)]
pub mod application;
pub mod domain;
pub mod infrastructure;
