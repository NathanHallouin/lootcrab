//! # Services
//!
//! ## Rust concept: Separation of concerns
//! Services encapsulate business logic (API calls, scheduling, persistence),
//! keeping it separate from Discord command handlers.

pub mod config;
pub mod free_games;
pub mod scheduler;
