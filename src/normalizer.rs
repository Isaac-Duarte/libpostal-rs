//! Address normalization functionality.

use crate::error::Result;
use crate::ffi::{self, NormalizeOptions};
use crate::types::{Language, NormalizationLevel};

/// High-level address normalizer with builder pattern.
#[derive(Debug)]
pub struct AddressNormalizer {
    options: NormalizeOptions,
}

impl AddressNormalizer {
    /// Create a new normalizer with default options.
    pub fn new() -> Self {
        Self {
            options: NormalizeOptions {
                languages: vec!["en".to_string()],
                address_components: 0xFFFF, // All components
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
            },
        }
    }

    /// Set languages for normalization.
    pub fn with_languages(mut self, languages: &[Language]) -> Self {
        self.options.languages = languages.iter().map(|l| l.to_string()).collect();
        self
    }

    /// Set normalization level (controls which transformations are applied).
    pub fn with_level(mut self, level: NormalizationLevel) -> Self {
        match level {
            NormalizationLevel::Light => {
                self.options.lowercase = true;
                self.options.trim_string = true;
                self.options.delete_final_periods = true;
                self.options.transliterate = false;
                self.options.decompose = false;
            }
            NormalizationLevel::Medium => {
                self.options.lowercase = true;
                self.options.trim_string = true;
                self.options.delete_final_periods = true;
                self.options.delete_acronym_periods = true;
                self.options.transliterate = true;
                self.options.decompose = true;
            }
            NormalizationLevel::Aggressive => {
                self.options.lowercase = true;
                self.options.trim_string = true;
                self.options.delete_final_periods = true;
                self.options.delete_acronym_periods = true;
                self.options.drop_english_possessives = true;
                self.options.delete_apostrophes = true;
                self.options.transliterate = true;
                self.options.decompose = true;
                self.options.expand_numex = true;
            }
        }
        self
    }

    /// Enable/disable Latin ASCII transliteration.
    pub fn with_latin_ascii(mut self, enabled: bool) -> Self {
        self.options.latin_ascii = enabled;
        self
    }

    /// Enable/disable lowercasing.
    pub fn with_lowercase(mut self, enabled: bool) -> Self {
        self.options.lowercase = enabled;
        self
    }

    /// Normalize an address string.
    ///
    /// # Arguments
    ///
    /// * `input` - The address string to normalize
    ///
    /// # Returns
    ///
    /// A `NormalizedAddress` containing all possible normalizations.
    ///
    /// # Errors
    ///
    /// Returns an error if normalization fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use libpostal_rs::AddressNormalizer;
    ///
    /// let normalizer = AddressNormalizer::new();
    /// let normalized = normalizer.normalize("Thirty-Fourth St")?;
    /// println!("Expansions: {:?}", normalized.expansions);
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn normalize(&self, input: &str) -> Result<NormalizedAddress> {
        let expansions = ffi::normalize_string(input, Some(&self.options))?;
        Ok(NormalizedAddress {
            original: input.to_string(),
            expansions,
        })
    }

    /// Normalize multiple address strings in batch.
    pub fn normalize_batch(&self, inputs: &[&str]) -> Result<Vec<NormalizedAddress>> {
        inputs.iter().map(|input| self.normalize(input)).collect()
    }
}

impl Default for AddressNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of address normalization containing all possible expansions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NormalizedAddress {
    /// Original input string
    pub original: String,
    /// All possible normalized expansions
    pub expansions: Vec<String>,
}

impl NormalizedAddress {
    /// Get the first (most likely) expansion.
    pub fn primary(&self) -> Option<&str> {
        self.expansions.first().map(|s| s.as_str())
    }

    /// Get all expansions except the first.
    pub fn alternatives(&self) -> &[String] {
        if self.expansions.len() > 1 {
            &self.expansions[1..]
        } else {
            &[]
        }
    }

    /// Check if normalization produced any results.
    pub fn is_empty(&self) -> bool {
        self.expansions.is_empty()
    }

    /// Get the number of expansions.
    pub fn len(&self) -> usize {
        self.expansions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_address() {
        let normalized = NormalizedAddress {
            original: "St".to_string(),
            expansions: vec!["street".to_string(), "saint".to_string()],
        };

        assert_eq!(normalized.primary(), Some("street"));
        assert_eq!(normalized.alternatives(), &["saint".to_string()]);
        assert!(!normalized.is_empty());
        assert_eq!(normalized.len(), 2);
    }

    #[test]
    fn test_empty_normalized_address() {
        let normalized = NormalizedAddress {
            original: "test".to_string(),
            expansions: vec![],
        };

        assert_eq!(normalized.primary(), None);
        assert_eq!(normalized.alternatives(), &[] as &[String]);
        assert!(normalized.is_empty());
        assert_eq!(normalized.len(), 0);
    }
}
