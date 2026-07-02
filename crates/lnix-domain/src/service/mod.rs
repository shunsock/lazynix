//! Pure domain services: transformations with no I/O.
//!
//! Each submodule turns domain values into other domain values or
//! display-ready strings. Anything that touches the filesystem, a
//! subprocess, or stdout belongs behind [`crate::interface`] instead.

pub mod flake;
pub mod lint;
pub mod task;
