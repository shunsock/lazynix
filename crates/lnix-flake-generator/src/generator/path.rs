//! Path resolution for files referenced from the dev shell.
//!
//! Paths are resolved at shell-hook runtime, so relative paths expand
//! against `$PWD` and `~/` expands against `$HOME` — not at generation
//! time.

fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/')
}

/// Rewrites a user-supplied path into a shell expression evaluated when
/// the dev shell starts.
///
/// - `~/foo`      → `$HOME/foo`
/// - `/etc/foo`   → `/etc/foo` (absolute, unchanged)
/// - `./foo`, `foo` → `$PWD/foo`
pub(super) fn resolve_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("~/") {
        format!("$HOME/{}", stripped)
    } else if is_absolute_path(path) {
        path.to_string()
    } else {
        format!("$PWD/{}", path.trim_start_matches("./"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tilde_to_home() {
        // Arrange / Act / Assert
        assert_eq!(resolve_path("~/.aliases"), "$HOME/.aliases");
    }

    #[test]
    fn keeps_absolute_path_unchanged() {
        // Arrange / Act / Assert
        assert_eq!(resolve_path("/home/user/.env"), "/home/user/.env");
    }

    #[test]
    fn anchors_relative_path_to_pwd() {
        // Arrange / Act / Assert
        assert_eq!(resolve_path(".env"), "$PWD/.env");
        assert_eq!(resolve_path("./config/.env"), "$PWD/config/.env");
        assert_eq!(resolve_path("../.env"), "$PWD/../.env");
    }
}
