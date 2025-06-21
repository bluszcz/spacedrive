use super::braw_decoder::{BrawDecoder, BrawMetadata};
use crate::media_processor;

use sd_media_metadata::FFmpegMetadata;

use std::path::Path;

#[must_use]
pub const fn can_extract_for_braw() -> bool {
    true
}

pub async fn extract(
    path: impl AsRef<Path> + Send,
) -> Result<FFmpegMetadata, media_processor::NonCriticalMediaProcessorError> {
    let path = path.as_ref();
    
    let decoder = BrawDecoder::new(path)
        .map_err(|e| media_processor::NonCriticalMediaProcessorError::from(
            media_processor::tasks::media_data_extractor::NonCriticalMediaDataExtractorError::FailedToExtractImageMediaData(
                path.to_path_buf(),
                e.to_string(),
            )
        ))?;

    let braw_metadata = decoder.extract_metadata()
        .map_err(|e| media_processor::NonCriticalMediaProcessorError::from(
            media_processor::tasks::media_data_extractor::NonCriticalMediaDataExtractorError::FailedToExtractImageMediaData(
                path.to_path_buf(),
                e.to_string(),
            )
        ))?;

    Ok(braw_to_ffmpeg_metadata(braw_metadata))
}

fn braw_to_ffmpeg_metadata(braw: BrawMetadata) -> FFmpegMetadata {
    use sd_media_metadata::ffmpeg::{
        codec::{Codec, Props},
        metadata::Metadata,
        program::Program,
        stream::Stream,
        video_props::VideoProps,
    };

    let duration = (braw.frame_count as f32 / braw.frame_rate) as u64;
    let duration_tuple = ((duration >> 32) as i32, duration as u32);

    let bit_rate = 50_000_000u64; // Estimate 50Mbps for BRAW
    let bit_rate_tuple = ((bit_rate >> 32) as i32, bit_rate as u32);

    let video_props = VideoProps {
        pixel_format: Some("bayer_rggb16le".to_string()),
        color_range: Some("tv".to_string()),
        bits_per_channel: Some(16),
        color_space: Some("rec709".to_string()),
        color_primaries: Some("bt709".to_string()),
        color_transfer: Some("bt709".to_string()),
        field_order: Some("progressive".to_string()),
        chroma_location: Some("left".to_string()),
        width: braw.width as i32,
        height: braw.height as i32,
        aspect_ratio_num: Some(braw.width as i32),
        aspect_ratio_den: Some(braw.height as i32),
        properties: vec!["raw".to_string(), "blackmagic".to_string()],
    };

    let codec = Codec {
        kind: Some("video".to_string()),
        sub_kind: Some("raw".to_string()),
        tag: Some("BRAW".to_string()),
        name: Some("Blackmagic RAW".to_string()),
        profile: Some("raw".to_string()),
        bit_rate: bit_rate as i32,
        props: Some(Props::Video(video_props)),
    };

    let stream = Stream {
        id: 0,
        name: Some("Video".to_string()),
        codec: Some(codec),
        aspect_ratio_num: braw.width as i32,
        aspect_ratio_den: braw.height as i32,
        frames_per_second_num: (braw.frame_rate * 1000.0) as i32,
        frames_per_second_den: 1000,
        time_base_real_den: (braw.frame_rate * 1000.0) as i32,
        time_base_real_num: 1000,
        dispositions: vec![],
        metadata: Metadata::default(),
    };

    let program = Program {
        id: 0,
        name: Some("BRAW Program".to_string()),
        streams: vec![stream],
        metadata: Metadata::default(),
    };


    FFmpegMetadata {
        formats: vec!["braw".to_string()],
        duration: Some(duration_tuple),
        start_time: Some((0, 0)),
        bit_rate: bit_rate_tuple,
        chapters: vec![],
        programs: vec![program],
        metadata: Metadata::default(),
    }
}