use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
const PORTABLE_MARKER: &str = "portable.sqdesk";
#[cfg(target_os = "windows")]
const INSTALLER_MARKER: &str = "uninstall.exe";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataDirMode {
    Default,
    EnvOverride,
    Portable { exe_dir: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirResolution {
    pub data_dir: PathBuf,
    pub default_data_dir: PathBuf,
    pub mode: DataDirMode,
    portable_data_dir: Option<PathBuf>,
}

impl DataDirResolution {
    pub fn uses_custom_data_dir(&self) -> bool {
        matches!(self.mode, DataDirMode::EnvOverride | DataDirMode::Portable { .. })
    }
}

pub fn resolve_data_dir_with_mode(default_app_data_dir: PathBuf) -> DataDirResolution {
    let env_data_dir = std::env::var_os("SQDESK_DATA_DIR").filter(|value| !value.is_empty()).map(PathBuf::from);

    #[cfg(target_os = "windows")]
    let exe_dir = current_exe_dir();
    #[cfg(not(target_os = "windows"))]
    let exe_dir = None;

    let portable_marker_exists = exe_dir.as_deref().is_some_and(portable_marker_exists);
    let installer_marker_exists = exe_dir.as_deref().is_some_and(installer_marker_exists);

    resolve_data_dir_from_inputs(
        default_app_data_dir,
        exe_dir,
        portable_marker_exists,
        installer_marker_exists,
        env_data_dir,
    )
}

pub fn alternative_data_dir(resolution: &DataDirResolution) -> Option<PathBuf> {
    match &resolution.mode {
        DataDirMode::Portable { .. } => Some(resolution.default_data_dir.clone()),
        DataDirMode::Default => resolution.portable_data_dir.clone(),
        DataDirMode::EnvOverride => None,
    }
}

#[cfg(target_os = "windows")]
fn current_exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|path| path.parent().map(Path::to_path_buf))
}

#[cfg(target_os = "windows")]
fn portable_marker_exists(exe_dir: &Path) -> bool {
    exe_dir.join(PORTABLE_MARKER).is_file()
}

#[cfg(not(target_os = "windows"))]
fn portable_marker_exists(_exe_dir: &Path) -> bool {
    false
}

#[cfg(target_os = "windows")]
fn installer_marker_exists(exe_dir: &Path) -> bool {
    exe_dir.join(INSTALLER_MARKER).is_file()
}

#[cfg(not(target_os = "windows"))]
fn installer_marker_exists(_exe_dir: &Path) -> bool {
    false
}

fn resolve_data_dir_from_inputs(
    default_app_data_dir: PathBuf,
    exe_dir: Option<PathBuf>,
    portable_marker_exists: bool,
    installer_marker_exists: bool,
    env_data_dir: Option<PathBuf>,
) -> DataDirResolution {
    let portable_data_dir = exe_dir.as_ref().filter(|_| portable_marker_exists).map(|dir| dir.join("data"));

    if let Some(env_dir) = env_data_dir {
        return DataDirResolution {
            data_dir: env_dir,
            default_data_dir: default_app_data_dir,
            mode: DataDirMode::EnvOverride,
            portable_data_dir,
        };
    }

    if portable_marker_exists && !installer_marker_exists {
        if let Some(exe_dir) = exe_dir {
            return DataDirResolution {
                data_dir: exe_dir.join("data"),
                default_data_dir: default_app_data_dir,
                mode: DataDirMode::Portable { exe_dir },
                portable_data_dir,
            };
        }
    }

    DataDirResolution {
        data_dir: default_app_data_dir.clone(),
        default_data_dir: default_app_data_dir,
        mode: DataDirMode::Default,
        portable_data_dir,
    }
}

/// Database file name used before the SQDesk rename.
const LEGACY_DATABASE_FILE: &str = "dbx.db";

/// Carries the pre-rename database over to `current_name`, together with its
/// WAL, SHM and rollback-journal sidecars. Must run before the storage is
/// opened; it does nothing once the current file exists. The main file moves
/// last, so an interrupted run is picked up again on the next start.
pub fn migrate_legacy_database_file(data_dir: &Path, current_name: &str) -> std::io::Result<bool> {
    let legacy = data_dir.join(LEGACY_DATABASE_FILE);
    if data_dir.join(current_name).exists() || !legacy.exists() {
        return Ok(false);
    }
    for suffix in ["-wal", "-shm", "-journal", ""] {
        let from = data_dir.join(format!("{LEGACY_DATABASE_FILE}{suffix}"));
        if from.exists() {
            std::fs::rename(from, data_dir.join(format!("{current_name}{suffix}")))?;
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{alternative_data_dir, migrate_legacy_database_file, resolve_data_dir_from_inputs};

    #[test]
    fn portable_mode_can_import_from_default_data_dir() {
        let default_dir = PathBuf::from(r"C:\Users\Administrator\AppData\Roaming\com.sqdesk.app");
        let exe_dir = PathBuf::from(r"D:\Apps\SQDesk");

        let resolution = resolve_data_dir_from_inputs(default_dir.clone(), Some(exe_dir), true, false, None);

        assert_eq!(alternative_data_dir(&resolution), Some(default_dir));
    }

    #[test]
    fn installed_mode_can_import_from_leftover_portable_data_dir() {
        let default_dir = PathBuf::from(r"C:\Users\Administrator\AppData\Roaming\com.sqdesk.app");
        let exe_dir = PathBuf::from(r"C:\Program Files\SQDesk");

        let resolution = resolve_data_dir_from_inputs(default_dir, Some(exe_dir.clone()), true, true, None);

        assert_eq!(alternative_data_dir(&resolution), Some(exe_dir.join("data")));
    }

    #[test]
    fn env_override_does_not_import_from_implicit_alternative_dir() {
        let default_dir = PathBuf::from(r"C:\Users\Administrator\AppData\Roaming\com.sqdesk.app");
        let exe_dir = PathBuf::from(r"D:\Apps\SQDesk");

        let resolution = resolve_data_dir_from_inputs(
            default_dir,
            Some(exe_dir),
            true,
            false,
            Some(PathBuf::from(r"E:\SQDESKData")),
        );

        assert_eq!(alternative_data_dir(&resolution), None);
    }

    #[test]
    fn legacy_database_file_moves_with_its_sidecars_once() {
        let dir = std::env::temp_dir().join(format!("sqdesk-legacy-db-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("dbx.db"), b"main").unwrap();
        std::fs::write(dir.join("dbx.db-wal"), b"wal").unwrap();

        assert!(migrate_legacy_database_file(&dir, "sqdesk.db").unwrap());
        assert_eq!(std::fs::read(dir.join("sqdesk.db")).unwrap(), b"main");
        assert_eq!(std::fs::read(dir.join("sqdesk.db-wal")).unwrap(), b"wal");
        assert!(!dir.join("dbx.db").exists());

        // A current database is never replaced by a stray legacy file.
        std::fs::write(dir.join("dbx.db"), b"stale").unwrap();
        assert!(!migrate_legacy_database_file(&dir, "sqdesk.db").unwrap());
        assert_eq!(std::fs::read(dir.join("sqdesk.db")).unwrap(), b"main");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
