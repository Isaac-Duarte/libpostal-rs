//! # libpostal-rs
//!
//! Static Rust bindings for libpostal - international address parsing and normalization.
//!
//! This library provides safe, idiomatic Rust bindings to the libpostal C library,
//! allowing you to parse and normalize international addresses without requiring
//! system-level dependencies.
//!
//! ## Features
//!
//! - **Address Parsing**: Parse addresses into structured components
//! - **Address Normalization**: Expand abbreviations and normalize formats
//! - **Language Detection**: Automatic language classification
//! - **Zero External Dependencies**: No need to install libpostal separately
//! - **Memory Safe**: Rust wrappers with proper memory management
//! - **Thread Safe**: Support for concurrent usage
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use libpostal_rs::LibPostal;
//!
//! // Initialize libpostal (downloads data files on first run)
//! let postal = LibPostal::new().await?;
//!
//! // Parse an address
//! let parsed = postal.parse_address("123 Main St, New York, NY 10001").await?;
//! println!("House number: {}", parsed.house_number.unwrap_or_default());
//! println!("Street: {}", parsed.road.unwrap_or_default());
//! println!("City: {}", parsed.city.unwrap_or_default());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod data;
pub mod error;
pub mod ffi;
pub mod normalizer;
pub mod parser;
pub mod profiling;
pub mod types;

// Re-export main API
pub use error::{Error, Result};
pub use normalizer::{AddressNormalizer, NormalizedAddress};
pub use parser::{AddressParser, ParsedAddress};
pub use types::*;

/// Main entry point for libpostal functionality.
///
/// This struct manages the libpostal library lifecycle and provides
/// access to parsing and normalization functionality.
///
/// # Examples
///
/// ```rust,no_run
/// use libpostal_rs::LibPostal;
///
/// // Initialize with default configuration
/// let postal = LibPostal::new().await?;
///
/// // Parse an address
/// let parsed = postal.parse_address("123 Main St, New York, NY").await?;
///
/// // Normalize an address
/// let normalized = postal.normalize_address("123 Main Street").await?;
/// # Ok::<(), libpostal_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct LibPostal {
    config: LibPostalConfig,
}

impl LibPostal {
    /// Initialize libpostal with default configuration.
    ///
    /// This will ensure libpostal is properly initialized and data files
    /// are available. On first run, this may download required data files.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails, such as when data files
    /// cannot be found or libpostal setup fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::LibPostal;
    ///
    /// let postal = LibPostal::new().await?;
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub async fn new() -> Result<Self> {
        Self::with_config(LibPostalConfig::default()).await
    }

    /// Initialize libpostal with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for libpostal initialization
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::{LibPostal, LibPostalConfig};
    ///
    /// let config = LibPostalConfig::builder()
    ///     .auto_download_data(true)
    ///     .verify_data_integrity(true)
    ///     .build();
    ///
    /// let postal = LibPostal::with_config(config).await?;
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub async fn with_config(config: LibPostalConfig) -> Result<Self> {
        // Ensure data is available if auto-download is enabled
        if config.auto_download_data {
            let data_manager = data::DataManager::with_config(config.data_config.clone());
            #[cfg(feature = "runtime-data")]
            {
                data_manager.ensure_data().await?;
            }
            #[cfg(not(feature = "runtime-data"))]
            {
                if !data_manager.is_data_available() {
                    return Err(Error::initialization_failed(
                        "Data files not available and runtime-data feature disabled",
                    ));
                }
            }
        }

        // Initialize the FFI layer
        ffi::initialize()?;

        Ok(Self { config })
    }

    /// Create a new address parser with default options.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::LibPostal;
    ///
    /// let postal = LibPostal::new().await?;
    /// let parser = postal.parser();
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn parser(&self) -> AddressParser {
        AddressParser::new()
    }

    /// Create a new address normalizer with default options.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::LibPostal;
    ///
    /// let postal = LibPostal::new().await?;
    /// let normalizer = postal.normalizer();
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn normalizer(&self) -> AddressNormalizer {
        AddressNormalizer::new()
    }

    /// Parse an address string into structured components.
    ///
    /// This is a convenience method that creates a parser with default options
    /// and parses the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address string to parse
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::LibPostal;
    ///
    /// let postal = LibPostal::new().await?;
    /// let parsed = postal.parse_address("123 Main St, New York, NY 10001")?;
    ///
    /// println!("House number: {:?}", parsed.house_number);
    /// println!("Street: {:?}", parsed.road);
    /// println!("City: {:?}", parsed.city);
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn parse_address(&self, address: &str) -> Result<parser::ParsedAddress> {
        self.parser().parse(address)
    }

    /// Parse an address with language and country hints.
    ///
    /// # Arguments
    ///
    /// * `address` - The address string to parse
    /// * `language` - Optional language hint (e.g., "en", "es", "fr")
    /// * `country` - Optional country hint (e.g., "US", "CA", "GB")
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::LibPostal;
    ///
    /// let postal = LibPostal::new().await?;
    /// let parsed = postal.parse_address_with_hints(
    ///     "123 Main St, New York, NY",
    ///     Some("en"),
    ///     Some("US")
    /// )?;
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn parse_address_with_hints(
        &self,
        address: &str,
        language: Option<&str>,
        country: Option<&str>,
    ) -> Result<parser::ParsedAddress> {
        let mut parser = self.parser();
        if let Some(lang) = language {
            parser = parser.with_language(types::Language::from_str(lang));
        }
        if let Some(ctry) = country {
            parser = parser.with_country(types::Country::from_str(ctry));
        }
        parser.parse(address)
    }

    /// Normalize an address string.
    ///
    /// This is a convenience method that creates a normalizer with default options
    /// and normalizes the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address string to normalize
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libpostal_rs::LibPostal;
    ///
    /// let postal = LibPostal::new().await?;
    /// let normalized = postal.normalize_address("123 Main St")?;
    ///
    /// for expansion in normalized.expansions {
    ///     println!("Normalized: {}", expansion);
    /// }
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn normalize_address(&self, address: &str) -> Result<normalizer::NormalizedAddress> {
        self.normalizer().normalize(address)
    }

    /// Get the configuration used by this instance.
    pub fn config(&self) -> &LibPostalConfig {
        &self.config
    }
}

/// Configuration for LibPostal initialization and behavior.
///
/// This struct allows customizing various aspects of libpostal's behavior,
/// including data management and processing options.
#[derive(Debug, Clone)]
pub struct LibPostalConfig {
    /// Whether to automatically download data files if missing
    pub auto_download_data: bool,

    /// Whether to verify data file integrity on startup
    pub verify_data_integrity: bool,

    /// Data management configuration
    pub data_config: data::DataConfig,
}

impl Default for LibPostalConfig {
    fn default() -> Self {
        Self {
            auto_download_data: true,
            verify_data_integrity: true,
            data_config: data::DataConfig::default(),
        }
    }
}

impl LibPostalConfig {
    /// Create a new configuration builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use libpostal_rs::LibPostalConfig;
    ///
    /// let config = LibPostalConfig::builder()
    ///     .auto_download_data(false)
    ///     .verify_data_integrity(true)
    ///     .build();
    /// ```
    pub fn builder() -> LibPostalConfigBuilder {
        LibPostalConfigBuilder::new()
    }
}

/// Builder for LibPostalConfig.
#[derive(Debug, Clone)]
pub struct LibPostalConfigBuilder {
    auto_download_data: bool,
    verify_data_integrity: bool,
    data_config: data::DataConfig,
}

impl LibPostalConfigBuilder {
    /// Create a new configuration builder with default values.
    pub fn new() -> Self {
        Self {
            auto_download_data: true,
            verify_data_integrity: true,
            data_config: data::DataConfig::default(),
        }
    }

    /// Set whether to automatically download data files.
    pub fn auto_download_data(mut self, enabled: bool) -> Self {
        self.auto_download_data = enabled;
        self
    }

    /// Set whether to verify data file integrity.
    pub fn verify_data_integrity(mut self, enabled: bool) -> Self {
        self.verify_data_integrity = enabled;
        self
    }

    /// Set the data configuration.
    pub fn data_config(mut self, config: data::DataConfig) -> Self {
        self.data_config = config;
        self
    }

    /// Set a custom data directory.
    pub fn data_dir<P: Into<std::path::PathBuf>>(mut self, dir: P) -> Self {
        self.data_config.data_dir = dir.into();
        self
    }

    /// Build the configuration.
    pub fn build(self) -> LibPostalConfig {
        LibPostalConfig {
            auto_download_data: self.auto_download_data,
            verify_data_integrity: self.verify_data_integrity,
            data_config: self.data_config,
        }
    }
}

impl Default for LibPostalConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
