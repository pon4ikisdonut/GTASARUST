use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VfsError {
    #[error("game data directory does not exist: '{0}'")]
    MissingRoot(PathBuf),
    #[error("path '{0}' is not a valid authorized GTA:SA root (missing expected file '{1}')")]
    InvalidInstallation(PathBuf, String),
    #[error("asset path escapes game root: '{0}'")]
    PathTraversal(String),
    #[error("relative path not found under '{root}': '{relative}'")]
    NotFound { root: PathBuf, relative: String },
    #[error("I/O error reading '{0}': {1}")]
    Io(PathBuf, #[source] std::io::Error),
}

/// Cross-platform case-insensitive filesystem root for an authorized GTA: San Andreas directory.
///
/// Original `gta.dat` and `.ide`/`.ipl` references mix Windows backslashes and inconsistent casing
/// (e.g., `DATA\MAPS\LA\LAwn.IDE` vs `data/maps/LA/LaWn.ide`). `GameDataRoot` normalizes path
/// separators and caches exact disk paths across both Windows and case-sensitive Linux filesystems.
#[derive(Debug)]
pub struct GameDataRoot {
    root: PathBuf,
    case_index: RwLock<HashMap<String, PathBuf>>,
}

impl GameDataRoot {
    /// Open and validate a GTA: San Andreas installation root directory.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, VfsError> {
        let root_path = root.as_ref().to_path_buf();
        if !root_path.is_dir() {
            return Err(VfsError::MissingRoot(root_path));
        }

        let vfs = Self {
            root: root_path,
            case_index: RwLock::new(HashMap::new()),
        };

        // Verify structural markers of an authorized GTA:SA PC installation without bypassing DRM.
        for required in ["data/gta.dat", "data/default.dat", "models/gta3.img"] {
            if vfs.resolve(required).is_err() {
                return Err(VfsError::InvalidInstallation(
                    vfs.root.clone(),
                    required.to_string(),
                ));
            }
        }

        Ok(vfs)
    }

    /// Open a root directory without requiring full `gta3.img` validation (useful for minimal test fixtures).
    pub fn open_unvalidated(root: impl AsRef<Path>) -> Result<Self, VfsError> {
        let root_path = root.as_ref().to_path_buf();
        if !root_path.is_dir() {
            return Err(VfsError::MissingRoot(root_path));
        }
        Ok(Self {
            root: root_path,
            case_index: RwLock::new(HashMap::new()),
        })
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    /// Normalize a Windows or POSIX relative path into lowercase forward-slash components,
    /// rejecting absolute paths or `..` traversal attempts.
    pub fn normalize_relative(relative: &str) -> Result<Vec<String>, VfsError> {
        let trimmed = relative.trim();
        if trimmed.starts_with('/') || trimmed.starts_with('\\') || trimmed.contains(':') {
            return Err(VfsError::PathTraversal(relative.to_string()));
        }

        let mut parts = Vec::new();
        for seg in trimmed.split(['/', '\\']) {
            if seg.is_empty() || seg == "." {
                continue;
            }
            if seg == ".." {
                return Err(VfsError::PathTraversal(relative.to_string()));
            }
            parts.push(seg.to_string());
        }
        Ok(parts)
    }

    /// Resolve a relative GTA:SA path (case-insensitively across segments) to a real path on disk.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf, VfsError> {
        let parts = Self::normalize_relative(relative)?;
        let cache_key = parts
            .iter()
            .map(|p| p.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("/");

        if let Some(cached) = self.case_index.read().get(&cache_key) {
            return Ok(cached.clone());
        }

        let mut current = self.root.clone();
        for part in &parts {
            let direct = current.join(part);
            if direct.exists() {
                current = direct;
                continue;
            }

            let target_lower = part.to_ascii_lowercase();
            let entries =
                std::fs::read_dir(&current).map_err(|e| VfsError::Io(current.clone(), e))?;
            let mut matched = None;
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.to_ascii_lowercase() == target_lower
                {
                    matched = Some(entry.path());
                    break;
                }
            }

            match matched {
                Some(found) => current = found,
                None => {
                    return Err(VfsError::NotFound {
                        root: self.root.clone(),
                        relative: relative.to_string(),
                    });
                }
            }
        }

        self.case_index.write().insert(cache_key, current.clone());
        Ok(current)
    }

    /// Read bytes for a case-insensitive relative path inside the game installation.
    pub fn read_bytes(&self, relative: &str) -> Result<Vec<u8>, VfsError> {
        let resolved = self.resolve(relative)?;
        std::fs::read(&resolved).map_err(|e| VfsError::Io(resolved, e))
    }

    /// Read UTF-8 / Windows-1252-lossy text for a configuration, IDE, or IPL file.
    pub fn read_text_lossy(&self, relative: &str) -> Result<String, VfsError> {
        let bytes = self.read_bytes(relative)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_mixed_case_and_backslashes() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("data").join("maps").join("LA");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("LaWn.ide"), b"# test").unwrap();

        let vfs = GameDataRoot::open_unvalidated(dir.path()).unwrap();
        let resolved = vfs.resolve(r"DATA\MAPS\LA\LAWN.IDE").unwrap();
        assert!(resolved.exists());
        assert_eq!(
            vfs.read_text_lossy(r"data\maps\la\lawn.ide").unwrap(),
            "# test"
        );
    }

    #[test]
    fn rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let vfs = GameDataRoot::open_unvalidated(dir.path()).unwrap();
        assert!(matches!(
            vfs.resolve("../secret.txt"),
            Err(VfsError::PathTraversal(_))
        ));
    }
}
