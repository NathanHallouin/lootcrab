//! # Command Modules
//!
//! ## Rust concepts: Modules and visibility
//! - `pub mod` makes a module accessible from parent modules
//! - `mod` (without `pub`) keeps it private
//! - `mod.rs` is the entry point for a directory-based module

pub mod dev;
pub mod games;
pub mod general;
pub mod timer;
