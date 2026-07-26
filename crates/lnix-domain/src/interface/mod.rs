//! Ports: the interfaces the domain requires from the outside world.
//!
//! The domain defines *what* it needs; infrastructure adapters decide
//! *how* (dependency inversion). Ports are grouped by the kind of
//! outside world they touch:
//!
//! - [`persistence`] — the project's own files (repositories).
//! - [`gateway`] — external processes (`nix`, `nix-versions`).
//!
//! Application-level ports whose vocabulary belongs to the application
//! layer (e.g. `ReporterPort` in `lnix-app`) live next to their
//! use-cases rather than here.

pub mod gateway;
pub mod persistence;
