//! Address parsing functionality.

use crate::error::Result;
use crate::ffi::{self, AddressComponent, ParseOptions};
use crate::types::{AddressHint, Country, Language};

/// High-level address parser with idiomatic Rust API.
#[derive(Debug)]
pub struct AddressParser {
    options: ParseOptions,
}

impl AddressParser {
    /// Create a new parser with default options.
    pub fn new() -> Self {
        Self {
            options: ParseOptions {
                language: None,
                country: None,
            },
        }
    }

    /// Set language hint for parsing.
    pub fn with_language(mut self, language: Language) -> Self {
        self.options.language = Some(language.to_string());
        self
    }

    /// Set country hint for parsing.
    pub fn with_country(mut self, country: Country) -> Self {
        self.options.country = Some(country.to_string());
        self
    }

    /// Set multiple hints for parsing.
    pub fn with_hints(mut self, hints: &AddressHint) -> Self {
        if let Some(ref language) = hints.language {
            self.options.language = Some(language.to_string());
        }
        if let Some(ref country) = hints.country {
            self.options.country = Some(country.to_string());
        }
        self
    }

    /// Parse an address string into structured components.
    ///
    /// # Arguments
    ///
    /// * `address` - The address string to parse
    ///
    /// # Returns
    ///
    /// A `ParsedAddress` containing the structured components.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or if the address string is invalid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use libpostal_rs::AddressParser;
    ///
    /// let parser = AddressParser::new();
    /// let parsed = parser.parse("123 Main St, New York, NY 10001")?;
    /// println!("House number: {}", parsed.house_number.unwrap_or_default());
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    pub fn parse(&self, address: &str) -> Result<ParsedAddress> {
        let components = ffi::parse_address(address, Some(&self.options))?;
        ParsedAddress::from_components(components)
    }

    /// Parse multiple addresses in batch for better performance.
    pub fn parse_batch(&self, addresses: &[&str]) -> Result<Vec<ParsedAddress>> {
        addresses.iter().map(|addr| self.parse(addr)).collect()
    }

    /// Parse multiple addresses in parallel using multiple threads.
    ///
    /// This method is more efficient for large batches of addresses as it
    /// utilizes multiple CPU cores. The parsing is done in parallel chunks.
    ///
    /// # Arguments
    ///
    /// * `addresses` - Slice of address strings to parse
    ///
    /// # Returns
    ///
    /// A vector of `ParsedAddress` results in the same order as the input.
    /// Failed parses will be returned as errors in the result vector.
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe. Each thread gets its own parser instance
    /// with the same options as the original parser.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use libpostal_rs::AddressParser;
    ///
    /// let parser = AddressParser::new();
    /// let addresses = vec![
    ///     "123 Main St, New York, NY",
    ///     "456 Oak Ave, Los Angeles, CA",
    ///     "789 Pine Rd, Chicago, IL",
    /// ];
    ///
    /// let results = parser.parse_batch_parallel(&addresses)?;
    /// for result in results {
    ///     match result {
    ///         Ok(parsed) => println!("Parsed: {:?}", parsed.city),
    ///         Err(e) => println!("Error: {}", e),
    ///     }
    /// }
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    #[cfg(feature = "parallel")]
    pub fn parse_batch_parallel(&self, addresses: &[&str]) -> Result<Vec<Result<ParsedAddress>>> {
        use rayon::prelude::*;

        Ok(addresses
            .par_iter()
            .map(|addr| {
                // Each thread gets its own parser with the same options
                let parser = AddressParser {
                    options: self.options.clone(),
                };
                parser.parse(addr)
            })
            .collect())
    }

    /// Parse multiple addresses in parallel and return only successful results.
    ///
    /// This is a convenience method that filters out any parsing errors and
    /// returns only the successfully parsed addresses.
    ///
    /// # Arguments
    ///
    /// * `addresses` - Slice of address strings to parse
    ///
    /// # Returns
    ///
    /// A vector of successfully parsed `ParsedAddress` instances.
    /// Failed parses are silently ignored.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use libpostal_rs::AddressParser;
    ///
    /// let parser = AddressParser::new();
    /// let addresses = vec![
    ///     "123 Main St, New York, NY",
    ///     "", // This will fail but be filtered out
    ///     "456 Oak Ave, Los Angeles, CA",
    /// ];
    ///
    /// let successful_results = parser.parse_batch_parallel_ok(&addresses)?;
    /// println!("Successfully parsed {} addresses", successful_results.len());
    /// # Ok::<(), libpostal_rs::Error>(())
    /// ```
    #[cfg(feature = "parallel")]
    pub fn parse_batch_parallel_ok(&self, addresses: &[&str]) -> Result<Vec<ParsedAddress>> {
        let results = self
            .parse_batch_parallel(addresses)?
            .into_iter()
            .filter_map(|result| result.ok())
            .collect::<Vec<_>>();
        Ok(results)
    }
}

impl Default for AddressParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured representation of a parsed address.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default)]
pub struct ParsedAddress {
    /// House number (e.g., "123", "123A")
    pub house_number: Option<String>,
    /// Road/street name (e.g., "Main St", "Broadway")
    pub road: Option<String>,
    /// Unit/apartment number (e.g., "Apt 2B", "Unit 5")
    pub unit: Option<String>,
    /// Floor/level (e.g., "2nd Floor", "Floor 3")
    pub level: Option<String>,
    /// Staircase
    pub staircase: Option<String>,
    /// Entrance
    pub entrance: Option<String>,
    /// Post office box
    pub po_box: Option<String>,
    /// Postcode (e.g., "10001", "SW1A 1AA")
    pub postcode: Option<String>,
    /// Suburb/neighborhood
    pub suburb: Option<String>,
    /// City/locality (e.g., "New York", "London")
    pub city: Option<String>,
    /// City district
    pub city_district: Option<String>,
    /// Island
    pub island: Option<String>,
    /// State/province (e.g., "NY", "California", "Ontario")
    pub state: Option<String>,
    /// State district
    pub state_district: Option<String>,
    /// Country region
    pub country_region: Option<String>,
    /// Country (e.g., "USA", "United States")
    pub country: Option<String>,
    /// World region
    pub world_region: Option<String>,
    /// Category (e.g., building type)
    pub category: Option<String>,
    /// Near location reference
    pub near: Option<String>,
    /// Toponym (place name)
    pub toponym: Option<String>,
    /// All other unclassified components
    pub other: Vec<String>,
}

impl ParsedAddress {
    /// Create a ParsedAddress from raw FFI components.
    pub(crate) fn from_components(components: Vec<AddressComponent>) -> Result<Self> {
        let mut parsed = ParsedAddress::default();

        for component in components {
            match component.label.as_str() {
                "house_number" => parsed.house_number = Some(component.value),
                "road" => parsed.road = Some(component.value),
                "unit" => parsed.unit = Some(component.value),
                "level" => parsed.level = Some(component.value),
                "staircase" => parsed.staircase = Some(component.value),
                "entrance" => parsed.entrance = Some(component.value),
                "po_box" => parsed.po_box = Some(component.value),
                "postcode" => parsed.postcode = Some(component.value),
                "suburb" => parsed.suburb = Some(component.value),
                "city" => parsed.city = Some(component.value),
                "city_district" => parsed.city_district = Some(component.value),
                "island" => parsed.island = Some(component.value),
                "state" => parsed.state = Some(component.value),
                "state_district" => parsed.state_district = Some(component.value),
                "country_region" => parsed.country_region = Some(component.value),
                "country" => parsed.country = Some(component.value),
                "world_region" => parsed.world_region = Some(component.value),
                "category" => parsed.category = Some(component.value),
                "near" => parsed.near = Some(component.value),
                "toponym" => parsed.toponym = Some(component.value),
                _ => parsed.other.push(component.value),
            }
        }

        Ok(parsed)
    }

    /// Get all non-empty components as a map.
    pub fn components(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();

        macro_rules! add_component {
            ($field:expr, $name:expr) => {
                if let Some(ref value) = $field {
                    map.insert($name.to_string(), value.clone());
                }
            };
        }

        add_component!(self.house_number, "house_number");
        add_component!(self.road, "road");
        add_component!(self.unit, "unit");
        add_component!(self.level, "level");
        add_component!(self.staircase, "staircase");
        add_component!(self.entrance, "entrance");
        add_component!(self.po_box, "po_box");
        add_component!(self.postcode, "postcode");
        add_component!(self.suburb, "suburb");
        add_component!(self.city, "city");
        add_component!(self.city_district, "city_district");
        add_component!(self.island, "island");
        add_component!(self.state, "state");
        add_component!(self.state_district, "state_district");
        add_component!(self.country_region, "country_region");
        add_component!(self.country, "country");
        add_component!(self.world_region, "world_region");
        add_component!(self.category, "category");
        add_component!(self.near, "near");
        add_component!(self.toponym, "toponym");

        for (i, value) in self.other.iter().enumerate() {
            map.insert(format!("other_{i}"), value.clone());
        }

        map
    }

    /// Check if the parsed address has any components.
    pub fn is_empty(&self) -> bool {
        self.house_number.is_none()
            && self.road.is_none()
            && self.unit.is_none()
            && self.level.is_none()
            && self.staircase.is_none()
            && self.entrance.is_none()
            && self.po_box.is_none()
            && self.postcode.is_none()
            && self.suburb.is_none()
            && self.city.is_none()
            && self.city_district.is_none()
            && self.island.is_none()
            && self.state.is_none()
            && self.state_district.is_none()
            && self.country_region.is_none()
            && self.country.is_none()
            && self.world_region.is_none()
            && self.category.is_none()
            && self.near.is_none()
            && self.toponym.is_none()
            && self.other.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_address_default() {
        let parsed = ParsedAddress::default();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parsed_address_components() {
        let mut parsed = ParsedAddress::default();
        parsed.house_number = Some("123".to_string());
        parsed.road = Some("Main St".to_string());

        let components = parsed.components();
        assert_eq!(components.get("house_number"), Some(&"123".to_string()));
        assert_eq!(components.get("road"), Some(&"Main St".to_string()));
        assert!(!parsed.is_empty());
    }
}
