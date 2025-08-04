# Refactor libpostal-rs Data Module - Task Plan

## Overview

This plan outlines the refactoring of the `data.rs` module to replace the external `libpostal_data` shell script dependency with a native Rust implementation. The goal is to eliminate the need for the external executable while maintaining the same functionality for downloading and managing libpostal data files.

## Current Architecture Analysis

The current system relies on:
- An external `libpostal_data` shell script that downloads data in chunks
- Multiple methods to locate this executable (build dir, project root, env vars, system)
- Complex fallback logic through various download strategies

## Proposed Architecture

Replace the external `libpostal_data` dependency with:
- Native Rust HTTP client for multipart downloads
- Direct GitHub releases API integration
- Chunked download implementation using HTTP ranges
- Progress reporting and retry logic
- Version management and integrity verification

## Implementation Plan

### Phase 1: Core Infrastructure (Foundation)

- [ ] **Add new constants for GitHub API integration**
  - GitHub repository information (`LIBPOSTAL_REPO_NAME`)
  - Version constants for different data components
  - Chunk size configuration (`CHUNK_SIZE`, default 64MB)
  - File versioning constants (`LIBPOSTAL_DATA_DIR_VERSION_STRING`)

- [ ] **Create new data structures**
  - `DataComponent` enum for base, parser, language_classifier, all
  - `ComponentInfo` struct with version, chunks, filename information
  - `DownloadProgress` struct for tracking download state
  - `ReleaseAsset` struct for GitHub release information

- [ ] **Add helper utility functions**
  - `kill_background_processes()` equivalent for async task cancellation
  - `get_component_info(component: DataComponent)` to return component metadata
  - `validate_data_version(data_dir: &Path)` to check version compatibility
  - File manipulation helpers (create directories, cleanup)

### Phase 2: GitHub API Integration

- [ ] **Implement GitHub releases API client**
  - `fetch_latest_release_info(repo: &str)` to get release metadata
  - `get_release_asset_url(version: &str, filename: &str)` to construct URLs
  - Handle API rate limiting and error responses
  - Verify asset availability before attempting download

- [ ] **Create version management system**
  - `check_component_version(component: DataComponent, data_dir: &Path)` 
  - `write_version_file(component: DataComponent, version: &str, data_dir: &Path)`
  - `cleanup_old_version_data(data_dir: &Path)` for version migrations
  - Backwards compatibility with existing version files

### Phase 3: Multipart Download Implementation

- [ ] **Implement chunked HTTP download**
  - `download_release_multipart(url: &str, filename: &Path, num_chunks: usize)`
  - HTTP Range request implementation using `reqwest`
  - Parallel chunk downloading using `tokio` tasks
  - Chunk reassembly and integrity verification

- [ ] **Add download progress and error handling**
  - Progress callback support for download tracking
  - Retry logic for failed chunks (up to 3 retries with exponential backoff)
  - Resume capability for interrupted downloads
  - Proper cleanup of partial downloads on failure

- [ ] **Create download coordinator**
  - `download_component(component: DataComponent, data_dir: &Path)` main interface
  - Manage concurrent downloads with configurable worker pool
  - Handle download cancellation and cleanup
  - Integration with existing `download_from_official_sources()` method

### Phase 4: Archive Management

- [ ] **Enhance archive extraction**
  - Improve existing `extract_tar_gz()` method with better error handling
  - Add validation of archive integrity before extraction
  - Implement atomic extraction (extract to temp, then move)
  - Add progress reporting for extraction phase

- [ ] **Implement data validation**
  - Extend `verify_data()` to check component-specific file structure
  - Add file size validation against expected sizes
  - Implement optional checksum verification for downloaded files
  - Create `validate_component_data(component: DataComponent, data_dir: &Path)`

### Phase 5: Integration and Cleanup

- [ ] **Refactor existing download methods**
  - Replace `download_with_libpostal_data()` with new native implementation
  - Update `download_real_data()` to use new component-based system
  - Remove all `libpostal_data` executable finding methods:
    - `find_embedded_libpostal_data()`
    - `try_executable_from_env()`
    - `try_executable_from_project_root()`
    - `try_executable_from_build_dir()`
    - `try_system_executable()`
    - `find_file_with_pattern()`

- [ ] **Update public API methods**
  - Modify `copy_libpostal_data_to_root()` to no longer copy executable
  - Update method documentation to reflect native implementation
  - Ensure `ensure_data()` and `download_data()` work with new system
  - Maintain backwards compatibility for existing configuration

- [ ] **Clean up dependencies and build system**
  - Remove references to `libpostal_data` executable in build scripts
  - Update `build.rs` to not copy or locate the executable
  - Remove `LIBPOSTAL_DATA_EXECUTABLE` environment variable usage
  - Clean up any shell script execution code

### Phase 6: Testing and Documentation

- [ ] **Add comprehensive tests**
  - Unit tests for each component download function
  - Integration tests for full download and extraction process
  - Mock HTTP server tests for download error scenarios
  - Version migration tests

- [ ] **Update documentation and examples**
  - Update method documentation to reflect native implementation
  - Modify `data_download.rs` example to show new capabilities
  - Update README.md to remove references to external executable
  - Add troubleshooting guide for common download issues

- [ ] **Performance optimization**
  - Benchmark download performance vs. original shell script
  - Optimize chunk size and concurrent workers for different network conditions
  - Add configuration options for download tuning
  - Implement download caching and resumption

## Configuration Changes

### New DataConfig Options
```rust
pub struct DataConfig {
    // ... existing fields ...
    
    /// Number of parallel download workers (default: 12)
    pub download_workers: usize,
    
    /// Chunk size for multipart downloads (default: 64MB)
    pub chunk_size: usize,
    
    /// Number of retry attempts for failed downloads (default: 3)
    pub max_retries: usize,
    
    /// Timeout for individual chunk downloads (default: 30s)
    pub chunk_timeout_seconds: u64,
}
```

## Migration Strategy

1. **Backwards Compatibility**: Ensure existing code continues to work during transition
2. **Graceful Fallback**: Keep existing fallback methods initially, remove after testing
3. **Feature Flag**: Consider adding feature flag for new vs. old implementation during development
4. **Documentation**: Clear migration path for users currently relying on external executable

## Risk Mitigation

- **Download Reliability**: Implement robust retry and resume logic
- **Performance**: Ensure new implementation matches or exceeds shell script performance
- **Memory Usage**: Stream processing for large files to avoid memory issues
- **Platform Compatibility**: Test across different platforms (Linux, macOS, Windows)
- **Network Issues**: Handle various network conditions and proxy configurations

## Success Criteria

- [ ] All existing functionality preserved
- [ ] No external `libpostal_data` executable dependency
- [ ] Download performance equal or better than shell script
- [ ] Comprehensive error handling and user feedback
- [ ] Full test coverage for new implementation
- [ ] Updated documentation and examples

## Implementation Notes

- Use `reqwest` for HTTP client (already in dependencies)
- Use `tokio` for async operations (already in dependencies) 
- Use `tar` and `flate2` for archive handling (already in dependencies)
- Maintain existing error types and error handling patterns
- Follow existing code style and documentation standards
- Ensure thread safety for concurrent operations

## Estimated Complexity

- **High Complexity**: Multipart download implementation and GitHub API integration
- **Medium Complexity**: Version management and archive handling improvements
- **Low Complexity**: Code cleanup and documentation updates

This refactoring will significantly improve the library's self-containment and reduce external dependencies while maintaining all existing functionality.
