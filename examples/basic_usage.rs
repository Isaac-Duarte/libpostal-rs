//! Basic usage example for libpostal-rs.
//!
//! This example demonstrates the core functionality of the library:
//! - Initializing LibPostal
//! - Parsing addresses into structured components  
//! - Normalizing addresses for comparison
//!
//! Run with: cargo run --example basic_usage

use libpostal_rs::{Error, LibPostal};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("libpostal-rs Basic Usage Example");
    println!("================================\n");

    // Initialize LibPostal with default configuration
    println!("Initializing LibPostal...");
    let postal = LibPostal::new().await?;
    println!("LibPostal initialized successfully\n");

    // Example 1: Basic address parsing
    println!("1. Basic Address Parsing");
    println!("-----------------------");

    let address = "123 Main Street, New York, NY 10001, USA";
    println!("Input: {}", address);

    let parsed = postal.parse_address(address)?;
    println!("Parsed components:");
    if let Some(house_number) = &parsed.house_number {
        println!("  House Number: {}", house_number);
    }
    if let Some(road) = &parsed.road {
        println!("  Road: {}", road);
    }
    if let Some(city) = &parsed.city {
        println!("  City: {}", city);
    }
    if let Some(state) = &parsed.state {
        println!("  State: {}", state);
    }
    if let Some(postcode) = &parsed.postcode {
        println!("  Postcode: {}", postcode);
    }
    if let Some(country) = &parsed.country {
        println!("  Country: {}", country);
    }
    println!();

    // Example 2: Address parsing with hints
    println!("2. Address Parsing with Language/Country Hints");
    println!("----------------------------------------------");

    let french_address = "123 Rue de la Paix, Paris, 75001, France";
    println!("Input: {}", french_address);

    let parsed_fr = postal.parse_address_with_hints(
        french_address,
        Some("fr"), // French language hint
        Some("FR"), // France country hint
    )?;

    println!("Parsed components:");
    if let Some(house_number) = &parsed_fr.house_number {
        println!("  House Number: {}", house_number);
    }
    if let Some(road) = &parsed_fr.road {
        println!("  Road: {}", road);
    }
    if let Some(city) = &parsed_fr.city {
        println!("  City: {}", city);
    }
    if let Some(postcode) = &parsed_fr.postcode {
        println!("  Postcode: {}", postcode);
    }
    if let Some(country) = &parsed_fr.country {
        println!("  Country: {}", country);
    }
    println!();

    // Example 3: Address normalization
    println!("3. Address Normalization");
    println!("-----------------------");

    let unnormalized = "123 Main St";
    println!("Input: {}", unnormalized);

    let normalized = postal.normalize_address(unnormalized)?;
    println!("Normalized expansions:");
    for (i, expansion) in normalized.expansions.iter().enumerate() {
        println!("  {}: {}", i + 1, expansion);
    }
    println!();

    // Example 4: Using the builder pattern for advanced parsing
    println!("4. Advanced Parser Configuration");
    println!("-------------------------------");

    let parser = postal
        .parser()
        .with_language(libpostal_rs::types::Language::English)
        .with_country(libpostal_rs::types::Country::UnitedStates);

    let advanced_address = "Apt 5B, 456 Oak Avenue, San Francisco, California 94102";
    println!("Input: {}", advanced_address);

    let parsed_advanced = parser.parse(advanced_address)?;
    println!("Parsed components:");
    if let Some(unit) = &parsed_advanced.unit {
        println!("  Unit: {}", unit);
    }
    if let Some(house_number) = &parsed_advanced.house_number {
        println!("  House Number: {}", house_number);
    }
    if let Some(road) = &parsed_advanced.road {
        println!("  Road: {}", road);
    }
    if let Some(city) = &parsed_advanced.city {
        println!("  City: {}", city);
    }
    if let Some(state) = &parsed_advanced.state {
        println!("  State: {}", state);
    }
    if let Some(postcode) = &parsed_advanced.postcode {
        println!("  Postcode: {}", postcode);
    }
    println!();

    // Example 5: Advanced normalization with different levels
    println!("5. Advanced Normalization");
    println!("------------------------");

    let normalizer = postal
        .normalizer()
        .with_level(libpostal_rs::types::NormalizationLevel::Aggressive);

    let complex_address = "One Hundred Twenty-Third St.";
    println!("Input: {}", complex_address);

    let normalized_aggressive = normalizer.normalize(complex_address)?;
    println!("Aggressively normalized:");
    for (i, expansion) in normalized_aggressive.expansions.iter().enumerate() {
        println!("  {}: {}", i + 1, expansion);
    }
    println!();

    // Example 6: Batch processing
    println!("6. Batch Processing");
    println!("------------------");

    let addresses = vec![
        "123 Main St, Boston, MA",
        "456 Oak Ave, Portland, OR",
        "789 Pine Rd, Austin, TX",
    ];

    println!("Processing {} addresses in batch:", addresses.len());
    let batch_results = parser.parse_batch(&addresses)?;

    for (i, result) in batch_results.iter().enumerate() {
        println!(
            "  Address {}: {} -> City: {}",
            i + 1,
            addresses[i],
            result.city.as_deref().unwrap_or("Unknown")
        );
    }
    println!();

    println!("All examples completed successfully!");

    Ok(())
}
