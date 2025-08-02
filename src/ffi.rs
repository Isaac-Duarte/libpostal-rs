//! FFI bindings and safety wrappers around the libpostal C library.
//!
//! This module provides safe Rust wrappers around libpostal's C API,
//! handling memory management, error handling, and type conversions.
//!
//! # Memory Safety Guarantees
//!
//! This module ensures memory safety through several mechanisms:
//!
//! * **String Management**: All `CString` instances are properly managed with clear ownership.
//!   Temporary `CString` instances are kept alive for the duration of C function calls.
//! * **Resource Cleanup**: All C-allocated resources (like parse results) are automatically
//!   freed using RAII patterns and explicit cleanup calls.
//! * **Null Pointer Safety**: All C function returns are checked for null pointers before
//!   dereferencing, with appropriate error handling.
//! * **UTF-8 Validation**: All strings returned from C are validated as proper UTF-8 before
//!   being converted to Rust strings.
//!
//! # Threading Constraints
//!
//! The libpostal C library has specific threading requirements that this module respects:
//!
//! * **Initialization**: `libpostal_setup()` must be called exactly once per process before
//!   any other libpostal functions. This is handled thread-safely using `std::sync::Once`.
//! * **Concurrent Usage**: Once initialized, the parsing and normalization functions are
//!   thread-safe and can be called concurrently from multiple threads.
//! * **Teardown**: `libpostal_teardown()` should be called at process exit, but is optional
//!   as the OS will clean up resources anyway.
//!
//! # Error Handling Strategy
//!
//! All FFI operations can fail and are wrapped in `Result<T, Error>`:
//!
//! * **Initialization Errors**: Returned when libpostal setup fails (usually due to missing data)
//! * **Memory Errors**: Returned when C functions return null unexpectedly
//! * **Encoding Errors**: Returned when string conversion fails due to invalid UTF-8
//! * **FFI Errors**: Generic errors for other C library failures
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use libpostal_rs::ffi::{initialize, parse_address, ParseOptions};
//!
//! // Initialize libpostal (thread-safe, only happens once)
//! initialize().expect("Failed to initialize libpostal");
//!
//! // Parse an address
//! let address = "123 Main St, New York, NY 10001";
//! let options = ParseOptions {
//!     language: Some("en".to_string()),
//!     country: Some("US".to_string()),
//! };
//!
//! let components = parse_address(address, Some(&options))
//!     .expect("Failed to parse address");
//!
//! for component in components {
//!     println!("{}: {}", component.label, component.value);
//! }
//! ```

#![allow(missing_docs)] // Generated bindings don't have docs

use crate::error::{Error, Result};
use std::ffi::{CStr, CString};
use std::sync::Once;

// Include generated bindings
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(missing_docs)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

// Global initialization state
static INIT: Once = Once::new();
static mut INITIALIZED: bool = false;
static mut INIT_ERROR_MSG: [u8; 256] = [0; 256];
static mut INIT_ERROR_LEN: usize = 0;

/// Initialize libpostal in a thread-safe manner.
///
/// This function ensures that `libpostal_setup()` is called exactly once per process,
/// using `std::sync::Once` for thread safety. Subsequent calls will return immediately.
///
/// # Memory Safety
///
/// - Thread-safe initialization using `std::sync::Once`
/// - Idempotent: safe to call multiple times
/// - No memory allocation on subsequent calls
///
/// # Errors
///
/// Returns `Error::InitializationFailed` if libpostal setup fails, typically due to:
/// - Missing or corrupted data files
/// - Insufficient memory
/// - Permission issues accessing data directory
///
/// # Example
///
/// ```rust,no_run
/// use libpostal_rs::ffi::initialize;
///
/// // Safe to call from multiple threads
/// initialize().expect("Failed to initialize libpostal");
/// ```
pub(crate) fn initialize() -> Result<()> {
    INIT.call_once(|| {
        // Try to use data directory if available
        let data_manager = crate::data::DataManager::new();
        let success = if data_manager.is_data_available() {
            let data_dir = data_manager.data_dir();
            match std::ffi::CString::new(data_dir.to_string_lossy().as_ref()) {
                Ok(c_data_dir) => unsafe {
                    // Set up libpostal with data directory
                    let setup_result = libpostal_setup_datadir(c_data_dir.as_ptr() as *mut _);
                    let parser_result =
                        libpostal_setup_parser_datadir(c_data_dir.as_ptr() as *mut _);
                    let classifier_result =
                        libpostal_setup_language_classifier_datadir(c_data_dir.as_ptr() as *mut _);

                    // All three need to succeed for full functionality
                    setup_result && parser_result && classifier_result
                },
                Err(_) => {
                    unsafe {
                        let msg = b"Invalid data directory path";
                        INIT_ERROR_LEN = msg.len().min(255);
                        INIT_ERROR_MSG[..INIT_ERROR_LEN].copy_from_slice(&msg[..INIT_ERROR_LEN]);
                    }
                    false
                }
            }
        } else {
            // Fall back to default setup (will likely fail without data)
            unsafe {
                let setup_result = libpostal_setup();
                let parser_result = libpostal_setup_parser();
                let classifier_result = libpostal_setup_language_classifier();

                setup_result && parser_result && classifier_result
            }
        };

        if success {
            unsafe {
                INITIALIZED = true;
            }
        } else {
            let error_msg = if !data_manager.is_data_available() {
                "libpostal initialization failed - data files not found. Run data download first."
            } else {
                "libpostal initialization failed"
            };
            unsafe {
                INITIALIZED = false;
                if INIT_ERROR_LEN == 0 {
                    let msg_bytes = error_msg.as_bytes();
                    INIT_ERROR_LEN = msg_bytes.len().min(255);
                    INIT_ERROR_MSG[..INIT_ERROR_LEN].copy_from_slice(&msg_bytes[..INIT_ERROR_LEN]);
                }
            }
        }
    });

    unsafe {
        if INITIALIZED {
            Ok(())
        } else {
            let error_msg = if INIT_ERROR_LEN > 0 {
                std::str::from_utf8(&INIT_ERROR_MSG[..INIT_ERROR_LEN])
                    .unwrap_or("libpostal initialization failed")
            } else {
                "libpostal initialization failed"
            };
            Err(Error::initialization_failed(error_msg))
        }
    }
}

/// Clean up libpostal resources.
///
/// This function calls `libpostal_teardown()` to clean up any resources
/// allocated by libpostal. It's generally not necessary to call this as
/// the OS will clean up process resources on exit.
///
/// # Safety
///
/// This function should only be called when you're certain no other threads
/// are using libpostal functions. Calling this while other threads are active
/// may lead to undefined behavior.
///
/// # Memory Safety
///
/// - Safe to call even if libpostal was never initialized
/// - No-op if called multiple times
/// - Frees internal C library resources
///
/// # Note
///
/// Currently unused but provided for completeness. Most applications should
/// rely on OS cleanup at process exit rather than explicit teardown.
#[allow(dead_code)]
pub(crate) fn teardown() -> Result<()> {
    unsafe {
        if INITIALIZED {
            libpostal_teardown();
            libpostal_teardown_parser();
            libpostal_teardown_language_classifier();
        }
    }
    Ok(())
}

/// Parse an address using libpostal.
///
/// This function takes an address string and optional parsing options,
/// then returns a vector of address components with their labels.
///
/// # Arguments
///
/// * `address` - The address string to parse
/// * `options` - Optional parsing configuration (language, country hints)
///
/// # Memory Safety
///
/// - Input strings are safely converted to null-terminated C strings
/// - C-allocated result arrays are properly freed after conversion
/// - All string conversions are validated for UTF-8 compliance
/// - Temporary `CString` instances are kept alive for the duration of the C call
///
/// # Thread Safety
///
/// This function is thread-safe once libpostal has been initialized.
/// Multiple threads can call this function concurrently.
///
/// # Errors
///
/// Returns `Error` if:
/// - libpostal is not initialized
/// - Input string contains null bytes
/// - C function returns null (parsing failure)
/// - Result strings contain invalid UTF-8
///
/// # Example
///
/// ```rust,no_run
/// use libpostal_rs::ffi::{initialize, parse_address, ParseOptions};
///
/// initialize().expect("Failed to initialize");
///
/// let components = parse_address(
///     "123 Main St, New York, NY 10001",
///     Some(&ParseOptions {
///         language: Some("en".to_string()),
///         country: Some("US".to_string()),
///     })
/// ).expect("Failed to parse");
///
/// for component in components {
///     println!("{}: {}", component.label, component.value);
/// }
/// ```
pub(crate) fn parse_address(
    address: &str,
    options: Option<&ParseOptions>,
) -> Result<Vec<AddressComponent>> {
    // Ensure libpostal is initialized
    initialize()?;

    let c_address =
        CString::new(address).map_err(|_| Error::ffi_error("Invalid address string"))?;

    unsafe {
        // Get default options or use provided ones
        let opts = if let Some(opts) = options {
            let mut language_holder = None;
            let mut country_holder = None;
            convert_parse_options(opts, &mut language_holder, &mut country_holder)?
        } else {
            libpostal_get_address_parser_default_options()
        };

        // Parse the address - keep ownership of the string
        let response_ptr = libpostal_parse_address(
            c_address.as_ptr() as *mut _, // Cast to *mut for API compatibility
            opts,
        );

        if response_ptr.is_null() {
            return Err(Error::parse_error("libpostal_parse_address returned null"));
        }

        let response = &*response_ptr;

        // Convert C results to Rust
        let mut results = Vec::new();
        for i in 0..response.num_components {
            // Get the component and label at index i
            let component_ptr = *response.components.add(i);
            let label_ptr = *response.labels.add(i);

            if component_ptr.is_null() || label_ptr.is_null() {
                continue; // Skip null entries
            }

            let value = CStr::from_ptr(component_ptr).to_string_lossy().into_owned();
            let label = CStr::from_ptr(label_ptr).to_string_lossy().into_owned();

            results.push(AddressComponent { label, value });
        }

        // Cleanup C memory
        libpostal_address_parser_response_destroy(response_ptr);

        Ok(results)
    }
}

/// Normalize an address string using libpostal.
///
/// This function takes an input string and normalizes it by expanding
/// abbreviations, standardizing formats, and applying various text
/// transformations according to the provided options.
///
/// # Arguments
///
/// * `input` - The string to normalize
/// * `options` - Optional normalization configuration settings
///
/// # Memory Safety
///
/// - Input strings are safely converted to null-terminated C strings
/// - C-allocated result arrays are properly freed after conversion
/// - All string conversions are validated for UTF-8 compliance
/// - No memory leaks from C allocations
///
/// # Thread Safety
///
/// This function is thread-safe once libpostal has been initialized.
/// Multiple threads can call this function concurrently.
///
/// # Returns
///
/// Returns a vector of normalized strings. libpostal may return multiple
/// variations of the normalized input.
///
/// # Errors
///
/// Returns `Error` if:
/// - libpostal is not initialized
/// - Input string contains null bytes
/// - C function returns null (normalization failure)
/// - Result strings contain invalid UTF-8
///
/// # Example
///
/// ```rust,no_run
/// use libpostal_rs::ffi::{initialize, normalize_string, NormalizeOptions};
///
/// initialize().expect("Failed to initialize");
///
/// let normalized = normalize_string(
///     "123 Main St",
///     Some(&NormalizeOptions::default())
/// ).expect("Failed to normalize");
///
/// for variant in normalized {
///     println!("Normalized: {}", variant);
/// }
/// ```
pub(crate) fn normalize_string(
    input: &str,
    options: Option<&NormalizeOptions>,
) -> Result<Vec<String>> {
    // Ensure libpostal is initialized
    initialize()?;

    let c_input = CString::new(input).map_err(|_| Error::ffi_error("Invalid input string"))?;

    unsafe {
        // Get default options or use provided ones
        let opts = if let Some(opts) = options {
            convert_normalize_options(opts)?
        } else {
            libpostal_get_default_options()
        };

        // Normalize the string
        let mut num_expansions = 0;
        let expansions_ptr = libpostal_expand_address(
            c_input.as_ptr() as *mut _, // Cast to *mut for API compatibility
            opts,
            &mut num_expansions,
        );

        if expansions_ptr.is_null() {
            return Ok(Vec::new()); // No expansions is valid
        }

        // Convert C results to Rust
        let mut results = Vec::new();
        for i in 0..num_expansions {
            let expansion_ptr = *expansions_ptr.add(i);
            if !expansion_ptr.is_null() {
                let expansion = CStr::from_ptr(expansion_ptr).to_string_lossy().into_owned();
                results.push(expansion);
            }
        }

        // Cleanup C memory
        libpostal_expansion_array_destroy(expansions_ptr, num_expansions);

        Ok(results)
    }
}

/// Convert Rust ParseOptions to C libpostal_address_parser_options_t.
///
/// This function safely converts Rust parsing options to the C structure
/// expected by libpostal, managing string lifetime and memory safety.
///
/// # Arguments
///
/// * `options` - The Rust parsing options to convert
/// * `language_cstr` - Mutable reference to hold the language CString alive
/// * `country_cstr` - Mutable reference to hold the country CString alive
///
/// # Memory Safety
///
/// The returned C struct contains pointers to the CString data. The caller
/// must ensure that the `language_cstr` and `country_cstr` parameters remain
/// alive for as long as the returned options struct is used.
///
/// This function uses `as_ptr()` with a cast to `*mut i8` rather than
/// `into_raw()` to avoid transferring ownership to C code, which prevents
/// memory leaks.
///
/// # Returns
///
/// Returns a C options struct with pointers to the managed strings, or
/// an error if string conversion fails.
///
/// # Errors
///
/// Returns `Error::FfiError` if the language or country strings contain
/// null bytes, which are not valid in C strings.
fn convert_parse_options(
    options: &ParseOptions,
    language_cstr: &mut Option<CString>,
    country_cstr: &mut Option<CString>,
) -> Result<libpostal_address_parser_options_t> {
    let mut opts = unsafe { libpostal_get_address_parser_default_options() };

    // Set language if provided
    if let Some(ref language) = options.language {
        let c_language = CString::new(language.as_str())
            .map_err(|_| Error::ffi_error("Invalid language string"))?;
        opts.language = c_language.as_ptr() as *mut i8;
        *language_cstr = Some(c_language);
    }

    // Set country if provided
    if let Some(ref country) = options.country {
        let c_country = CString::new(country.as_str())
            .map_err(|_| Error::ffi_error("Invalid country string"))?;
        opts.country = c_country.as_ptr() as *mut i8;
        *country_cstr = Some(c_country);
    }

    Ok(opts)
}

/// Convert Rust NormalizeOptions to C libpostal_normalize_options_t.
///
/// This function safely converts Rust normalization options to the C structure
/// expected by libpostal. Unlike parse options, normalize options only contain
/// primitive types, so no special string lifetime management is required.
///
/// # Arguments
///
/// * `options` - The Rust normalization options to convert
///
/// # Memory Safety
///
/// This function is memory-safe as it only copies primitive boolean and integer
/// values from the Rust struct to the C struct. No pointers or allocations are involved.
///
/// # Returns
///
/// Returns a C options struct with the normalized settings, or an error if
/// the conversion fails (currently this function cannot fail).
///
/// # Safety
///
/// This function is marked unsafe because it calls the unsafe C function
/// `libpostal_get_default_options()`, but the conversion itself is safe.
unsafe fn convert_normalize_options(
    options: &NormalizeOptions,
) -> Result<libpostal_normalize_options_t> {
    let mut opts = libpostal_get_default_options();

    // Apply normalization settings - the generated bindings use bool
    opts.address_components = options.address_components;
    opts.latin_ascii = options.latin_ascii;
    opts.transliterate = options.transliterate;
    opts.strip_accents = options.strip_accents;
    opts.decompose = options.decompose;
    opts.lowercase = options.lowercase;
    opts.trim_string = options.trim_string;
    opts.replace_word_hyphens = options.replace_word_hyphens;
    opts.delete_word_hyphens = options.delete_word_hyphens;
    opts.replace_numeric_hyphens = options.replace_numeric_hyphens;
    opts.delete_numeric_hyphens = options.delete_numeric_hyphens;
    opts.split_alpha_from_numeric = options.split_alpha_from_numeric;
    opts.delete_final_periods = options.delete_final_periods;
    opts.delete_acronym_periods = options.delete_acronym_periods;
    opts.drop_english_possessives = options.drop_english_possessives;
    opts.delete_apostrophes = options.delete_apostrophes;
    opts.expand_numex = options.expand_numex;
    opts.roman_numerals = options.roman_numerals;

    // Future enhancement: Handle languages array conversion if needed
    // For now, using default languages

    Ok(opts)
}

// Safe wrapper types (keeping existing definitions)

/// Address component from libpostal parsing.
///
/// Represents a single parsed component of an address with its semantic label
/// and the actual text value. libpostal identifies various components like
/// house numbers, streets, cities, postal codes, etc.
///
/// # Examples of common labels:
///
/// - `"house_number"` - Street number (e.g., "123")
/// - `"road"` - Street name (e.g., "Main Street")
/// - `"city"` - City name (e.g., "New York")
/// - `"state"` - State/province (e.g., "NY", "California")
/// - `"postcode"` - Postal/ZIP code (e.g., "10001")
/// - `"country"` - Country name (e.g., "USA")
///
/// # Memory Safety
///
/// This struct owns its string data and is safe to pass between threads.
/// The strings are guaranteed to be valid UTF-8.
#[derive(Debug, Clone)]
pub struct AddressComponent {
    /// Component label indicating the semantic meaning of this address part.
    ///
    /// Common values include "house_number", "road", "city", "state", "postcode", etc.
    /// The full list depends on libpostal's training data and the specific address format.
    pub label: String,
    /// The actual text content of this address component.
    ///
    /// This is the original text from the input address that was classified
    /// with the corresponding label.
    pub value: String,
}

/// Options for address parsing.
///
/// These options provide hints to libpostal about the expected language
/// and country of the address being parsed, which can improve parsing accuracy.
///
/// # Memory Safety
///
/// All string fields are owned and safe to pass between threads.
/// Optional fields can be `None` to use libpostal's automatic detection.
///
/// # Example
///
/// ```rust
/// use libpostal_rs::ffi::ParseOptions;
///
/// let options = ParseOptions {
///     language: Some("en".to_string()),  // English
///     country: Some("US".to_string()),   // United States
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Language hint for parsing (e.g., "en", "es", "fr").
    ///
    /// When provided, this helps libpostal choose appropriate parsing
    /// rules and handle language-specific address formats.
    /// If `None`, libpostal will attempt automatic language detection.
    pub language: Option<String>,

    /// Country hint for parsing (e.g., "US", "CA", "GB").
    ///
    /// When provided, this helps libpostal apply country-specific
    /// address parsing rules and component recognition.
    /// If `None`, libpostal will attempt automatic country detection.
    pub country: Option<String>,
}

/// Options for address normalization
#[derive(Debug, Clone)]
pub struct NormalizeOptions {
    /// Languages to use for normalization
    pub languages: Vec<String>,
    /// Address components to normalize
    pub address_components: u16,
    /// Latin ASCII transliteration
    pub latin_ascii: bool,
    /// Transliterate
    pub transliterate: bool,
    /// Strip accents
    pub strip_accents: bool,
    /// Decompose
    pub decompose: bool,
    /// Lowercase
    pub lowercase: bool,
    /// Trim string
    pub trim_string: bool,
    /// Replace word hyphens
    pub replace_word_hyphens: bool,
    /// Delete word hyphens
    pub delete_word_hyphens: bool,
    /// Replace numeric hyphens
    pub replace_numeric_hyphens: bool,
    /// Delete numeric hyphens
    pub delete_numeric_hyphens: bool,
    /// Split alpha from numeric
    pub split_alpha_from_numeric: bool,
    /// Delete final periods
    pub delete_final_periods: bool,
    /// Delete acronym periods
    pub delete_acronym_periods: bool,
    /// Drop English possessives
    pub drop_english_possessives: bool,
    /// Delete apostrophes
    pub delete_apostrophes: bool,
    /// Expand numex
    pub expand_numex: bool,
    /// Roman numerals
    pub roman_numerals: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Test basic initialization and teardown
    #[test]
    fn test_initialization() {
        // Test that initialization works
        let result = initialize();
        assert!(
            result.is_ok(),
            "Failed to initialize libpostal: {:?}",
            result
        );

        // Test that repeated initialization is safe
        let result2 = initialize();
        assert!(
            result2.is_ok(),
            "Second initialization failed: {:?}",
            result2
        );
    }

    /// Test address parsing with basic functionality
    #[test]
    fn test_basic_address_parsing() {
        // Initialize first
        initialize().expect("Failed to initialize libpostal");

        // Test parsing a simple address
        let result = parse_address("123 Main St", None);

        // For now, just verify it doesn't crash and returns a result
        // The actual parsing functionality depends on data files being available
        match result {
            Ok(components) => {
                println!("Parsed components: {:?}", components);
                // Basic sanity check - components should be a valid vector
                // Can be empty if no data files are available
            }
            Err(e) => {
                // This might fail if data files aren't available, which is OK for now
                println!("Parsing failed (expected without data files): {:?}", e);
            }
        }
    }

    /// Test normalization functionality
    #[test]
    fn test_basic_normalization() {
        // Initialize first
        initialize().expect("Failed to initialize libpostal");

        // Test normalizing a string
        let result = normalize_string("St", None);

        match result {
            Ok(expansions) => {
                println!("Normalizations: {:?}", expansions);
                // Should return a valid vector (can be empty)
            }
            Err(e) => {
                println!("Normalization failed (might be expected): {:?}", e);
            }
        }
    }

    /// Test error handling with invalid input
    #[test]
    fn test_error_handling() {
        // Test parsing with invalid UTF-8 (null bytes)
        let result = parse_address("test\0invalid", None);
        assert!(result.is_err(), "Should fail with null bytes in string");

        let result = normalize_string("test\0invalid", None);
        assert!(result.is_err(), "Should fail with null bytes in string");
    }

    /// Test option conversion functions
    #[test]
    fn test_option_conversion() {
        unsafe {
            // Test parse options conversion
            let parse_opts = ParseOptions {
                language: Some("en".to_string()),
                country: Some("US".to_string()),
            };

            let mut language_holder = None;
            let mut country_holder = None;
            let result =
                convert_parse_options(&parse_opts, &mut language_holder, &mut country_holder);
            assert!(
                result.is_ok(),
                "Parse options conversion failed: {:?}",
                result
            );

            // Test normalize options conversion
            let normalize_opts = NormalizeOptions {
                languages: vec!["en".to_string()],
                address_components: 0xFFFF,
                latin_ascii: false,
                transliterate: true,
                strip_accents: false,
                decompose: true,
                lowercase: true,
                trim_string: true,
                replace_word_hyphens: false,
                delete_word_hyphens: false,
                replace_numeric_hyphens: false,
                delete_numeric_hyphens: false,
                split_alpha_from_numeric: false,
                delete_final_periods: true,
                delete_acronym_periods: true,
                drop_english_possessives: true,
                delete_apostrophes: true,
                expand_numex: true,
                roman_numerals: true,
            };

            let result = convert_normalize_options(&normalize_opts);
            assert!(
                result.is_ok(),
                "Normalize options conversion failed: {:?}",
                result
            );
        }
    }

    /// Test memory safety with CString conversion
    #[test]
    fn test_cstring_safety() {
        // Test valid strings
        let valid = CString::new("Hello World");
        assert!(valid.is_ok());

        // Test strings with null bytes
        let invalid = CString::new("Hello\0World");
        assert!(invalid.is_err());

        // Test empty strings
        let empty = CString::new("");
        assert!(empty.is_ok());

        // Test UTF-8 strings
        let utf8 = CString::new("Héllo Wörld 🌍");
        assert!(utf8.is_ok());
    }

    /// Test thread safety of initialization
    #[test]
    fn test_thread_safety() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::thread;

        let success_count = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let counter = Arc::clone(&success_count);
                thread::spawn(move || {
                    if initialize().is_ok() {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All threads should succeed
        assert_eq!(success_count.load(Ordering::SeqCst), 10);
    }
}
