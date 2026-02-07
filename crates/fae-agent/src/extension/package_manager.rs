//! Package management system for extensions.

use super::{Extension, ExtensionMetadata};
use crate::error::{FaeAgentError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// An extension package with metadata and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPackage {
    /// Package metadata.
    pub metadata: ExtensionMetadata,
    /// Path to the extension.
    pub path: PathBuf,
    /// Configuration key-value pairs.
    pub config: HashMap<String, serde_json::Value>,
    /// Whether this extension is enabled.
    pub enabled: bool,
}

impl ExtensionPackage {
    /// Creates a new extension package.
    pub fn new(metadata: ExtensionMetadata, path: PathBuf) -> Self {
        Self {
            metadata,
            path,
            config: HashMap::new(),
            enabled: false,
        }
    }
}

/// Package manager for installing and managing extensions.
pub struct PackageManager {
    /// Directory where extensions are installed.
    extensions_dir: PathBuf,
    /// Installed packages.
    packages: HashMap<String, ExtensionPackage>,
}

impl PackageManager {
    /// Creates a new package manager with the given extensions directory.
    pub fn new(extensions_dir: PathBuf) -> Self {
        Self {
            extensions_dir,
            packages: HashMap::new(),
        }
    }

    /// Loads the package list from disk.
    pub fn load(&mut self) -> Result<()> {
        let manifest_path = self.extensions_dir.join("extensions.json");
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)?;
            let packages: Vec<ExtensionPackage> = serde_json::from_str(&content)?;
            for pkg in packages {
                self.packages.insert(pkg.metadata.name.clone(), pkg);
            }
        }
        Ok(())
    }

    /// Saves the package list to disk.
    fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.extensions_dir)?;
        let manifest_path = self.extensions_dir.join("extensions.json");
        let packages: Vec<_> = self.packages.values().cloned().collect();
        let content = serde_json::to_string_pretty(&packages)?;
        fs::write(manifest_path, content)?;
        Ok(())
    }

    /// Installs an extension from the given path.
    ///
    /// The path should point to an extension directory or package file.
    pub fn install(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(FaeAgentError::Extension(format!(
                "extension path does not exist: {}",
                path.display()
            )));
        }

        let metadata = ExtensionMetadata::new(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            "0.1.0".to_string(),
            "User-installed extension".to_string(),
            "Unknown".to_string(),
        );

        let pkg = ExtensionPackage::new(metadata.clone(), path.to_path_buf());

        if self.packages.contains_key(&metadata.name) {
            return Err(FaeAgentError::Extension(format!(
                "extension '{}' is already installed",
                metadata.name
            )));
        }

        self.packages.insert(metadata.name, pkg);
        self.save()?;
        Ok(())
    }

    /// Uninstalls an extension by name.
    pub fn uninstall(&mut self, name: &str) -> Result<()> {
        self.packages
            .remove(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("extension '{}' not found", name)))?;
        self.save()?;
        Ok(())
    }

    /// Lists all installed packages.
    pub fn list(&self) -> Vec<&ExtensionPackage> {
        self.packages.values().collect()
    }

    /// Enables an extension by name.
    pub fn enable(&mut self, name: &str) -> Result<()> {
        let pkg = self
            .packages
            .get_mut(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("extension '{}' not found", name)))?;
        pkg.enabled = true;
        self.save()?;
        Ok(())
    }

    /// Disables an extension by name.
    pub fn disable(&mut self, name: &str) -> Result<()> {
        let pkg = self
            .packages
            .get_mut(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("extension '{}' not found", name)))?;
        pkg.enabled = false;
        self.save()?;
        Ok(())
    }

    /// Gets the configuration for an extension.
    pub fn get_config(&self, name: &str) -> Option<&HashMap<String, serde_json::Value>> {
        self.packages.get(name).map(|pkg| &pkg.config)
    }

    /// Sets a configuration value for an extension.
    pub fn set_config(&mut self, name: &str, key: String, value: serde_json::Value) -> Result<()> {
        let pkg = self
            .packages
            .get_mut(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("extension '{}' not found", name)))?;
        pkg.config.insert(key, value);
        self.save()?;
        Ok(())
    }

    /// Loads a package by name, returning a boxed extension.
    ///
    /// This is a placeholder that returns an error - actual loading
    /// would require dynamic library loading or WASM runtime.
    pub fn load_package(&self, name: &str) -> Result<Box<dyn Extension>> {
        let _pkg = self
            .packages
            .get(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("extension '{}' not found", name)))?;

        Err(FaeAgentError::Extension(
            "Extension loading not yet implemented (requires WASM runtime)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn package_manager_new() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let manager = PackageManager::new(temp.path().to_path_buf());
        assert_eq!(manager.list().len(), 0);
    }

    #[test]
    fn install_extension() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());
        let result = manager.install(&ext_dir);
        assert!(result.is_ok());
        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn install_duplicate_fails() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());
        assert!(manager.install(&ext_dir).is_ok());
        let result = manager.install(&ext_dir);
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("already installed"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn uninstall_extension() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());
        assert!(manager.install(&ext_dir).is_ok());
        assert!(manager.uninstall("test-extension").is_ok());
        assert_eq!(manager.list().len(), 0);
    }

    #[test]
    fn enable_disable_extension() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());
        assert!(manager.install(&ext_dir).is_ok());
        assert!(manager.enable("test-extension").is_ok());

        let pkg = manager.packages.get("test-extension");
        assert!(pkg.is_some());
        let pkg = pkg.unwrap_or_else(|| unreachable!());
        assert!(pkg.enabled);

        assert!(manager.disable("test-extension").is_ok());
        let pkg = manager.packages.get("test-extension");
        assert!(pkg.is_some());
        let pkg = pkg.unwrap_or_else(|| unreachable!());
        assert!(!pkg.enabled);
    }

    #[test]
    fn set_config() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());
        assert!(manager.install(&ext_dir).is_ok());
        let result = manager.set_config(
            "test-extension",
            "key".to_string(),
            serde_json::json!("value"),
        );
        assert!(result.is_ok());

        let config = manager.get_config("test-extension");
        assert!(config.is_some());
        let config = config.unwrap_or_else(|| unreachable!());
        assert_eq!(config.get("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn save_and_load() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());
        assert!(manager.install(&ext_dir).is_ok());
        assert!(manager.save().is_ok());

        let mut manager2 = PackageManager::new(temp.path().to_path_buf());
        assert!(manager2.load().is_ok());
        assert_eq!(manager2.list().len(), 1);
    }
}
