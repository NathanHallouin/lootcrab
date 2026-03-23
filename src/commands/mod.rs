//! # Module des commandes
//!
//! ## Concept Rust : Modules
//! Les modules permettent d'organiser le code de manière hiérarchique.
//! `mod.rs` est le point d'entrée d'un module-dossier.
//!
//! ## Concept : pub (visibilité)
//! - `pub mod` : Le module est accessible depuis l'extérieur
//! - `mod` seul : Le module est privé à ce fichier

pub mod dev;
pub mod games;
pub mod general;
pub mod timer;
