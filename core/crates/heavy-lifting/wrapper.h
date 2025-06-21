// C wrapper header for BlackmagicRAW C++ API
#ifndef BLACKMAGIC_RAW_WRAPPER_H
#define BLACKMAGIC_RAW_WRAPPER_H

#include "BlackmagicRawAPI_patched.h"

#ifdef __cplusplus
extern "C" {
#endif

// Forward declare Variant for C compatibility
typedef struct Variant Variant;

// Factory functions
void* create_blackmagic_raw_factory_instance_from_path(const char* path);
long blackmagic_raw_factory_create_codec(void* factory_ptr, void** codec_ptr);
long blackmagic_raw_set_callback(void* codec_ptr, void* callback_ptr);

// Codec functions  
long blackmagic_raw_open_clip(void* codec_ptr, const char* filename, void** clip_ptr);
long blackmagic_raw_flush_jobs(void* codec_ptr);

// Clip functions
long blackmagic_raw_clip_get_width(void* clip_ptr, unsigned int* width);
long blackmagic_raw_clip_get_height(void* clip_ptr, unsigned int* height);
long blackmagic_raw_clip_get_frame_rate(void* clip_ptr, float* frame_rate);
long blackmagic_raw_clip_get_frame_count(void* clip_ptr, unsigned long long* frame_count);
long blackmagic_raw_clip_get_metadata_iterator(void* clip_ptr, void** iterator_ptr);
long blackmagic_raw_clip_create_job_read_frame(void* clip_ptr, unsigned long long frame_index, void** job_ptr);

// Metadata iterator functions
long blackmagic_raw_metadata_iterator_next(void* iterator_ptr);
long blackmagic_raw_metadata_iterator_get_key(void* iterator_ptr, void** key_ptr);
long blackmagic_raw_metadata_iterator_get_data(void* iterator_ptr, Variant* data);

// Job functions
long blackmagic_raw_job_submit(void* job_ptr);

// Frame reading with callback for metadata extraction
long blackmagic_raw_job_set_callback(void* job_ptr, void* callback_ptr);
long blackmagic_raw_frame_get_metadata_iterator(void* frame_ptr, void** iterator_ptr);
void* get_captured_frame_ptr();
bool is_frame_callback_completed();
void reset_frame_callback_state();

// Utility functions
void blackmagic_raw_unknown_release(void* ptr);
void* buffer_data(void* buffer_ptr);
void buffer_release(void* buffer_ptr);

// Variant functions
long blackmagic_raw_variant_init(Variant* variant);
long blackmagic_raw_variant_clear(Variant* variant);
long blackmagic_raw_variant_get_string(Variant* variant, void** string_ptr);

#ifdef __cplusplus
}
#endif

#endif // BLACKMAGIC_RAW_WRAPPER_H