# libpostal-rs


Rust bindings for [libpostal](https://github.com/openvenues/libpostal), a library for parsing and normalizing international addresses. If you need to parse street addresses from around the world into structured components, this library can help.

This library handles the complexities of international address formats by wrapping the battle-tested libpostal C library. It can take messy address strings and break them down into useful parts like house numbers, street names, cities, and postal codes.

## What it does

- **Address Parsing**: Turn "123 Main St, New York, NY 10001" into structured data
- **Address Normalization**: Convert "St" to "Street", expand abbreviations 
- **International Support**: Works with addresses from many countries and languages
- **Memory Safe**: Rust wrappers around the C library with proper cleanup

## Getting Started

First, add this to your `Cargo.toml`:

```toml
[dependencies]
libpostal-rs = "0.1"
```

Here's a simple example:

```rust
use libpostal_rs::LibPostal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the library (downloads data files on first run)
    let postal = LibPostal::new().await?;

    // Parse an address
    let parsed = postal.parse_address("123 Main St, New York, NY 10001")?;
    
    if let Some(house_number) = parsed.house_number {
        println!("House number: {}", house_number);
    }
    if let Some(road) = parsed.road {
        println!("Street: {}", road);
    }
    if let Some(city) = parsed.city {
        println!("City: {}", city);
    }

    Ok(())
}
```

The first time you run this, it will download about 1GB of language models and data files. After that, startup is fast.

## Basic Usage

### Parsing Addresses

The main thing you'll probably want to do is parse addresses:

```rust
let postal = LibPostal::new().await?;
let parsed = postal.parse_address("221B Baker Street, London")?;

// Access individual components
println!("House: {:?}", parsed.house_number);
println!("Street: {:?}", parsed.road);
println!("City: {:?}", parsed.city);
```

### Normalizing Addresses

You can also normalize addresses to expand abbreviations:

```rust
let normalized = postal.normalize_address("123 Main St")?;
for expansion in normalized.expansions {
    println!("{}", expansion); // "123 main street", "123 main st"
}
```

### Working with Non-English Addresses

The library works with international addresses. You can provide language hints:

```rust
let parsed = postal.parse_address_with_hints(
    "123 Rue de la Paix, Paris", 
    Some("fr"), // French
    Some("FR")  // France
)?;
```

### Processing Multiple Addresses

For better performance when processing many addresses:

```rust
let parser = postal.parser();
let addresses = vec!["123 Main St", "456 Oak Ave", "789 Pine Rd"];
let results = parser.parse_batch(&addresses)?;
```

## Installation and Setup

### Build Requirements

This library compiles libpostal from source, so you'll need some build tools:

**Ubuntu/Debian:**
```bash
sudo apt-get install build-essential autoconf automake libtool pkg-config
```

**macOS:**
```bash
brew install autoconf automake libtool pkg-config
```

**Windows:**
You'll need MSYS2 or Visual Studio Build Tools. This is the trickiest platform to set up.

### Data Files

The library needs about 1GB of data files (language models, address dictionaries, etc.). On first run, it will automatically download these to your home directory under `.libpostal/`. This only happens once.

If you want to control where the data goes:

```rust
use libpostal_rs::{LibPostal, LibPostalConfig};

let config = LibPostalConfig::builder()
    .data_dir("/custom/path/to/data")
    .build();
    
let postal = LibPostal::with_config(config).await?;
```

## What Gets Parsed

The parser can extract these components from addresses:

- `house_number` - "123", "123A"
- `road` - "Main Street", "Broadway"  
- `unit` - "Apt 2B", "Unit 5"
- `city` - "New York", "London"
- `state` - "NY", "California"
- `postcode` - "10001", "SW1A 1AA"
- `country` - "USA", "United Kingdom"

Plus several others like `level`, `suburb`, `entrance`, etc. Check the `ParsedAddress` struct for the full list.

## Current Status

This library is in early development. Here's what you should know:

**What works:**
- Basic address parsing for common formats
- Address normalization 
- International address support (many languages)
- Async API with proper memory management

If you run into issues, please open a GitHub issue. This is a work in progress.

## Examples

You can find more examples in the [`examples/`](examples/) directory:

- [`basic_usage.rs`](examples/basic_usage.rs) - Simple address parsing and normalization
- [`advanced_usage.rs`](examples/advanced_usage.rs) - More complex scenarios
- [`basic_parsing.rs`](examples/basic_parsing.rs) - Just parsing without the extras

To run an example:
```bash
cargo run --example basic_usage
```

## Optional Features

You can enable additional features in your `Cargo.toml`:

```toml
[dependencies]
libpostal-rs = { version = "0.1", features = ["serde", "parallel"] }
```

- `serde` - Serialization support for parsed addresses
- `parallel` - Parallel batch processing with rayon
- `runtime-data` - Download data files at runtime (enabled by default)

## Contributing

This project needs help! Some areas where contributions would be valuable:

- Testing on different platforms (especially Windows)
- Performance benchmarking and optimization
- Better error handling and recovery
- Documentation improvements
- Example applications

If you're interested in contributing, please:

1. Check the existing issues on GitHub
2. Open an issue to discuss larger changes
3. Submit a pull request with tests

The codebase is straightforward Rust with FFI bindings to the C library.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

The underlying libpostal C library is licensed under the MIT License.

## Acknowledgments

This library builds upon the excellent [libpostal](https://github.com/openvenues/libpostal) project by Al Barrentine and contributors. The heavy lifting is done by their C library - this is just a Rust wrapper to make it easier to use from Rust code.
