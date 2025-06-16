//! BRAW SDK Test Example
//!
//! This example demonstrates the BRAW SDK functionality including:
//! - SDK initialization and availability testing
//! - File validation and opening
//! - Frame extraction with format detection
//! - Error handling and diagnostics
//!
//! Usage:
//! ```bash
//! cargo run --example test_braw --features native-ffi -- /path/to/file.braw
//! ```

use sd_braw::{BrawSdk, BrawFile, validate_braw_file, test_braw_file_with_sdk};
use std::env;
use std::path::Path;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_braw_file>", args[0]);
        std::process::exit(1);
    }

    let braw_path = Path::new(&args[1]);
    info!("Testing BRAW file: {}", braw_path.display());

    // Test 1: Basic file validation
    info!("=== Test 1: Basic File Validation ===");
    match validate_braw_file(braw_path).await {
        Ok(_) => info!("✓ Basic file validation passed"),
        Err(e) => {
            error!("✗ Basic file validation failed: {}", e);
            return Err(e.into());
        }
    }

    // Test 2: SDK initialization
    info!("=== Test 2: SDK Initialization ===");
    let mut sdk = match BrawSdk::new() {
        Ok(sdk) => {
            info!("✓ SDK initialized successfully");
            sdk
        }
        Err(e) => {
            error!("✗ SDK initialization failed: {}", e);
            return Err(e.into());
        }
    };

    info!("SDK using native implementation: {}", sdk.is_using_native_sdk());
    info!("SDK initialized: {}", sdk.is_initialized());

    // Test 3: SDK availability test
    info!("=== Test 3: SDK Availability Test ===");
    match sdk.test_sdk_availability() {
        Ok(_) => info!("✓ SDK availability test passed"),
        Err(e) => {
            warn!("⚠ SDK availability test failed: {}", e);
            info!("This is expected if the BlackmagicRAW SDK is not installed");
        }
    }

    // Test 4: Codec creation
    info!("=== Test 4: Codec Creation ===");
    match sdk.create_codec() {
        Ok(_) => info!("✓ Codec created successfully"),
        Err(e) => {
            warn!("⚠ Codec creation failed: {}", e);
            info!("Continuing with stub mode...");
        }
    }

    // Test 5: Comprehensive file testing with SDK
    info!("=== Test 5: Comprehensive File Testing ===");
    match test_braw_file_with_sdk(braw_path).await {
        Ok(_) => info!("✓ Comprehensive file test passed"),
        Err(e) => {
            warn!("⚠ Comprehensive file test failed: {}", e);
        }
    }

    // Test 6: File opening
    info!("=== Test 6: File Opening ===");
    match sdk.open_clip(braw_path) {
        Ok(_) => {
            info!("✓ File opened successfully with SDK");

            // Test 7: Get file information
            info!("=== Test 7: File Information ===");

            match sdk.get_frame_count() {
                Ok(count) => info!("Frame count: {}", count),
                Err(e) => warn!("Failed to get frame count: {}", e),
            }

            match sdk.get_dimensions() {
                Ok((width, height)) => info!("Dimensions: {}x{}", width, height),
                Err(e) => warn!("Failed to get dimensions: {}", e),
            }

            // Test 8: Frame extraction
            info!("=== Test 8: Frame Extraction ===");
            match sdk.extract_frame(0).await {
                Ok(image) => {
                    info!("✓ Frame extracted successfully");
                    info!("Image format: {:?}", image.color());
                    info!("Image dimensions: {}x{}", image.width(), image.height());

                    // Save the extracted frame for inspection
                    let output_path = "extracted_frame.png";
                    match image.save(output_path) {
                        Ok(_) => info!("Frame saved to: {}", output_path),
                        Err(e) => warn!("Failed to save frame: {}", e),
                    }
                }
                Err(e) => warn!("Frame extraction failed: {}", e),
            }
        }
        Err(e) => {
            warn!("⚠ File opening failed: {}", e);
            info!("This is expected if the BlackmagicRAW SDK is not available or the file is not a valid BRAW file");
        }
    }

    // Test 9: High-level API
    info!("=== Test 9: High-Level API ===");
    match BrawFile::open(braw_path.to_path_buf()).await {
        Ok(braw_file) => {
            info!("✓ BRAW file opened with high-level API");
            info!("Using native SDK: {}", braw_file.is_using_native_sdk());

            match braw_file.get_frame_count().await {
                Ok(count) => info!("Frame count (high-level): {}", count),
                Err(e) => warn!("Failed to get frame count (high-level): {}", e),
            }

            match braw_file.get_dimensions().await {
                Ok((width, height)) => info!("Dimensions (high-level): {}x{}", width, height),
                Err(e) => warn!("Failed to get dimensions (high-level): {}", e),
            }
        }
        Err(e) => warn!("High-level API failed: {}", e),
    }

    info!("=== BRAW Test Complete ===");
    info!("All tests completed. Check the logs above for detailed results.");

    Ok(())
}