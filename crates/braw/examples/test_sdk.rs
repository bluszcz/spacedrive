use std::path::Path;

#[cfg(feature = "native-ffi")]
use sd_braw::BrawFile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 BRAW SDK Diagnostic Test");
    println!("==========================");

    // Test file paths - try both the sample and user files
    let test_files = [
        "/Applications/Blackmagic RAW/Blackmagic RAW SDK/Media/sample.braw",
        "/Volumes/BRAW_2023/2025/2025-01-01-BMPCC4k-tests/A007_10190959_C004.braw",
    ];

    for test_file in &test_files {
        println!("\n📁 Testing file: {}", test_file);
        let path = Path::new(test_file);

        test_braw_file(path).await;
    }

    println!("\n🔍 Environment Check:");

    // Check environment variables
    if let Ok(dyld_framework_path) = std::env::var("DYLD_FRAMEWORK_PATH") {
        println!("✅ DYLD_FRAMEWORK_PATH: {}", dyld_framework_path);
    } else {
        println!("❌ DYLD_FRAMEWORK_PATH not set");
    }

    if let Ok(dyld_library_path) = std::env::var("DYLD_LIBRARY_PATH") {
        println!("✅ DYLD_LIBRARY_PATH: {}", dyld_library_path);
    } else {
        println!("❌ DYLD_LIBRARY_PATH not set");
    }

    // Check if BlackmagicRAW framework exists
    let framework_paths = [
        "/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Libraries/BlackmagicRawAPI.framework",
        "/Library/Frameworks/BlackmagicRawAPI.framework",
        "/System/Library/Frameworks/BlackmagicRawAPI.framework",
    ];

    println!("\n📦 Framework Search:");
    for framework_path in &framework_paths {
        if Path::new(framework_path).exists() {
            println!("✅ Found framework: {}", framework_path);
        } else {
            println!("❌ Not found: {}", framework_path);
        }
    }

    Ok(())
}

async fn test_braw_file(path: &Path) {
    // Check if file exists
    if !path.exists() {
        println!("❌ File does not exist!");
        return;
    }

    println!("✅ File exists");

    // Check file size
    if let Ok(metadata) = std::fs::metadata(path) {
        println!("📊 File size: {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1024.0 / 1024.0);
    }

    // Test BRAW SDK functionality
    #[cfg(feature = "native-ffi")]
    {
        println!("\n🔧 Testing BRAW SDK...");

        match BrawFile::open(path).await {
            Ok(braw_file) => {
                println!("✅ Successfully opened BRAW file!");

                // Try to get metadata
                match braw_file.get_metadata().await {
                    Ok(metadata) => {
                        println!("✅ Successfully extracted metadata:");
                        println!("   - Duration: {:.2}s", metadata.duration_seconds);
                        println!("   - Frame rate: {:.2} fps", metadata.frame_rate);
                        println!("   - Resolution: {}", metadata.resolution());
                        println!("   - Frame count: {}", metadata.total_frames);
                        println!("   - Codec: {}", metadata.codec);
                        if let Some(camera) = &metadata.camera_model {
                            println!("   - Camera: {}", camera);
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to extract metadata: {}", e);
                    }
                }

                // Try to generate thumbnail
                println!("\n🖼️  Testing thumbnail generation...");
                let config = sd_braw::thumbnail::ThumbnailConfig::default();

                match braw_file.generate_thumbnail(config).await {
                    Ok(thumbnail) => {
                        println!("✅ Successfully generated thumbnail!");
                        println!("   - Dimensions: {}x{}", thumbnail.width(), thumbnail.height());
                    }
                    Err(e) => {
                        println!("❌ Failed to generate thumbnail: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to open BRAW file: {}", e);
                println!("   This matches the error we see in the logs!");
            }
        }
    }

    #[cfg(not(feature = "native-ffi"))]
    {
        println!("❌ BRAW native-ffi feature not enabled in this build");
    }
}