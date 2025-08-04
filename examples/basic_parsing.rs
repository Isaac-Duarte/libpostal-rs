//! Basic address parsing example
//!
//! This example demonstrates how to parse a simple address
//! into its component parts.

use libpostal_rs::{AddressParser, Language, LibPostal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("libpostal-rs Basic Parsing Example");
    println!("==================================");

    println!("Initializing libpostal...");

    match LibPostal::new().await {
        Ok(_postal) => {
            println!("libpostal initialized successfully");

            // Example addresses to parse
            let addresses = vec![
                "123 Main Street, New York, NY 10001",
                "221B Baker Street, London, UK",
                "1600 Pennsylvania Avenue NW, Washington, DC 20500",
                "Champ de Mars, 5 Avenue Anatole France, 75007 Paris, France",
            ];

            // Create parser with English language hint
            let parser = AddressParser::new().with_language(Language::English);

            println!("\nParsing addresses:");
            println!("-----------------");

            for address in addresses {
                println!("\nOriginal: {}", address);

                match parser.parse(address) {
                    Ok(parsed) => {
                        println!(
                            "  House number: {}",
                            parsed.house_number.unwrap_or_default()
                        );
                        println!("  Street: {}", parsed.road.unwrap_or_default());
                        println!("  City: {}", parsed.city.unwrap_or_default());
                        println!("  State: {}", parsed.state.unwrap_or_default());
                        println!("  Postcode: {}", parsed.postcode.unwrap_or_default());
                        println!("  Country: {}", parsed.country.unwrap_or_default());
                    }
                    Err(e) => {
                        println!("  Error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to initialize libpostal: {}", e);
            println!("Note: This requires libpostal data files to be available.");
            println!("Run the data setup script or install libpostal system-wide.");
        }
    }

    println!("\nExample complete!");
    Ok(())
}
