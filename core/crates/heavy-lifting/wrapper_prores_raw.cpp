#include <CoreFoundation/CoreFoundation.h>
#include <AVFoundation/AVFoundation.h>
#include <VideoToolbox/VideoToolbox.h>
#include <CoreVideo/CoreVideo.h>
#include <CoreMedia/CoreMedia.h>
#include <iostream>
#include <cstring>

extern "C" {

int extract_prores_raw_frame(
    const char* file_path,
    int frame_number,
    int* width,
    int* height,
    unsigned char** data,
    size_t* data_size
) {
    @autoreleasepool {
        // Convert C string to NSString
        NSString* filePath = [NSString stringWithUTF8String:file_path];
        NSURL* fileURL = [NSURL fileURLWithPath:filePath];
        
        // Create AVAsset
        AVAsset* asset = [AVAsset assetWithURL:fileURL];
        if (!asset) {
            std::cerr << "Failed to create AVAsset from file: " << file_path << std::endl;
            return -1;
        }
        
        // Get video tracks
        NSArray<AVAssetTrack*>* videoTracks = [asset tracksWithMediaType:AVMediaTypeVideo];
        if (videoTracks.count == 0) {
            std::cerr << "No video tracks found in file: " << file_path << std::endl;
            return -2;
        }
        
        AVAssetTrack* videoTrack = videoTracks[0];
        CGSize naturalSize = videoTrack.naturalSize;
        
        *width = (int)naturalSize.width;
        *height = (int)naturalSize.height;
        
        // Create asset reader
        NSError* error = nil;
        AVAssetReader* reader = [[AVAssetReader alloc] initWithAsset:asset error:&error];
        if (!reader || error) {
            std::cerr << "Failed to create AVAssetReader: " << error.localizedDescription.UTF8String << std::endl;
            return -3;
        }
        
        // Create output settings for RGBA format
        NSDictionary* outputSettings = @{
            (NSString*)kCVPixelBufferPixelFormatTypeKey: @(kCVPixelFormatType_32RGBA)
        };
        
        // Create asset reader output
        AVAssetReaderTrackOutput* readerOutput = [[AVAssetReaderTrackOutput alloc] 
                                                   initWithTrack:videoTrack 
                                                   outputSettings:outputSettings];
        
        if (![reader canAddOutput:readerOutput]) {
            std::cerr << "Cannot add reader output" << std::endl;
            return -4;
        }
        
        [reader addOutput:readerOutput];
        
        // Start reading
        if (![reader startReading]) {
            std::cerr << "Failed to start reading" << std::endl;
            return -5;
        }
        
        // Skip to desired frame
        int currentFrame = 0;
        CMSampleBufferRef sampleBuffer = nil;
        
        while (currentFrame <= frame_number) {
            sampleBuffer = [readerOutput copyNextSampleBuffer];
            if (!sampleBuffer) {
                std::cerr << "Failed to get sample buffer for frame " << currentFrame << std::endl;
                return -6;
            }
            
            if (currentFrame == frame_number) {
                break;
            }
            
            CFRelease(sampleBuffer);
            currentFrame++;
        }
        
        if (!sampleBuffer) {
            std::cerr << "No sample buffer for frame " << frame_number << std::endl;
            return -7;
        }
        
        // Get CVImageBuffer from sample buffer
        CVImageBufferRef imageBuffer = CMSampleBufferGetImageBuffer(sampleBuffer);
        if (!imageBuffer) {
            std::cerr << "Failed to get image buffer" << std::endl;
            CFRelease(sampleBuffer);
            return -8;
        }
        
        // Lock the pixel buffer
        OSType pixelFormat = CVPixelBufferGetPixelFormatType(imageBuffer);
        if (pixelFormat != kCVPixelFormatType_32RGBA) {
            std::cerr << "Unexpected pixel format: " << pixelFormat << std::endl;
            CFRelease(sampleBuffer);
            return -9;
        }
        
        CVPixelBufferLockBaseAddress(imageBuffer, kCVPixelBufferLock_ReadOnly);
        
        // Get pixel data
        void* baseAddress = CVPixelBufferGetBaseAddress(imageBuffer);
        size_t bytesPerRow = CVPixelBufferGetBytesPerRow(imageBuffer);
        size_t bufferHeight = CVPixelBufferGetHeight(imageBuffer);
        size_t bufferWidth = CVPixelBufferGetWidth(imageBuffer);
        
        // Calculate expected data size (RGBA = 4 bytes per pixel)
        *data_size = bufferWidth * bufferHeight * 4;
        
        // Allocate memory for the output data
        *data = (unsigned char*)malloc(*data_size);
        if (!*data) {
            std::cerr << "Failed to allocate memory for frame data" << std::endl;
            CVPixelBufferUnlockBaseAddress(imageBuffer, kCVPixelBufferLock_ReadOnly);
            CFRelease(sampleBuffer);
            return -10;
        }
        
        // Copy pixel data row by row (handling potential padding)
        unsigned char* src = (unsigned char*)baseAddress;
        unsigned char* dst = *data;
        size_t rowSize = bufferWidth * 4; // 4 bytes per pixel (RGBA)
        
        for (size_t row = 0; row < bufferHeight; row++) {
            memcpy(dst + (row * rowSize), src + (row * bytesPerRow), rowSize);
        }
        
        // Update actual dimensions
        *width = (int)bufferWidth;
        *height = (int)bufferHeight;
        
        // Cleanup
        CVPixelBufferUnlockBaseAddress(imageBuffer, kCVPixelBufferLock_ReadOnly);
        CFRelease(sampleBuffer);
        
        return 0; // Success
    }
}

void free_frame_data(unsigned char* data) {
    if (data) {
        free(data);
    }
}

} // extern "C"