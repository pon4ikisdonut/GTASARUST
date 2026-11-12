use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Operating systems supported by GTASARUST (64-bit x86_64 only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostOs {
    WindowsX64,
    LinuxX64,
    OtherX64,
}

/// Runtime host metadata captured during engine startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub os: HostOs,
    pub os_name: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub data_dir: PathBuf,
    pub save_dir: PathBuf,
}

impl HostInfo {
    /// Detect current host platform and standard application directories.
    pub fn detect() -> Self {
        let os = if cfg!(target_os = "windows") {
            HostOs::WindowsX64
        } else if cfg!(target_os = "linux") {
            HostOs::LinuxX64
        } else {
            HostOs::OtherX64
        };

        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let (data_dir, save_dir) =
            if let Some(proj) = directories::ProjectDirs::from("org", "gtasarust", "GTASARUST") {
                (
                    proj.data_local_dir().to_path_buf(),
                    proj.data_local_dir().join("saves"),
                )
            } else {
                (PathBuf::from("runtime"), PathBuf::from("saves"))
            };

        Self {
            os,
            os_name: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores,
            data_dir,
            save_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_arch_is_x86_64() {
        let info = HostInfo::detect();
        assert_eq!(info.arch, "x86_64");
        assert!(info.cpu_cores >= 1);
    }
}
