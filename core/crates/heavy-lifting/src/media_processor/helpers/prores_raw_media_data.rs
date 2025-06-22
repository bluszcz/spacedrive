use crate::media_processor;

use sd_media_metadata::{
    ffmpeg::{
        codec::{Codec, Props},
        metadata::Metadata,
        program::Program,
        stream::Stream,
        video_props::VideoProps,
    },
    FFmpegMetadata,
};

use std::{collections::HashMap, path::Path};


pub async fn extract(
    path: impl AsRef<Path> + Send,
) -> Result<FFmpegMetadata, media_processor::NonCriticalMediaProcessorError> {
    let path = path.as_ref();

    // For ProRes RAW, we'll create minimal metadata compatible with FFmpeg schema
    // since we can't extract full metadata like we do with BRAW
    let formats = vec!["mov".to_string()];
    
    // Set basic metadata - these would ideally come from file analysis
    let duration = Some((0, 0)); // Would need to analyze file for actual duration
    let start_time = Some((0, 0));
    let bit_rate = (0, 0); // Would need to analyze file for actual bitrate
    
    let metadata = Metadata {
        title: None,
        artist: None,
        album: None,
        album_artist: None,
        composer: None,
        genre: None,
        creation_time: None,
        disc: None,
        track: None,
        encoder: Some("Apple ProRes RAW".to_string()),
        language: None,
        custom: HashMap::new(),
        ..Default::default()
    };

    // Create a minimal video stream for ProRes RAW
    let video_props = VideoProps {
        pixel_format: Some("prores_raw".to_string()),
        color_range: None,
        bits_per_channel: Some(16), // ProRes RAW is typically 16-bit
        color_space: None,
        color_primaries: None,
        color_transfer: None,
        field_order: Some("progressive".to_string()),
        chroma_location: None,
        width: 5760, // Typical for Ninja Ultra - would need to extract from file
        height: 4320, // Typical for Ninja Ultra - would need to extract from file
        aspect_ratio_num: Some(1),
        aspect_ratio_den: Some(1),
        properties: vec!["prores_raw".to_string()],
    };

    let video_codec = Codec {
        kind: Some("video".to_string()),
        sub_kind: Some("prores_raw".to_string()),
        tag: Some("aprn".to_string()),
        name: Some("Apple ProRes RAW".to_string()),
        profile: None,
        bit_rate: 0,
        props: Some(Props::Video(video_props)),
    };

    let video_stream = Stream {
        id: 0,
        name: None,
        codec: Some(video_codec),
        aspect_ratio_num: 1,
        aspect_ratio_den: 1,
        frames_per_second_num: 25, // Typical for Ninja Ultra - would need to extract
        frames_per_second_den: 1,
        time_base_real_den: 25,
        time_base_real_num: 1,
        dispositions: vec!["default".to_string()],
        metadata: Metadata {
            title: None,
            artist: None,
            album: None,
            album_artist: None,
            composer: None,
            genre: None,
            creation_time: None,
            disc: None,
            track: None,
            encoder: Some("Apple ProRes RAW".to_string()),
            language: Some("eng".to_string()),
            custom: HashMap::new(),
            ..Default::default()
        },
    };

    let program = Program {
        id: 0,
        name: None,
        streams: vec![video_stream],
        metadata: Metadata::default(),
    };

    Ok(FFmpegMetadata {
        formats,
        duration,
        start_time,
        bit_rate,
        chapters: vec![],
        programs: vec![program],
        metadata,
    })
}