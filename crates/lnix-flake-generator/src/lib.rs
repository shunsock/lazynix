//! Nix flake generator for LazyNix
//!
//! Reads `lazynix.yaml` into the [`lnix_domain::Config`] AST. Rendering
//! moved to [`lnix_domain::render_flake`] (pure domain service); the
//! re-export below keeps this crate's public API stable until the crate
//! is dismantled.

mod error;
mod parser;

pub use error::{FlakeGeneratorError, Result};
pub use lnix_domain::render_flake;
pub use parser::LazyNixParser;
