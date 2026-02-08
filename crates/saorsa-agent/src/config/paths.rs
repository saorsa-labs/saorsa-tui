//! Configuration directory paths.

use std::path::PathBuf;

use crate::error::{Result, SaorsaAgentError};

/// Returns the Saorsa configuration directory (`~/.saorsa/`).
///
/// # Errors
///
/// Returns [`SaorsaAgentError::HomeDirectory`] if the home directory cannot be
/// determined.
pub fn saorsa_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(SaorsaAgentError::HomeDirectory)?;
    Ok(home.join(".saorsa"))
}

/// Returns the Saorsa configuration directory, creating it if it does not
/// exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::HomeDirectory`] if the home directory cannot be
/// determined, or [`SaorsaAgentError::ConfigIo`] if the directory cannot be
/// created.
pub fn ensure_config_dir() -> Result<PathBuf> {
    let dir = saorsa_config_dir()?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(SaorsaAgentError::ConfigIo)?;
    }
    Ok(dir)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn saorsa_config_dir_ends_with_dot_saorsa() {
        let dir = saorsa_config_dir().unwrap();
        assert!(dir.ends_with(".saorsa"));
    }

    #[test]
    fn ensure_config_dir_creates_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let fake_home = tmp.path().join("home");
        std::fs::create_dir_all(&fake_home).unwrap();

        // We cannot override dirs::home_dir easily, so instead test the
        // real config dir path is returned and the function succeeds.
        let dir = ensure_config_dir().unwrap();
        assert!(dir.exists());
        assert!(dir.ends_with(".saorsa"));
    }

    #[test]
    fn ensure_config_dir_is_idempotent() {
        let dir1 = ensure_config_dir().unwrap();
        let dir2 = ensure_config_dir().unwrap();
        assert_eq!(dir1, dir2);
    }
}
