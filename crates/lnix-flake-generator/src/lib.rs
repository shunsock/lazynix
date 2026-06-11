//! Nix flake generator for LazyNix
//!
//! Reads `lazynix.yaml` into the [`lnix_core::Config`] AST and renders
//! it into a `flake.nix` string.

mod error;
mod generator;
mod parser;

// Public API
pub use error::{FlakeGeneratorError, Result};
pub use generator::render_flake;
pub use parser::LazyNixParser;
