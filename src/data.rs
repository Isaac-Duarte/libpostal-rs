//! Data file management for libpostal.

use crate::error::Result;
use std::path::{Path, PathBuf};

/// Data file manager for libpostal.
pub struct DataManager {
    data_dir: PathBuf,
    config: DataConfig,
}

impl DataManager {
    /// Create a new data manager with the default data directory.
    pub fn new() -> Self {
        Self {
            data_dir: default_data_dir(),
            config: DataConfig::default(),
        }
    }

    /// Create a new data manager with a custom data directory.
    pub fn with_data_dir<P: AsRef<Path>>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            config: DataConfig::default(),
        }
    }

    /// Create a new data manager with custom configuration.
    pub fn with_config(config: DataConfig) -> Self {
        Self {
            data_dir: config.data_dir.clone(),
            config,
        }
    }

    /// Get the data directory path.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get the configuration.
    pub fn config(&self) -> &DataConfig {
        &self.config
    }

    /// Check if required data files are present.
    pub fn is_data_available(&self) -> bool {
        if !self.data_dir.exists() {
            return false;
        }

        // Check for essential libpostal data files
        let required_files = [
            "address_expansions/address_dictionary.dat",
            "numex/numex.dat",
            "transliteration/transliteration.dat",
            "address_parser/address_parser_crf.dat",
            "address_parser/address_parser_phrases.dat",
            "address_parser/address_parser_postal_codes.dat",
            "address_parser/address_parser_vocab.trie",
            "language_classifier/language_classifier.dat",
        ];

        for file in &required_files {
            if !self.data_dir.join(file).exists() {
                return false;
            }
        }

        true
    }

    /// Download required data files.
    #[cfg(feature = "runtime-data")]
    pub async fn download_data(&self) -> Result<()> {
        // Implementation moved to ensure_data method
        self.ensure_data().await
    }

    /// Verify integrity of data files.
    pub fn verify_data(&self) -> Result<()> {
        if !self.is_data_available() {
            return Err(crate::error::Error::data_error("Data files not found"));
        }

        // Check that files exist and are non-empty
        // Future enhancement: implement SHA256 checksum verification
        let required_files = [
            "address_expansions/address_dictionary.dat",
            "numex/numex.dat",
            "transliteration/transliteration.dat",
            "address_parser/address_parser_crf.dat",
            "address_parser/address_parser_phrases.dat",
            "address_parser/address_parser_postal_codes.dat",
            "address_parser/address_parser_vocab.trie",
            "language_classifier/language_classifier.dat",
        ];

        for file in &required_files {
            let path = self.data_dir.join(file);
            if !path.exists() {
                return Err(crate::error::Error::data_error(format!(
                    "Missing data file: {file}"
                )));
            }

            let metadata = std::fs::metadata(&path).map_err(|e| {
                crate::error::Error::data_error(format!("Failed to read metadata for {file}: {e}"))
            })?;

            if metadata.len() == 0 {
                return Err(crate::error::Error::data_error(format!(
                    "Empty data file: {file}"
                )));
            }
        }

        Ok(())
    }

    /// Get the size of downloaded data.
    pub fn data_size(&self) -> Result<u64> {
        if !self.data_dir.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;

        fn visit_dir(dir: &Path, total: &mut u64) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    visit_dir(&entry.path(), total)?;
                } else {
                    *total += metadata.len();
                }
            }
            Ok(())
        }

        visit_dir(&self.data_dir, &mut total_size).map_err(|e| {
            crate::error::Error::data_error(format!("Failed to calculate data size: {e}"))
        })?;

        Ok(total_size)
    }

    /// Clean up old or corrupted data files.
    pub fn cleanup(&self) -> Result<()> {
        if self.data_dir.exists() {
            std::fs::remove_dir_all(&self.data_dir).map_err(|e| {
                crate::error::Error::data_error(format!("Failed to cleanup data directory: {e}"))
            })?;
        }
        Ok(())
    }

    /// Ensure data is available, downloading if necessary.
    #[cfg(feature = "runtime-data")]
    pub async fn ensure_data(&self) -> Result<()> {
        if !self.is_data_available() {
            if self.config.auto_download {
                println!("Downloading libpostal data files...");
                self.download_real_data().await?;
            } else {
                return Err(crate::error::Error::data_error(
                    "Data files not available and auto_download is disabled",
                ));
            }
        }

        if self.config.verify_integrity {
            self.verify_data()?;
        }

        Ok(())
    }

    /// Download real libpostal data files
    #[cfg(feature = "runtime-data")]
    async fn download_real_data(&self) -> Result<()> {
        use std::fs;

        // Create data directory
        fs::create_dir_all(&self.data_dir).map_err(|e| {
            crate::error::Error::data_error(format!("Failed to create data directory: {e}"))
        })?;

        println!("Setting up libpostal data files...");

        // Method 1: Try using libpostal_data command if available
        if let Ok(()) = self.download_with_libpostal_data().await {
            println!("✓ Successfully downloaded data using libpostal_data command");
            return Ok(());
        }

        // Method 2: Try to copy from system libpostal installation
        if let Ok(()) = self.copy_from_system_libpostal().await {
            println!("✓ Successfully copied data from system libpostal installation");
            return Ok(());
        }

        // Method 3: Try to copy from current working directory (development scenario)
        if let Ok(()) = self.copy_from_project_data().await {
            println!("✓ Successfully copied data from project directory");
            return Ok(());
        }

        // Method 4: Try to download directly from libpostal's data sources
        if let Ok(()) = self.download_from_official_sources().await {
            println!("✓ Successfully downloaded data from official sources");
            return Ok(());
        }

        // Fall back to informative error message
        Err(crate::error::Error::data_error(
            "Could not find or download libpostal data files.\n\
            \n\
            To resolve this issue, try one of the following:\n\
            \n\
            1. Install libpostal system-wide:\n\
               https://github.com/openvenues/libpostal#installation-maclinux\n\
            \n\
            2. Set LIBPOSTAL_DATA_DIR environment variable:\n\
               export LIBPOSTAL_DATA_DIR=/path/to/libpostal/data\n\
            \n\
            3. Copy data files manually to: ~/.cache/libpostal-rs/\n\
            \n\
            4. Run the setup script: ./scripts/setup-data.sh",
        ))
    }

    /// Try to download data using libpostal_data command
    #[cfg(feature = "runtime-data")]
    async fn download_with_libpostal_data(&self) -> Result<()> {
        // Check if libpostal_data command is available
        let output = std::process::Command::new("libpostal_data")
            .arg("--help")
            .output();

        if output.is_err() {
            return Err(crate::error::Error::data_error(
                "libpostal_data command not found",
            ));
        }

        println!("Found libpostal_data command, downloading data files...");
        println!("This may take a while as the data files are large (~1GB)");

        let output = std::process::Command::new("libpostal_data")
            .arg("download")
            .arg("all")
            .arg(&self.data_dir)
            .output()
            .map_err(|e| {
                crate::error::Error::data_error(format!("Failed to run libpostal_data: {e}"))
            })?;

        if !output.status.success() {
            return Err(crate::error::Error::data_error(format!(
                "libpostal_data failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Try to download from official libpostal data sources
    #[cfg(feature = "runtime-data")]
    async fn download_from_official_sources(&self) -> Result<()> {
        println!("Attempting to download from official libpostal data sources...");
        println!("This may take a while as the data files are large (~1GB)");

        // Libpostal data is hosted on various CDNs and S3 buckets
        // We'll try the most common endpoints
        let data_urls = [
            "https://github.com/openvenues/libpostal/releases/download/v1.1/libpostal_data_v1.tar.gz",
            "http://download.geonames.org/libpostal/libpostal_data_v1.tar.gz", 
        ];

        for url in &data_urls {
            println!("Trying to download from: {url}");
            match self.download_and_extract_data(url).await {
                Ok(()) => {
                    println!("✓ Successfully downloaded and extracted data");
                    return Ok(());
                }
                Err(e) => {
                    println!("⚠ Failed to download from {url}: {e}");
                    continue;
                }
            }
        }

        Err(crate::error::Error::data_error(
            "All download sources failed",
        ))
    }

    /// Download and extract data archive
    #[cfg(feature = "runtime-data")]
    async fn download_and_extract_data(&self, url: &str) -> Result<()> {
        use std::fs;
        use std::io::Write;

        let response = reqwest::get(url).await.map_err(|e| {
            crate::error::Error::data_error(format!("Failed to download data: {e}"))
        })?;

        if !response.status().is_success() {
            return Err(crate::error::Error::data_error(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let content = response.bytes().await.map_err(|e| {
            crate::error::Error::data_error(format!("Failed to read response: {e}"))
        })?;

        println!("Downloaded {} bytes, extracting...", content.len());

        // Save to archive file
        let archive_path = self.data_dir.join("data.tar.gz");
        let mut file = fs::File::create(&archive_path).map_err(|e| {
            crate::error::Error::data_error(format!("Failed to create archive file: {e}"))
        })?;

        file.write_all(&content).map_err(|e| {
            crate::error::Error::data_error(format!("Failed to write archive file: {e}"))
        })?;

        // Extract the archive
        self.extract_tar_gz(&archive_path)?;

        // Clean up archive file
        fs::remove_file(&archive_path).ok();

        Ok(())
    }

    /// Extract tar.gz file to data directory
    #[cfg(feature = "runtime-data")]
    fn extract_tar_gz(&self, archive_path: &std::path::Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use std::fs::File;
        use tar::Archive;

        let file = File::open(archive_path)
            .map_err(|e| crate::error::Error::data_error(format!("Failed to open archive: {e}")))?;

        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);

        archive.unpack(&self.data_dir).map_err(|e| {
            crate::error::Error::data_error(format!("Failed to extract archive: {e}"))
        })?;

        Ok(())
    }

    /// Try to copy data from system libpostal installation
    #[cfg(feature = "runtime-data")]
    async fn copy_from_system_libpostal(&self) -> Result<()> {
        // Common system paths where libpostal data might be installed
        let system_paths = [
            "/usr/share/libpostal",
            "/usr/local/share/libpostal",
            "/opt/libpostal",
            "/opt/local/share/libpostal",
        ];

        for path in &system_paths {
            let system_data_dir = std::path::PathBuf::from(path);
            if system_data_dir.exists() && self.validate_data_dir(&system_data_dir) {
                return self.copy_data_directory(&system_data_dir).await;
            }
        }

        Err(crate::error::Error::data_error(
            "No system libpostal installation found",
        ))
    }

    /// Try to copy data from project directory (for development)
    #[cfg(feature = "runtime-data")]
    async fn copy_from_project_data(&self) -> Result<()> {
        let project_data_paths = [
            "data/libpostal",
            "../data/libpostal",
            "../../data/libpostal",
        ];

        for path in &project_data_paths {
            let project_data_dir = std::path::PathBuf::from(path);
            if project_data_dir.exists() && self.validate_data_dir(&project_data_dir) {
                return self.copy_data_directory(&project_data_dir).await;
            }
        }

        Err(crate::error::Error::data_error(
            "No project data directory found",
        ))
    }

    /// Validate that a directory contains the expected libpostal data structure
    fn validate_data_dir(&self, dir: &std::path::Path) -> bool {
        let required_files = [
            "address_expansions/address_dictionary.dat",
            "numex/numex.dat",
            "transliteration/transliteration.dat",
            "address_parser/address_parser_crf.dat",
        ];

        for file in &required_files {
            if !dir.join(file).exists() {
                return false;
            }
        }

        true
    }

    /// Copy data directory from source to target
    #[cfg(feature = "runtime-data")]
    async fn copy_data_directory(&self, source_dir: &std::path::Path) -> Result<()> {
        use std::fs;

        println!("Copying data from: {}", source_dir.display());

        // Use std::process::Command to run cp -r for efficiency
        let output = std::process::Command::new("cp")
            .arg("-r")
            .arg(source_dir)
            .arg(self.data_dir.parent().unwrap()) // Copy to parent, then rename
            .output()
            .map_err(|e| crate::error::Error::data_error(format!("Failed to copy data: {e}")))?;

        if !output.status.success() {
            return Err(crate::error::Error::data_error(format!(
                "Copy failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Rename to correct directory name if needed
        let copied_dir = self
            .data_dir
            .parent()
            .unwrap()
            .join(source_dir.file_name().unwrap());
        if copied_dir != self.data_dir {
            fs::rename(&copied_dir, &self.data_dir).map_err(|e| {
                crate::error::Error::data_error(format!("Failed to rename data directory: {e}"))
            })?;
        }

        Ok(())
    }
}

impl Default for DataManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default data directory for libpostal.
pub fn default_data_dir() -> PathBuf {
    // Check for environment variable first
    if let Ok(env_data_dir) = std::env::var("LIBPOSTAL_DATA_DIR") {
        let path = PathBuf::from(env_data_dir);
        if path.exists() {
            return path;
        }
    }

    // Check for data downloaded during build (compile-time env var)
    if let Some(built_data_dir) = option_env!("LIBPOSTAL_BUILT_DATA_DIR") {
        let path = PathBuf::from(built_data_dir);
        if path.exists() {
            return path;
        }
    }

    // First check if we have data in the project directory (for development)
    let project_data_dir = PathBuf::from("data/libpostal");
    if project_data_dir.exists() {
        return project_data_dir;
    }

    // Fall back to cache directory
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("libpostal-rs")
    } else {
        // Fallback to current directory
        PathBuf::from(".libpostal-rs")
    }
}

/// Configuration for data management.
#[derive(Debug, Clone)]
pub struct DataConfig {
    /// Data directory path
    pub data_dir: PathBuf,
    /// Whether to download data automatically
    pub auto_download: bool,
    /// Whether to verify data integrity
    pub verify_integrity: bool,
    /// Base URL for data downloads
    pub base_url: String,
    /// Connection timeout for downloads
    pub timeout_seconds: u64,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            auto_download: true,
            verify_integrity: true,
            base_url: "https://github.com/openvenues/libpostal/releases/download/v1.1/".to_string(),
            timeout_seconds: 300, // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_manager_default() {
        let manager = DataManager::new();
        // Data directory should exist and be valid
        let data_dir = manager.data_dir();
        assert!(
            !data_dir.as_os_str().is_empty(),
            "Data directory should not be empty"
        );
    }

    #[test]
    fn test_data_config_default() {
        let config = DataConfig::default();
        assert!(config.auto_download);
        assert!(config.verify_integrity);
        assert!(config.timeout_seconds > 0);
    }
}
