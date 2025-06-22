use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prores_raw_file = Path::new("/Volumes/BRAW_2023/2025/2025-01-02-NinjaUltra/NINJAU_S001_S001_T006.MOV");
    
    if !prores_raw_file.exists() {
        println!("❌ ProRes RAW test file not found: {}", prores_raw_file.display());
        return Ok(());
    }
    
    println!("🎯 Testing ProRes RAW integration...");
    
    // Test detection first
    println!("Testing ProRes RAW detection...");
    let is_prores_raw = sd_core_heavy_lifting::media_processor::helpers::prores_raw_decoder::is_prores_raw_file(&prores_raw_file).await;
    println!("ProRes RAW detected: {}", is_prores_raw);
    
    if is_prores_raw {
        println!("✅ ProRes RAW file correctly detected!");
        
        // Test frame extraction
        println!("Testing frame extraction...");
        match sd_core_heavy_lifting::media_processor::helpers::prores_raw_decoder::extract_first_frame(&prores_raw_file).await {
            Ok(image) => {
                println!("✅ Successfully extracted frame! Dimensions: {}x{}", image.width(), image.height());
                
                // Test thumbnail generation
                println!("Testing thumbnail generation...");
                let output_path = "/tmp/prores_raw_test_thumb.webp";
                match sd_core_heavy_lifting::media_processor::helpers::prores_raw_thumbnailer::generate_prores_raw_thumbnail(&prores_raw_file, output_path).await {
                    Ok(_) => {
                        println!("✅ Successfully generated ProRes RAW thumbnail at: {}", output_path);
                        if Path::new(output_path).exists() {
                            let metadata = std::fs::metadata(output_path)?;
                            println!("✅ Thumbnail file exists, size: {} bytes", metadata.len());
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to generate thumbnail: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to extract frame: {}", e);
            }
        }
    } else {
        println!("❌ ProRes RAW file not detected (this should not happen!)");
    }
    
    println!("🏆 ProRes RAW integration test completed!");
    
    Ok(())
}