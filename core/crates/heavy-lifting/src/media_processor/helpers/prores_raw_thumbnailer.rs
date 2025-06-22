use super::prores_raw_decoder::extract_first_frame;
use crate::media_processor::thumbnailer::NonCriticalThumbnailerError;

use sd_images::scale_dimensions;

use std::path::Path;

use image::{imageops, DynamicImage, GenericImageView};
use tokio::task::spawn_blocking;
use webp::Encoder;

const TARGET_PX: f32 = 1_048_576.0; // 1024x1024
const TARGET_QUALITY: f32 = 60.0;

pub async fn generate_prores_raw_thumbnail(
    file_path: impl AsRef<Path> + Send,
    output_path: impl AsRef<Path> + Send,
) -> Result<(), NonCriticalThumbnailerError> {
    let file_path = file_path.as_ref().to_path_buf();
    let output_path = output_path.as_ref().to_path_buf();
    let file_path_clone = file_path.clone();

    spawn_blocking(move || {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| NonCriticalThumbnailerError::VideoThumbnailGenerationFailed(
                file_path.clone(),
                format!("Failed to create async runtime: {}", e),
            ))?;

        let rgba_image = runtime.block_on(extract_first_frame(&file_path))
            .map_err(|e| NonCriticalThumbnailerError::VideoThumbnailGenerationFailed(
                file_path.clone(),
                e.to_string(),
            ))?;

        let mut img = DynamicImage::ImageRgba8(rgba_image);
        let (w, h) = img.dimensions();
        let (w_scaled, h_scaled) = scale_dimensions(w as f32, h as f32, TARGET_PX);

        if w != w_scaled && h != h_scaled {
            img = DynamicImage::ImageRgba8(imageops::resize(
                &img,
                w_scaled,
                h_scaled,
                imageops::FilterType::Triangle,
            ));
        }

        let encoder = Encoder::from_image(&img)
            .map_err(|_| NonCriticalThumbnailerError::WebPEncoding(
                file_path.clone(),
                "Failed to create WebP encoder".to_string(),
            ))?;

        let mut config = webp::WebPConfig::new()
            .map_err(|_| NonCriticalThumbnailerError::WebPEncoding(
                file_path.clone(),
                "Failed to create WebP config".to_string(),
            ))?;
        config.lossless = 0;
        config.alpha_compression = 1;
        config.quality = TARGET_QUALITY;

        let thumb = encoder.encode_advanced(&config)
            .map_err(|_| NonCriticalThumbnailerError::WebPEncoding(
                file_path.clone(),
                "WebP encoding failed".to_string(),
            ))?;

        std::fs::create_dir_all(output_path.parent().unwrap())
            .map_err(|e| NonCriticalThumbnailerError::SaveThumbnail(
                file_path.clone(),
                e.to_string(),
            ))?;

        std::fs::write(&output_path, &*thumb)
            .map_err(|e| NonCriticalThumbnailerError::SaveThumbnail(
                file_path,
                e.to_string(),
            ))?;

        Ok(())
    }).await
    .map_err(|e| NonCriticalThumbnailerError::PanicWhileGeneratingThumbnail(
        file_path_clone,
        e.to_string(),
    ))?
}