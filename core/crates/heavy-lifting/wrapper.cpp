// C wrapper for BlackmagicRAW C++ API - The Viking bridge!
#include "/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Include/BlackmagicRawAPI.h"
#include "wrapper.h"

// Global frame pointer for capturing frames
static void* g_frame_ptr = nullptr;
static bool g_frame_completed = false;

// Global state for image extraction
static bool g_extraction_completed = false;
static HRESULT g_extraction_result = 0;
static unsigned int g_image_width = 0;
static unsigned int g_image_height = 0;
static unsigned int g_image_data_size = 0;
static void* g_image_data = nullptr;

// Frame callback implementation
class FrameCallback : public IBlackmagicRawCallback {
public:
    void ReadComplete(IBlackmagicRawJob* /*job*/, HRESULT result, IBlackmagicRawFrame* frame) {
        if (SUCCEEDED(result) && frame) {
            g_frame_ptr = frame;
            frame->AddRef(); // Keep reference for metadata extraction
            
            // Set output format to RGBA8 and decode
            frame->SetResourceFormat(blackmagicRawResourceFormatRGBAU8);
            
            // Create decode job
            IBlackmagicRawJob* decodeJob = nullptr;
            result = frame->CreateJobDecodeAndProcessFrame(nullptr, nullptr, &decodeJob);
            if (SUCCEEDED(result) && decodeJob) {
                decodeJob->Submit();
                decodeJob->Release();
            }
        } else {
            g_frame_completed = true;
            g_extraction_completed = true;
            g_extraction_result = result;
        }
    }
    
    void ProcessComplete(IBlackmagicRawJob* /*processJob*/, HRESULT result, IBlackmagicRawProcessedImage* processedImage) {
        if (SUCCEEDED(result) && processedImage) {
            processedImage->GetWidth(&g_image_width);
            processedImage->GetHeight(&g_image_height);
            processedImage->GetResourceSizeBytes(&g_image_data_size);
            processedImage->GetResource(&g_image_data);
        }
        
        g_frame_completed = true;
        g_extraction_completed = true;
        g_extraction_result = result;
    }
    
    void DecodeComplete(IBlackmagicRawJob*, HRESULT) {}
    void TrimProgress(IBlackmagicRawJob*, float) {}
    void TrimComplete(IBlackmagicRawJob*, HRESULT) {}
    void SidecarMetadataParseWarning(IBlackmagicRawClip*, CFStringRef, uint32_t, CFStringRef) {}
    void SidecarMetadataParseError(IBlackmagicRawClip*, CFStringRef, uint32_t, CFStringRef) {}
    void PreparePipelineComplete(void*, HRESULT) {}
    
    // IUnknown methods
    HRESULT QueryInterface(REFIID, void**) { return E_NOINTERFACE; }
    ULONG AddRef() { return 1; }
    ULONG Release() { return 1; }
};

static FrameCallback g_frame_callback;

extern "C" {

// Factory functions
void* create_blackmagic_raw_factory_instance_from_path(const char* path) {
    // Use basic factory instance since FromPath version isn't exported
    IBlackmagicRawFactory* factory = CreateBlackmagicRawFactoryInstance();
    return factory;
}

long blackmagic_raw_factory_create_codec(void* factory_ptr, void** codec_ptr) {
    IBlackmagicRawFactory* factory = static_cast<IBlackmagicRawFactory*>(factory_ptr);
    IBlackmagicRaw** codec = reinterpret_cast<IBlackmagicRaw**>(codec_ptr);
    return factory->CreateCodec(codec);
}

long blackmagic_raw_set_callback(void* codec_ptr, void* callback_ptr) {
    IBlackmagicRaw* codec = static_cast<IBlackmagicRaw*>(codec_ptr);
    return codec->SetCallback(&g_frame_callback);
}

// Codec functions
long blackmagic_raw_open_clip(void* codec_ptr, const char* filename, void** clip_ptr) {
    IBlackmagicRaw* codec = static_cast<IBlackmagicRaw*>(codec_ptr);
    CFStringRef filenameRef = CFStringCreateWithCString(kCFAllocatorDefault, filename, kCFStringEncodingUTF8);
    IBlackmagicRawClip** clip = reinterpret_cast<IBlackmagicRawClip**>(clip_ptr);
    HRESULT result = codec->OpenClip(filenameRef, clip);
    CFRelease(filenameRef);
    return result;
}

long blackmagic_raw_flush_jobs(void* codec_ptr) {
    IBlackmagicRaw* codec = static_cast<IBlackmagicRaw*>(codec_ptr);
    return codec->FlushJobs();
}

// Clip functions
long blackmagic_raw_clip_get_width(void* clip_ptr, unsigned int* width) {
    IBlackmagicRawClip* clip = static_cast<IBlackmagicRawClip*>(clip_ptr);
    return clip->GetWidth(width);
}

long blackmagic_raw_clip_get_height(void* clip_ptr, unsigned int* height) {
    IBlackmagicRawClip* clip = static_cast<IBlackmagicRawClip*>(clip_ptr);
    return clip->GetHeight(height);
}

long blackmagic_raw_clip_get_frame_rate(void* clip_ptr, float* frame_rate) {
    IBlackmagicRawClip* clip = static_cast<IBlackmagicRawClip*>(clip_ptr);
    return clip->GetFrameRate(frame_rate);
}

long blackmagic_raw_clip_get_frame_count(void* clip_ptr, unsigned long long* frame_count) {
    IBlackmagicRawClip* clip = static_cast<IBlackmagicRawClip*>(clip_ptr);
    return clip->GetFrameCount(frame_count);
}

long blackmagic_raw_clip_get_metadata_iterator(void* clip_ptr, void** iterator_ptr) {
    IBlackmagicRawClip* clip = static_cast<IBlackmagicRawClip*>(clip_ptr);
    IBlackmagicRawMetadataIterator** iterator = reinterpret_cast<IBlackmagicRawMetadataIterator**>(iterator_ptr);
    return clip->GetMetadataIterator(iterator);
}

long blackmagic_raw_clip_create_job_read_frame(void* clip_ptr, unsigned long long frame_index, void** job_ptr) {
    IBlackmagicRawClip* clip = static_cast<IBlackmagicRawClip*>(clip_ptr);
    IBlackmagicRawJob** job = reinterpret_cast<IBlackmagicRawJob**>(job_ptr);
    return clip->CreateJobReadFrame(frame_index, job);
}

// Metadata iterator functions
long blackmagic_raw_metadata_iterator_next(void* iterator_ptr) {
    IBlackmagicRawMetadataIterator* iterator = static_cast<IBlackmagicRawMetadataIterator*>(iterator_ptr);
    return iterator->Next();
}

long blackmagic_raw_metadata_iterator_get_key(void* iterator_ptr, void** key_ptr) {
    IBlackmagicRawMetadataIterator* iterator = static_cast<IBlackmagicRawMetadataIterator*>(iterator_ptr);
    CFStringRef key = nullptr;
    HRESULT result = iterator->GetKey(&key);
    *key_ptr = const_cast<void*>(static_cast<const void*>(key));
    return result;
}

long blackmagic_raw_metadata_iterator_get_data(void* iterator_ptr, Variant* data) {
    IBlackmagicRawMetadataIterator* iterator = static_cast<IBlackmagicRawMetadataIterator*>(iterator_ptr);
    return iterator->GetData(data);
}

// Job functions
long blackmagic_raw_job_submit(void* job_ptr) {
    IBlackmagicRawJob* job = static_cast<IBlackmagicRawJob*>(job_ptr);
    return job->Submit();
}

// Frame reading functions

long blackmagic_raw_job_set_callback(void* job_ptr, void* callback_ptr) {
    // IBlackmagicRawJob doesn't have SetCallback - this is handled at codec level
    // For now, return success - the callback is already set at a higher level
    return 0; // S_OK
}

long blackmagic_raw_frame_get_metadata_iterator(void* frame_ptr, void** iterator_ptr) {
    IBlackmagicRawFrame* frame = static_cast<IBlackmagicRawFrame*>(frame_ptr);
    IBlackmagicRawMetadataIterator** iterator = reinterpret_cast<IBlackmagicRawMetadataIterator**>(iterator_ptr);
    return frame->GetMetadataIterator(iterator);
}

void* get_captured_frame_ptr() {
    return g_frame_ptr;
}

bool is_frame_callback_completed() {
    return g_frame_completed;
}

void reset_frame_callback_state() {
    g_frame_completed = false;
    if (g_frame_ptr) {
        static_cast<IUnknown*>(g_frame_ptr)->Release();
        g_frame_ptr = nullptr;
    }
}

// Utility functions
void blackmagic_raw_unknown_release(void* ptr) {
    IUnknown* unknown = static_cast<IUnknown*>(ptr);
    if (unknown) {
        unknown->Release();
    }
}

// Buffer functions (simplified)
void* buffer_data(void* buffer_ptr) {
    // This is a placeholder - actual implementation would depend on Buffer type
    return buffer_ptr;
}

void buffer_release(void* buffer_ptr) {
    // This is a placeholder - actual implementation would depend on Buffer type
    if (buffer_ptr) {
        CFRelease(static_cast<CFTypeRef>(buffer_ptr));
    }
}

// Variant functions
long blackmagic_raw_variant_init(Variant* variant) {
    return VariantInit(variant);
}

long blackmagic_raw_variant_clear(Variant* variant) {
    return VariantClear(variant);
}

long blackmagic_raw_variant_get_string(Variant* variant, void** string_ptr) {
    if (variant->vt == blackmagicRawVariantTypeString) {
        *string_ptr = const_cast<void*>(static_cast<const void*>(variant->bstrVal));
        return 0; // S_OK
    }
    return -1; // E_FAIL
}

// Image extraction functions
bool is_extraction_completed() {
    return g_extraction_completed;
}

long get_extraction_result() {
    return g_extraction_result;
}

void reset_extraction_state() {
    g_extraction_completed = false;
    g_extraction_result = 0;
    g_image_width = 0;
    g_image_height = 0;
    g_image_data_size = 0;
    g_image_data = nullptr;
}

unsigned int get_extracted_image_width() {
    return g_image_width;
}

unsigned int get_extracted_image_height() {
    return g_image_height;
}

void* get_extracted_image_data() {
    return g_image_data;
}

} // extern "C"