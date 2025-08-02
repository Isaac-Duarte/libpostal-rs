//! Advanced usage example for libpostal-rs.
//!
//! This example demonstrates advanced features:
//! - Custom configuration
//! - Error handling patterns
//! - Performance optimization
//! - Integration patterns
//!
//! Run with: cargo run --example advanced_usage

use libpostal_rs::{Error, LibPostal, LibPostalConfig};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("libpostal-rs Advanced Usage Example");
    println!("===================================\n");

    // Example 1: Custom configuration with default data directory
    println!("1. Custom Configuration");
    println!("----------------------");

    let config = LibPostalConfig::builder()
        .auto_download_data(true)
        .verify_data_integrity(true)
        .build();

    println!("Initializing with custom configuration...");
    let postal = LibPostal::with_config(config).await?;
    println!("✓ Custom LibPostal initialized\n");

    // Example 2: Performance measurement
    println!("2. Performance Measurement");
    println!("-------------------------");

    let test_addresses = vec![
        "123 Main Street, New York, NY 10001",
        "456 Oak Avenue, Los Angeles, CA 90210",
        "789 Pine Road, Chicago, IL 60601",
        "321 Elm Street, Houston, TX 77001",
        "654 Maple Drive, Phoenix, AZ 85001",
    ];

    // Single address parsing performance
    let start = Instant::now();
    for address in &test_addresses {
        let _parsed = postal.parse_address(address)?;
    }
    let single_duration = start.elapsed();
    println!(
        "Single parsing: {} addresses in {:?}",
        test_addresses.len(),
        single_duration
    );

    // Batch parsing performance
    let start = Instant::now();
    let address_refs: Vec<&str> = test_addresses.iter().map(|s| s.as_ref()).collect();
    let _batch_results = postal.parser().parse_batch(&address_refs)?;
    let batch_duration = start.elapsed();
    println!(
        "Batch parsing: {} addresses in {:?}",
        test_addresses.len(),
        batch_duration
    );

    println!(
        "Performance improvement: {:.2}x faster\n",
        single_duration.as_nanos() as f64 / batch_duration.as_nanos() as f64
    );

    // Example 3: Comprehensive address analysis
    println!("3. Comprehensive Address Analysis");
    println!("---------------------------------");

    let complex_address = "Apartment 12B, 1600 Pennsylvania Avenue NW, Washington, District of Columbia 20500, United States of America";
    println!("Analyzing: {}", complex_address);

    // Parse the address
    let parsed = postal.parse_address(complex_address)?;
    println!("\nParsing results:");
    println!("  Components found: {}", parsed.components().len());
    println!("  Is empty: {}", parsed.is_empty());

    // Show all components
    for (label, value) in parsed.components() {
        println!("  {}: {}", label, value);
    }

    // Normalize the address with different levels
    println!("\nNormalization results:");

    let light_normalizer = postal
        .normalizer()
        .with_level(libpostal_rs::types::NormalizationLevel::Light);
    let light_normalized = light_normalizer.normalize(complex_address)?;
    println!(
        "  Light normalization ({} variants):",
        light_normalized.len()
    );
    for variant in light_normalized.expansions.iter().take(3) {
        println!("    - {}", variant);
    }

    let aggressive_normalizer = postal
        .normalizer()
        .with_level(libpostal_rs::types::NormalizationLevel::Aggressive);
    let aggressive_normalized = aggressive_normalizer.normalize(complex_address)?;
    println!(
        "  Aggressive normalization ({} variants):",
        aggressive_normalized.len()
    );
    for variant in aggressive_normalized.expansions.iter().take(3) {
        println!("    - {}", variant);
    }
    println!();

    // Example 4: International addresses
    println!("4. International Address Handling");
    println!("--------------------------------");

    let international_addresses = vec![
        (
            "German",
            "de",
            "DE",
            "Musterstraße 123, 12345 Berlin, Deutschland",
        ),
        (
            "French",
            "fr",
            "FR",
            "123 rue de la Paix, 75001 Paris, France",
        ),
        ("Japanese", "ja", "JP", "東京都渋谷区神南1-2-3"),
        (
            "Spanish",
            "es",
            "ES",
            "Calle Mayor 123, 28013 Madrid, España",
        ),
    ];

    for (language_name, lang_code, country_code, address) in international_addresses {
        println!("{} address: {}", language_name, address);

        match postal.parse_address_with_hints(address, Some(lang_code), Some(country_code)) {
            Ok(parsed) => {
                println!(
                    "  ✓ Parsed successfully ({} components)",
                    parsed.components().len()
                );
                if let Some(city) = &parsed.city {
                    println!("    City: {}", city);
                }
                if let Some(country) = &parsed.country {
                    println!("    Country: {}", country);
                }
            }
            Err(e) => {
                println!("  ✗ Parse failed: {}", e);
            }
        }
        println!();
    }

    // Example 5: Error handling patterns
    println!("5. Error Handling Patterns");
    println!("-------------------------");

    // Graceful handling of problematic inputs
    let long_string = "x".repeat(10000);
    let problematic_inputs = vec![
        "",           // Empty string
        "   ",        // Whitespace only
        "123",        // Too short
        &long_string, // Very long string
    ];

    for input in problematic_inputs {
        let display_input = if input.len() > 50 {
            format!("{}... ({} chars)", &input[..50], input.len())
        } else {
            input.to_string()
        };

        match postal.parse_address(&input) {
            Ok(parsed) => {
                if parsed.is_empty() {
                    println!("  '{}' -> No components found", display_input);
                } else {
                    println!(
                        "  '{}' -> {} components",
                        display_input,
                        parsed.components().len()
                    );
                }
            }
            Err(e) => {
                println!("  '{}' -> Error: {}", display_input, e);
            }
        }
    }
    println!();

    // Example 6: Configuration checking
    println!("6. Configuration Information");
    println!("---------------------------");

    let config = postal.config();
    println!("LibPostal configuration:");
    println!("  Auto download data: {}", config.auto_download_data);
    println!("  Verify data integrity: {}", config.verify_data_integrity);
    println!("  Data directory: {:?}", config.data_config.data_dir);
    println!("  Auto download: {}", config.data_config.auto_download);
    println!(
        "  Verify integrity: {}",
        config.data_config.verify_integrity
    );
    println!();

    println!("✓ All advanced examples completed successfully!");

    Ok(())
}
