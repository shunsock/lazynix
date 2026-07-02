//! Port for checking dotenv file existence.

/// Checks whether a dotenv file referenced by the config exists.
///
/// Only presence is checked — contents are read by `nix develop`
/// itself, not by lazynix.
pub trait EnvFilePresenceChecker {
    /// `path` is the value as written in `lazynix.yaml`: absolute, or
    /// relative to the config directory the implementation owns.
    fn exists(&self, path: &str) -> bool;
}
