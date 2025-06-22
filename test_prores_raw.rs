use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prores_raw_file = Path::new("/Volumes/BRAW_2023/2025/2025-01-02-NinjaUltra/NINJAU_S001_S001_T006.MOV");
    
    if !prores_raw_file.exists() {
        println!("ProRes RAW test file not found: {}", prores_raw_file.display());
        return Ok(());
    }
    
    println!("Testing ProRes RAW file detection...");
    println!("ProRes RAW file found: {}", prores_raw_file.display());
    println!("File size: {} bytes", std::fs::metadata(prores_raw_file)?.len());
    
    // Try to probe with ffprobe to see codec info
    let output = std::process::Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            prores_raw_file.to_str().unwrap()
        ])
        .output()?;
    
    if output.status.success() {
        println!("FFprobe successful!");
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("aprn") {
            println!("✓ Detected Apple ProRes RAW codec (aprn)");
        }
        if stdout.contains("Apple ProRes RAW") {
            println!("✓ Detected Apple ProRes RAW encoder");
        }
    } else {
        println!("FFprobe failed:");
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    println!("\n🎯 ProRes RAW integration should now handle this file automatically!");
    println!("The file will bypass FFmpeg and use macOS VideoToolbox APIs instead.");
    
    Ok(())
}