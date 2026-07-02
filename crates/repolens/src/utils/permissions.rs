//! File permissions utilities for secure configuration file handling

use std::path::Path;

/// Sets secure file permissions (0600 - owner read/write only) on Unix systems.
///
/// This function ensures that sensitive configuration files like `.repolens.toml`
/// are only readable and writable by the file owner, protecting any secrets
/// or sensitive configuration values.
///
/// # Arguments
///
/// * `path` - Path to the file to secure
///
/// # Returns
///
/// * `Ok(())` - Permissions were set successfully (or no-op on non-Unix)
/// * `Err(io::Error)` - Failed to set permissions
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use repolens::utils::permissions::set_secure_permissions;
///
/// let config_path = Path::new(".repolens.toml");
/// set_secure_permissions(config_path).expect("Failed to set permissions");
/// ```
#[cfg(unix)]
pub fn set_secure_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600); // rw-------
    std::fs::set_permissions(path, perms)
}

/// No-op implementation for non-Unix systems.
///
/// Windows has a different permissions model, so this function
/// does nothing on non-Unix platforms.
#[cfg(not(unix))]
pub fn set_secure_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(()) // No-op on non-Unix systems
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[cfg(unix)]
    #[test]
    fn test_set_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_config.toml");

        // Create a test file
        File::create(&file_path).expect("Failed to create test file");

        // Set secure permissions
        set_secure_permissions(&file_path).expect("Failed to set permissions");

        // Verify permissions are 0600
        let metadata = std::fs::metadata(&file_path).expect("Failed to get metadata");
        let mode = metadata.permissions().mode() & 0o777; // Mask to get permission bits only
        assert_eq!(mode, 0o600, "Expected permissions 0600, got {:o}", mode);
    }

    #[cfg(unix)]
    #[test]
    fn test_set_secure_permissions_nonexistent_file() {
        let path = Path::new("/nonexistent/path/to/file.toml");
        let result = set_secure_permissions(path);
        assert!(result.is_err(), "Expected error for nonexistent file");
    }

    #[cfg(not(unix))]
    #[test]
    fn test_set_secure_permissions_noop() {
        let path = Path::new("any_path.toml");
        let result = set_secure_permissions(path);
        assert!(result.is_ok(), "Non-Unix should always return Ok");
    }
}
