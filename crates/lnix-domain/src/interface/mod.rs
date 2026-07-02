//! Ports: the interfaces the domain requires from the outside world.
//!
//! The domain defines *what* it needs; infrastructure adapters decide
//! *how* (dependency inversion). Ports are grouped by the kind of
//! outside world they touch:
//!
//! - [`persistence`] — the project's own files (repositories).
//! - [`gateway`] — external processes (`nix`, `nix-versions`).
//! - [`output`] — the user-facing display sink.

pub mod gateway;
pub mod output;
pub mod persistence;
