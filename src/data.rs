//! Data file management for libpostal.

use crate::error::Result;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(feature = "runtime-data")]
use std::io::Write;

#[cfg(feature = "runtime-data")]
use futures::future;

/// Essential libpostal data files that must be present for the library to function.
const REQUIRED_DATA_FILES: &[&str] = &[
    "address_expansions/address_dictionary.dat",
    "numex/numex.dat",
    "transliteration/transliteration.dat",
    "address_parser/address_parser_crf.dat",
    "address_parser/address_parser_phrases.dat",
    "address_parser/address_parser_postal_codes.dat",
    "address_parser/address_parser_vocab.trie",
    "language_classifier/language_classifier.dat",
];

/// Chunk size for multipart downloads (64MB).
const CHUNK_SIZE: usize = 64 * 1024 * 1024;

/// Default number of parallel download workers.
const DEFAULT_NUM_WORKERS: usize = 12;

/// Component file information.
const LIBPOSTAL_DATA_FILE_CHUNKS: usize = 1;
const LIBPOSTAL_PARSER_MODEL_CHUNKS: usize = 12;
const LIBPOSTAL_LANG_CLASS_MODEL_CHUNKS: usize = 1;

const LIBPOSTAL_DATA_FILE_LATEST_VERSION: &str = "v1.0.0";
const LIBPOSTAL_PARSER_MODEL_LATEST_VERSION: &str = "v1.0.0";
const LIBPOSTAL_LANG_CLASS_MODEL_LATEST_VERSION: &str = "v1.0.0";

const LIBPOSTAL_DATA_FILE: &str = "libpostal_data.tar.gz";
const LIBPOSTAL_PARSER_FILE: &str = "parser.tar.gz";
const LIBPOSTAL_LANG_CLASS_FILE: &str = "language_classifier.tar.gz";

const LIBPOSTAL_BASE_URL: &str = "https://github.com/openvenues/libpostal/releases/download";

/// Module directories for data organization.
const BASIC_MODULE_DIRS: &[&str] = &["address_expansions", "numex", "transliteration"];
const PARSER_MODULE_DIR: &str = "address_parser";
const LANGUAGE_CLASSIFIER_MODULE_DIR: &str = "language_classifier";

/// Data component types for libpostal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataComponent {
    /// Base data (address_expansions, numex, transliteration)
    Base,
    /// Address parser model
    Parser,
    /// Language classifier model  
    LanguageClassifier,
    /// All components
    All,
}

/// Information about a data component.
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Component version
    pub version: String,
    /// Number of chunks for multipart download
    pub num_chunks: usize,
    /// Archive filename
    pub filename: String,
    /// Component display name
    pub name: String,
    /// Subdirectories that will be created/updated
    pub subdirs: Vec<String>,
}

/// Download progress information.
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total bytes to download
    pub total_bytes: u64,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Current chunk being downloaded
    pub current_chunk: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Download speed in bytes per second
    pub speed_bps: u64,
}

/// GitHub release asset information.
#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    /// Asset name
    pub name: String,
    /// Download URL
    pub download_url: String,
    /// Asset size in bytes
    pub size: u64,
}

/// Get component information for a data component.
pub fn get_component_info(component: DataComponent) -> ComponentInfo {
    match component {
        DataComponent::Base => ComponentInfo {
            version: LIBPOSTAL_DATA_FILE_LATEST_VERSION.to_string(),
            num_chunks: LIBPOSTAL_DATA_FILE_CHUNKS,
            filename: LIBPOSTAL_DATA_FILE.to_string(),
            name: "data file".to_string(),
            subdirs: BASIC_MODULE_DIRS.iter().map(|s| s.to_string()).collect(),
        },
        DataComponent::Parser => ComponentInfo {
            version: LIBPOSTAL_PARSER_MODEL_LATEST_VERSION.to_string(),
            num_chunks: LIBPOSTAL_PARSER_MODEL_CHUNKS,
            filename: LIBPOSTAL_PARSER_FILE.to_string(),
            name: "parser data file".to_string(),
            subdirs: vec![PARSER_MODULE_DIR.to_string()],
        },
        DataComponent::LanguageClassifier => ComponentInfo {
            version: LIBPOSTAL_LANG_CLASS_MODEL_LATEST_VERSION.to_string(),
            num_chunks: LIBPOSTAL_LANG_CLASS_MODEL_CHUNKS,
            filename: LIBPOSTAL_LANG_CLASS_FILE.to_string(),
            name: "language classifier data file".to_string(),
            subdirs: vec![LANGUAGE_CLASSIFIER_MODULE_DIR.to_string()],
        },
        DataComponent::All => ComponentInfo {
            version: "all".to_string(),
            num_chunks: 0, // Special case handled separately
            filename: "all".to_string(),
            name: "all components".to_string(),
            subdirs: vec![],
        },
    }
}

/// Get version file path for a component.
fn get_version_file_path(component: DataComponent, data_dir: &Path) -> PathBuf {
    let filename = match component {
        DataComponent::Base => "base_data_file_version",
        DataComponent::Parser => "parser_model_file_version", 
        DataComponent::LanguageClassifier => "language_classifier_model_file_version",
        DataComponent::All => "data_version",
    };
    data_dir.join(filename)
}

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

    /// Helper to create data errors with consistent formatting
    fn data_error<S: Into<String>>(message: S) -> crate::error::Error {
        crate::error::Error::data_error(message.into())
    }

    /// Helper to create directories with error handling
    fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
        std::fs::create_dir_all(path.as_ref()).map_err(|e| {
            Self::data_error(format!("Failed to create directory {}: {e}", path.as_ref().display()))
        })
    }

    /// Check if required data files are present.
    pub fn is_data_available(&self) -> bool {
        if !self.data_dir.exists() {
            return false;
        }

        // Check for essential libpostal data files
        REQUIRED_DATA_FILES.iter().all(|file| {
            self.data_dir.join(file).exists()
        })
    }

    /// Download required data files.
    #[cfg(feature = "runtime-data")]
    pub async fn download_data(&self) -> Result<()> {
        // Implementation moved to ensure_data method
        self.ensure_data().await
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

    /// Verify integrity of data files.
    pub fn verify_data(&self) -> Result<()> {
        if !self.is_data_available() {
            return Err(crate::error::Error::data_error("Data files not found"));
        }

        // Check that files exist and are non-empty
        // Future enhancement: implement SHA256 checksum verification
        for file in REQUIRED_DATA_FILES {
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

    /// Check the version of a specific component.
    #[cfg(feature = "runtime-data")]
    fn check_component_version(&self, component: DataComponent) -> Result<Option<String>> {
        let version_file = get_version_file_path(component, &self.data_dir);
        if !version_file.exists() {
            return Ok(None);
        }

        let version = std::fs::read_to_string(&version_file)
            .map_err(|e| Self::data_error(format!("Failed to read version file: {e}")))?
            .trim()
            .to_string();

        Ok(Some(version))
    }

    /// Write version file for a component.
    #[cfg(feature = "runtime-data")]
    fn write_version_file(&self, component: DataComponent, version: &str) -> Result<()> {
        let version_file = get_version_file_path(component, &self.data_dir);
        std::fs::write(&version_file, version)
            .map_err(|e| Self::data_error(format!("Failed to write version file: {e}")))?;
        Ok(())
    }

    /// Get GitHub release asset URL for a component.
    #[cfg(feature = "runtime-data")]
    fn get_release_asset_url(&self, component: DataComponent) -> String {
        let info = get_component_info(component);
        format!("{}/{}/{}", LIBPOSTAL_BASE_URL, info.version, info.filename)
    }

    /// Download a file in multiple chunks using HTTP Range requests.
    #[cfg(feature = "runtime-data")]
    async fn download_release_multipart(&self, url: &str, filename: &Path, num_chunks: usize) -> Result<()> {
        println!("Downloading multipart: {}, num_chunks={}", url, num_chunks);
        
        let chunk_size = self.config.chunk_size;
        let mut download_tasks = Vec::new();

        // Create tasks for each chunk
        for i in 0..num_chunks {
            let chunk_url = url.to_string();
            let chunk_filename = filename.with_extension(format!("part{}", i + 1));
            let offset = i * chunk_size;
            let max_range = offset + chunk_size - 1;
            
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(self.config.chunk_timeout_seconds))
                .build()
                .map_err(|e| Self::data_error(format!("Failed to create HTTP client: {e}")))?;

            let task = tokio::spawn(async move {
                println!("Downloading part {}: filename={}, offset={}, max={}", 
                    i + 1, chunk_filename.display(), offset, max_range);

                let mut retries = 0;
                const MAX_RETRIES: usize = 3;

                while retries < MAX_RETRIES {
                    let result = client
                        .get(&chunk_url)
                        .header("Range", format!("bytes={}-{}", offset, max_range))
                        .send()
                        .await;

                    match result {
                        Ok(response) => {
                            if response.status().is_success() || response.status() == 206 {
                                let bytes = response.bytes().await
                                    .map_err(|e| format!("Failed to read response: {e}"))?;
                                
                                std::fs::write(&chunk_filename, &bytes)
                                    .map_err(|e| format!("Failed to write chunk: {e}"))?;
                                
                                return Ok::<(), String>(());
                            } else {
                                return Err(format!("HTTP error: {}", response.status()));
                            }
                        }
                        Err(e) => {
                            retries += 1;
                            if retries >= MAX_RETRIES {
                                return Err(format!("Failed after {} retries: {e}", MAX_RETRIES));
                            }
                            tokio::time::sleep(Duration::from_secs(2_u64.pow(retries as u32))).await;
                        }
                    }
                }
                Err("Max retries exceeded".to_string())
            });

            download_tasks.push(task);
        }

        // Wait for all downloads to complete
        let results = future::join_all(download_tasks).await;
        
        // Check for errors
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(Self::data_error(format!("Chunk {} failed: {e}", i + 1))),
                Err(e) => return Err(Self::data_error(format!("Task {} panicked: {e}", i + 1))),
            }
        }

        // Reassemble the file
        let mut output_file = std::fs::File::create(filename)
            .map_err(|e| Self::data_error(format!("Failed to create output file: {e}")))?;

        for i in 0..num_chunks {
            let chunk_filename = filename.with_extension(format!("part{}", i + 1));
            let chunk_data = std::fs::read(&chunk_filename)
                .map_err(|e| Self::data_error(format!("Failed to read chunk {}: {e}", i + 1)))?;
            
            output_file.write_all(&chunk_data)
                .map_err(|e| Self::data_error(format!("Failed to write to output file: {e}")))?;
            
            // Clean up chunk file
            std::fs::remove_file(&chunk_filename).ok();
        }

        println!("Multipart download completed: {}", filename.display());
        Ok(())
    }

    /// Download a single component.
    #[cfg(feature = "runtime-data")]
    async fn download_component(&self, component: DataComponent) -> Result<()> {
        let info = get_component_info(component);
        let url = self.get_release_asset_url(component);
        let local_path = self.data_dir.join(&info.filename);

        // Check if update is needed
        let current_version = self.check_component_version(component)?;
        if let Some(current) = current_version {
            if current == info.version {
                println!("libpostal {} up to date", info.name);
                return Ok(());
            }
        }

        println!("New libpostal {} available", info.name);

        // Download the file
        if info.num_chunks > 1 {
            self.download_release_multipart(&url, &local_path, info.num_chunks).await?;
        } else {
            // Single file download
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(self.config.timeout_seconds))
                .build()
                .map_err(|e| Self::data_error(format!("Failed to create HTTP client: {e}")))?;

            let response = client.get(&url).send().await
                .map_err(|e| Self::data_error(format!("Failed to download {}: {e}", url)))?;

            if !response.status().is_success() {
                return Err(Self::data_error(format!("Download failed with status: {}", response.status())));
            }

            let bytes = response.bytes().await
                .map_err(|e| Self::data_error(format!("Failed to read response: {e}")))?;

            std::fs::write(&local_path, &bytes)
                .map_err(|e| Self::data_error(format!("Failed to write file: {e}")))?;
        }

        // Remove old subdirectories
        for subdir in &info.subdirs {
            let path = self.data_dir.join(subdir);
            if path.exists() {
                std::fs::remove_dir_all(&path)
                    .map_err(|e| Self::data_error(format!("Failed to remove {}: {e}", path.display())))?;
            }
        }

        // Extract the archive
        self.extract_tar_gz(&local_path)?;

        // Clean up downloaded file
        std::fs::remove_file(&local_path).ok();

        // Write version file
        self.write_version_file(component, &info.version)?;

        println!("libpostal {} updated successfully", info.name);
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

    /// Download real libpostal data files using native Rust implementation
    #[cfg(feature = "runtime-data")]
    async fn download_real_data(&self) -> Result<()> {
        // Create data directory
        Self::create_dir_all(&self.data_dir)?;

        println!("Setting up libpostal data files...");

        // Try native component-based download first
        self.download_native_components().await
    }

    /// Download all components using the native Rust implementation.
    #[cfg(feature = "runtime-data")]
    async fn download_native_components(&self) -> Result<()> {
        // Download base data files
        self.download_component(DataComponent::Base).await?;
        
        // Download parser model
        self.download_component(DataComponent::Parser).await?;
        
        // Download language classifier model
        self.download_component(DataComponent::LanguageClassifier).await?;

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
    /// Number of parallel download workers (default: 12)
    pub download_workers: usize,
    /// Chunk size for multipart downloads (default: 64MB)
    pub chunk_size: usize,
    /// Number of retry attempts for failed downloads (default: 3)
    pub max_retries: usize,
    /// Timeout for individual chunk downloads (default: 30s)
    pub chunk_timeout_seconds: u64,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            auto_download: true,
            verify_integrity: true,
            base_url: "https://github.com/openvenues/libpostal/releases/download/v1.1/".to_string(),
            timeout_seconds: 300, // 5 minutes
            download_workers: DEFAULT_NUM_WORKERS,
            chunk_size: CHUNK_SIZE,
            max_retries: 3,
            chunk_timeout_seconds: 30,
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
