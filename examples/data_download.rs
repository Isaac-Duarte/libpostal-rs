//! Example demonstrating how to download libpostal data using the embedded libpostal_data command.
//!
//! This example shows how to:
//! 1. Use the embedded libpostal_data executable that gets built during compilation
//! 2. Download the required data files for libpostal to function
//! 3. Verify that the data was downloaded correctly
//!
//! Run with: cargo run --example data_download --features runtime-data

use libpostal_rs::{data::DataManager, LibPostal, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== libpostal-rs Data Download Example ===\n");

    // Create a data manager
    let data_manager = DataManager::new();
    
    println!("Data directory: {}", data_manager.data_dir().display());
    println!("Data available: {}\n", data_manager.is_data_available());

    // Copy the libpostal_data executable to the project root for easy access
    println!("1. Copying libpostal_data executable to project root...");
    match data_manager.copy_libpostal_data_to_root() {
        Ok(()) => println!("   Successfully copied libpostal_data executable"),
        Err(e) => println!("   Failed to copy libpostal_data: {}", e),
    }
    println!();

    // Check if data is already available
    if data_manager.is_data_available() {
        println!("2. Data files are already available!");
        
        // Verify the data
        match data_manager.verify_data() {
            Ok(()) => println!("   Data verification passed"),
            Err(e) => {
                println!("   Data verification failed: {}", e);
                println!("   You may need to re-download the data files.");
            }
        }
        
        // Show data size
        match data_manager.data_size() {
            Ok(size) => println!("   Data size: {:.1} MB", size as f64 / 1024.0 / 1024.0),
            Err(e) => println!("   Could not calculate data size: {}", e),
        }
    } else {
        println!("2. Data files not found. Downloading...");
        println!("   This will take several minutes as the data files are large (~1GB)");
        
        // Download the data using the embedded libpostal_data command
        match data_manager.ensure_data().await {
            Ok(()) => {
                println!("   Data download completed successfully!");
                
                // Verify the downloaded data
                match data_manager.verify_data() {
                    Ok(()) => println!("   Data verification passed"),
                    Err(e) => println!("   Data verification failed: {}", e),
                }
                
                // Show final data size
                match data_manager.data_size() {
                    Ok(size) => println!("   Final data size: {:.1} MB", size as f64 / 1024.0 / 1024.0),
                    Err(e) => println!("   Could not calculate data size: {}", e),
                }
            }
            Err(e) => {
                println!("   Data download failed: {}", e);
                println!("   Please check your internet connection and try again.");
                return Err(e);
            }
        }
    }

    println!("\n3. Testing libpostal functionality...");
    
    // Test that libpostal can now be initialized and used
    match test_libpostal_functionality().await {
        Ok(()) => println!("   libpostal is working correctly!"),
        Err(e) => {
            println!("   libpostal test failed: {}", e);
            println!("   The data may not be complete or there may be a configuration issue.");
        }
    }

    println!("\n=== Data download example completed ===");
    Ok(())
}

async fn test_libpostal_functionality() -> Result<()> {
    // Test that we can initialize LibPostal now that data is available
    let postal = LibPostal::new().await?;
    
    // Test address normalization
    let normalized = postal.normalize_address("123 Main St")?;
    if !normalized.expansions.is_empty() {
        println!("   Address normalization test: '123 Main St' -> {:?}", normalized.expansions);
    }
    
    // Test address parsing
    let parsed = postal.parse_address("123 Main Street, New York, NY 10001")?;
    println!("   Address parsing test:");
    if let Some(house_number) = &parsed.house_number {
        println!("     house_number: {}", house_number);
    }
    if let Some(road) = &parsed.road {
        println!("     road: {}", road);
    }
    if let Some(city) = &parsed.city {
        println!("     city: {}", city);
    }
    if let Some(state) = &parsed.state {
        println!("     state: {}", state);
    }
    if let Some(postcode) = &parsed.postcode {
        println!("     postcode: {}", postcode);
    }
    
    Ok(())
}
