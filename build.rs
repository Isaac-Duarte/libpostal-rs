//! Build script for libpostal-rs
//!
//! This script handles:
//! - Compiling libpostal C library from source
//! - Setting up FFI bindings with bindgen
//! - Managing static linking
//! - Platform-specific configuration

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=LIBPOSTAL_SYSTEM");
    println!("cargo:rerun-if-env-changed=LIBPOSTAL_SKIP_BUILD");

    // Check for skip build flag (useful for docs.rs)
    if env::var("LIBPOSTAL_SKIP_BUILD").is_ok() {
        println!("cargo:warning=Skipping libpostal build due to LIBPOSTAL_SKIP_BUILD");
        create_dummy_bindings(&PathBuf::from(env::var("OUT_DIR").unwrap()));
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();

    // Check for required build tools (only if not skipping)
    check_build_dependencies();

    // Try to find system libpostal first (faster builds for development)
    if try_system_libpostal() {
        println!("cargo:warning=Using system libpostal");
        generate_bindings(&out_dir);
        return;
    }

    println!("cargo:warning=System libpostal not found, building from source");

    // Download and build libpostal from source
    let libpostal_dir = download_and_extract_libpostal(&out_dir);
    let install_dir = build_libpostal(&libpostal_dir, &out_dir, &target);

    // Set up linking
    setup_linking(&install_dir);

    // Generate bindings
    generate_bindings_with_include(&out_dir, &install_dir);

    // Copy libpostal_data executable to a more accessible location
    copy_libpostal_data_executable(&install_dir, &out_dir);

    println!("cargo:warning=libpostal build completed successfully");
}

/// Create dummy bindings for when we can't build libpostal
fn create_dummy_bindings(out_dir: &Path) {
    let dummy_bindings = r#"
// Dummy bindings for when libpostal cannot be built
pub type libpostal_address_parser_options_t = u32;
pub type libpostal_normalize_options_t = u32;
pub type libpostal_address_parser_response_t = u32;

pub unsafe extern "C" fn libpostal_setup() -> bool { false }
pub unsafe extern "C" fn libpostal_setup_parser() -> bool { false }
pub unsafe extern "C" fn libpostal_teardown() {}
pub unsafe extern "C" fn libpostal_teardown_parser() {}
pub unsafe extern "C" fn libpostal_get_address_parser_default_options() -> libpostal_address_parser_options_t { 0 }
pub unsafe extern "C" fn libpostal_get_default_options() -> libpostal_normalize_options_t { 0 }
pub unsafe extern "C" fn libpostal_parse_address(_: *const std::os::raw::c_char, _: libpostal_address_parser_options_t, _: *mut usize) -> *mut libpostal_address_parser_response_t { std::ptr::null_mut() }
pub unsafe extern "C" fn libpostal_expand_address(_: *const std::os::raw::c_char, _: libpostal_normalize_options_t, _: *mut usize) -> *mut *mut std::os::raw::c_char { std::ptr::null_mut() }
pub unsafe extern "C" fn libpostal_address_parser_response_destroy(_: *mut libpostal_address_parser_response_t) {}
pub unsafe extern "C" fn libpostal_expansion_array_destroy(_: *mut *mut std::os::raw::c_char, _: usize) {}
"#;

    let bindings_path = out_dir.join("bindings.rs");
    std::fs::write(&bindings_path, dummy_bindings).expect("Failed to write dummy bindings");
}

/// Check for required build dependencies
fn check_build_dependencies() {
    let required_tools = if cfg!(target_os = "windows") {
        vec!["gcc", "make"] // Assuming MinGW/MSYS2
    } else {
        vec!["gcc", "make", "autoconf", "automake", "libtool"]
    };

    for tool in required_tools {
        if Command::new(tool).arg("--version").output().is_err() {
            panic!(
                "Required build tool '{tool}' not found. Please install build dependencies.\n\
                 On Ubuntu/Debian: sudo apt-get install build-essential autoconf automake libtool pkg-config\n\
                 On macOS: brew install autoconf automake libtool pkg-config\n\
                 On Windows: Install MSYS2 and the mingw-w64 toolchain"
            );
        }
    }
}

/// Try to use system-installed libpostal via pkg-config
fn try_system_libpostal() -> bool {
    // Only try system libpostal if explicitly requested
    if env::var("LIBPOSTAL_SYSTEM").is_ok() {
        if let Ok(library) = pkg_config::probe_library("libpostal") {
            for path in &library.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }
            for lib in &library.libs {
                println!("cargo:rustc-link-lib={lib}");
            }
            return true;
        }
    }
    false
}

/// Download and extract libpostal source code
fn download_and_extract_libpostal(out_dir: &Path) -> PathBuf {
    let libpostal_version = "1.1";
    let libpostal_url = format!(
        "https://github.com/openvenues/libpostal/archive/refs/tags/v{libpostal_version}.tar.gz"
    );
    let libpostal_dir = out_dir.join(format!("libpostal-{libpostal_version}"));

    // Skip download if already extracted
    if libpostal_dir.exists() {
        return libpostal_dir;
    }

    println!("cargo:warning=Downloading libpostal v{libpostal_version}");

    // Download the tarball
    let response = reqwest::blocking::get(&libpostal_url).expect("Failed to download libpostal");

    if !response.status().is_success() {
        panic!("Failed to download libpostal: HTTP {}", response.status());
    }

    let tarball_path = out_dir.join("libpostal.tar.gz");
    let content = response.bytes().expect("Failed to read response");

    // Verify the content is actually gzipped
    if content.len() < 2 || content[0] != 0x1f || content[1] != 0x8b {
        panic!(
            "Downloaded file is not a valid gzip file (got {} bytes, magic: {:02x} {:02x})",
            content.len(),
            content.first().unwrap_or(&0),
            content.get(1).unwrap_or(&0)
        );
    }

    std::fs::write(&tarball_path, &content).expect("Failed to write tarball");

    // Extract the tarball
    let tar_gz = std::fs::File::open(&tarball_path).expect("Failed to open tarball");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(out_dir).expect("Failed to extract tarball");

    // Clean up
    std::fs::remove_file(&tarball_path).ok();

    libpostal_dir
}

/// Build libpostal from source using autotools
fn build_libpostal(libpostal_dir: &Path, out_dir: &Path, target: &str) -> PathBuf {
    let install_dir = out_dir.join("libpostal-install");

    // Skip build if already built
    let lib_file = install_dir.join("lib").join("libpostal.a");
    if lib_file.exists() {
        return install_dir;
    }

    println!("cargo:warning=Building libpostal from source...");

    // Create install directory
    fs::create_dir_all(&install_dir).expect("Failed to create install directory");

    // Run autogen.sh (if it exists) or autoreconf
    if libpostal_dir.join("autogen.sh").exists() {
        run_command(
            Command::new("bash")
                .arg("autogen.sh")
                .current_dir(libpostal_dir),
            "autogen.sh",
        );
    } else {
        run_command(
            Command::new("autoreconf")
                .args(["-fiv"])
                .current_dir(libpostal_dir),
            "autoreconf",
        );
    }

    // Apply patches to fix compilation issues
    apply_libpostal_patches(libpostal_dir).expect("Failed to apply patches");

    // Configure build
    let mut configure_cmd = Command::new("./configure");
    configure_cmd
        .arg(format!("--prefix={}", install_dir.display()))
        .arg("--enable-static")
        .arg("--disable-shared")
        .current_dir(libpostal_dir);

    // Check if we should enable data download
    // Enable data download if runtime-data feature is enabled and we're not in a restricted environment
    let enable_data_download = cfg!(feature = "runtime-data")
        && env::var("LIBPOSTAL_DISABLE_DATA_DOWNLOAD").is_err()
        && env::var("DOCS_RS").is_err(); // Disable on docs.rs

    if enable_data_download {
        println!("cargo:warning=Enabling libpostal data download");
        // Set datadir to a location we can access later
        let data_dir = install_dir.join("share").join("libpostal");
        configure_cmd.arg(format!("--datadir={}", data_dir.display()));
    } else {
        println!("cargo:warning=Disabling libpostal data download (will handle separately)");
        configure_cmd.arg("--disable-data-download");
    }

    // Cross-compilation setup
    if target.contains("windows") {
        configure_cmd.arg("--host=x86_64-w64-mingw32");
    } else if target.contains("apple") {
        configure_cmd.env("CC", "clang");
        configure_cmd.env("CXX", "clang++");
    }

    run_command(&mut configure_cmd, "configure");

    // Build
    let num_jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "4".to_string());
    run_command(
        Command::new("make")
            .arg(format!("-j{num_jobs}"))
            .current_dir(libpostal_dir),
        "make",
    );

    // Install
    run_command(
        Command::new("make")
            .arg("install")
            .current_dir(libpostal_dir),
        "make install",
    );

    // If data download was enabled, the data should now be available
    if enable_data_download {
        let data_dir = install_dir
            .join("share")
            .join("libpostal")
            .join("libpostal");
        if data_dir.exists() {
            println!(
                "cargo:warning=libpostal data downloaded to: {}",
                data_dir.display()
            );
            // Set environment variable so our Rust code knows where to find the data
            println!(
                "cargo:rustc-env=LIBPOSTAL_BUILT_DATA_DIR={}",
                data_dir.display()
            );
        } else {
            println!(
                "cargo:warning=Data download may have failed, falling back to runtime download"
            );
        }
    }

    install_dir
}

/// Set up linking flags for the built library
fn setup_linking(install_dir: &Path) {
    let lib_dir = install_dir.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=postal");

    // Link additional libraries that libpostal depends on
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-lib=dl");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=m");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=userenv");
    }
}

/// Generate FFI bindings using bindgen (system libpostal)
fn generate_bindings(out_dir: &Path) {
    generate_bindings_impl(out_dir, None);
}

/// Generate FFI bindings using bindgen (built from source)
fn generate_bindings_with_include(out_dir: &Path, install_dir: &Path) {
    let include_dir = install_dir.join("include");
    generate_bindings_impl(out_dir, Some(include_dir));
}

/// Internal implementation for generating bindings
fn generate_bindings_impl(out_dir: &Path, include_dir: Option<PathBuf>) {
    // Create wrapper.h if it doesn't exist
    let wrapper_h_content = r#"
#include <libpostal/libpostal.h>
"#;

    let wrapper_path = out_dir.join("wrapper.h");
    std::fs::write(&wrapper_path, wrapper_h_content).expect("Failed to write wrapper.h");

    let mut builder = bindgen::Builder::default()
        .header(wrapper_path.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("libpostal_.*")
        .allowlist_type("libpostal_.*")
        .allowlist_var("LIBPOSTAL_.*")
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_ord(true)
        .derive_partialeq(true)
        .derive_partialord(true);

    // Add include directory if building from source
    if let Some(include_dir) = include_dir {
        builder = builder.clang_arg(format!("-I{}", include_dir.display()));
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let bindings_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Failed to write bindings");
}

/// Run a command and panic if it fails
fn run_command(cmd: &mut Command, name: &str) {
    println!("cargo:warning=Running: {cmd:?}");
    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("Failed to execute {name}"));
    if !status.success() {
        panic!("{} failed with exit code: {:?}", name, status.code());
    }
}

fn apply_libpostal_patches(libpostal_dir: &Path) -> std::io::Result<()> {
    println!("cargo:warning=Applying patches to libpostal source...");

    // Fix the type incompatibility in sparse_matrix_utils.c
    let sparse_matrix_utils_path = libpostal_dir.join("src/sparse_matrix_utils.c");
    if sparse_matrix_utils_path.exists() {
        let content = std::fs::read_to_string(&sparse_matrix_utils_path)?;

        // Fix the incompatible pointer type issue in sparse_matrix_unique_columns
        let patched_content = content.replace(
            "if (sparse_matrix_add_unique_columns(matrix, unique_columns, ret)) {",
            "if (sparse_matrix_add_unique_columns(matrix, (khash_t(int_uint32) *)unique_columns, ret)) {"
        );

        if content != patched_content {
            println!("cargo:warning=Applied patch to fix pointer type compatibility in sparse_matrix_utils.c");
            std::fs::write(&sparse_matrix_utils_path, patched_content)?;
        }
    }

    // Also check for the libpostal.c issue (might exist in some versions)
    let libpostal_c_path = libpostal_dir.join("src/libpostal.c");
    if libpostal_c_path.exists() {
        let content = std::fs::read_to_string(&libpostal_c_path)?;

        // Fix the incompatible pointer type issue
        let patched_content = content.replace(
            "libpostal_language_classifier_response_t *response = classify_languages(address);",
            "language_classifier_response_t *raw_response = classify_languages(address);\n    libpostal_language_classifier_response_t *response = (libpostal_language_classifier_response_t *)raw_response;"
        );

        if content != patched_content {
            println!(
                "cargo:warning=Applied patch to fix pointer type compatibility in libpostal.c"
            );
            std::fs::write(&libpostal_c_path, patched_content)?;
        }
    }

    Ok(())
}

/// Copy the libpostal_data executable to a more accessible location
fn copy_libpostal_data_executable(install_dir: &Path, out_dir: &Path) {
    let source_path = install_dir.join("bin/libpostal_data");
    
    if source_path.exists() {
        // Copy to the OUT_DIR with a predictable name
        let dest_path = out_dir.join("libpostal_data");
        
        if let Err(e) = std::fs::copy(&source_path, &dest_path) {
            println!("cargo:warning=Failed to copy libpostal_data executable: {}", e);
        } else {
            println!("cargo:warning=Copied libpostal_data to: {}", dest_path.display());
            // Set the path as an environment variable so the Rust code can find it
            println!("cargo:rustc-env=LIBPOSTAL_DATA_EXECUTABLE={}", dest_path.display());
            
            // Make it executable on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&dest_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    if let Err(e) = std::fs::set_permissions(&dest_path, perms) {
                        println!("cargo:warning=Failed to set executable permissions: {}", e);
                    }
                }
            }
        }
    } else {
        println!("cargo:warning=libpostal_data executable not found at: {}", source_path.display());
    }
}
