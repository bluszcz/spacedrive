#include <CoreFoundation/CoreFoundation.h>
#include <AVFoundation/AVFoundation.h>
#include <VideoToolbox/VideoToolbox.h>
#include <CoreVideo/CoreVideo.h>
#include <CoreMedia/CoreMedia.h>
#include <CoreGraphics/CoreGraphics.h>
#include <ImageIO/ImageIO.h>
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
        std::cerr << "🎯 ProRes RAW: Starting frame extraction with AVAssetImageGenerator" << std::endl;
        
        // Convert C string to NSString
        NSString* filePath = [NSString stringWithUTF8String:file_path];
        NSURL* fileURL = [NSURL fileURLWithPath:filePath];
        
        // Create AVAsset
        AVAsset* asset = [AVAsset assetWithURL:fileURL];
        if (!asset) {
            std::cerr << "❌ Failed to create AVAsset from file: " << file_path << std::endl;
            return -1;
        }
        
        std::cerr << "✅ Created AVAsset successfully" << std::endl;
        
        // Create AVAssetImageGenerator - this is Apple's dedicated thumbnail API
        AVAssetImageGenerator* imageGenerator = [AVAssetImageGenerator assetImageGeneratorWithAsset:asset];
        if (!imageGenerator) {
            std::cerr << "❌ Failed to create AVAssetImageGenerator" << std::endl;
            return -2;
        }
        
        // Configure the image generator for high quality
        imageGenerator.appliesPreferredTrackTransform = YES;
        imageGenerator.maximumSize = CGSizeMake(4096, 4096); // High quality for ProRes RAW
        imageGenerator.requestedTimeToleranceBefore = kCMTimeZero;
        imageGenerator.requestedTimeToleranceAfter = kCMTimeZero;
        
        std::cerr << "✅ Configured AVAssetImageGenerator for ProRes RAW" << std::endl;
        
        // Generate thumbnail at time zero (first frame)
        NSError* error = nil;
        CMTime requestedTime = CMTimeMake(frame_number, 25); // Assume 25fps, adjust as needed
        if (frame_number == 0) {
            requestedTime = kCMTimeZero;
        }
        
        std::cerr << "🎯 Extracting frame " << frame_number << " using AVAssetImageGenerator..." << std::endl;
        
        // Use modern async API wrapped in synchronous dispatch
        __block CGImageRef imageRef = NULL;
        __block NSError* blockError = nil;
        dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
        
        [imageGenerator generateCGImageAsynchronouslyForTime:requestedTime completionHandler:^(CGImageRef _Nullable image, CMTime /* actualTime */, NSError * _Nullable error) {
            imageRef = image;
            if (imageRef) {
                CGImageRetain(imageRef); // Retain the image since it will be used outside the block
            }
            blockError = error;
            dispatch_semaphore_signal(semaphore);
        }];
        
        // Wait for completion
        dispatch_semaphore_wait(semaphore, DISPATCH_TIME_FOREVER);
        dispatch_release(semaphore);
        
        error = blockError;
        
        if (!imageRef) {
            if (error) {
                std::cerr << "❌ AVAssetImageGenerator failed: " << error.localizedDescription.UTF8String << std::endl;
            } else {
                std::cerr << "❌ AVAssetImageGenerator returned nil image" << std::endl;
            }
            return -3;
        }
        
        std::cerr << "✅ Successfully extracted CGImage from ProRes RAW!" << std::endl;
        
        // Get image dimensions
        size_t imageWidth = CGImageGetWidth(imageRef);
        size_t imageHeight = CGImageGetHeight(imageRef);
        *width = (int)imageWidth;
        *height = (int)imageHeight;
        
        std::cerr << "📐 Image dimensions: " << imageWidth << "x" << imageHeight << std::endl;
        
        // Create a bitmap context to extract RGBA data
        size_t bytesPerPixel = 4; // RGBA
        size_t bytesPerRow = imageWidth * bytesPerPixel;
        *data_size = imageHeight * bytesPerRow;
        
        // Allocate memory for the pixel data
        *data = (unsigned char*)malloc(*data_size);
        if (!*data) {
            std::cerr << "❌ Failed to allocate memory for pixel data" << std::endl;
            CGImageRelease(imageRef);
            return -4;
        }
        
        // Create color space and bitmap context
        CGColorSpaceRef colorSpace = CGColorSpaceCreateDeviceRGB();
        if (!colorSpace) {
            std::cerr << "❌ Failed to create RGB color space" << std::endl;
            free(*data);
            CGImageRelease(imageRef);
            return -5;
        }
        
        CGContextRef context = CGBitmapContextCreate(*data, 
                                                    imageWidth, 
                                                    imageHeight, 
                                                    8, // bits per component
                                                    bytesPerRow, 
                                                    colorSpace,
                                                    kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
        
        CGColorSpaceRelease(colorSpace);
        
        if (!context) {
            std::cerr << "❌ Failed to create bitmap context" << std::endl;
            free(*data);
            CGImageRelease(imageRef);
            return -6;
        }
        
        // Draw the CGImage into our bitmap context to get RGBA pixel data
        CGContextDrawImage(context, CGRectMake(0, 0, imageWidth, imageHeight), imageRef);
        
        std::cerr << "✅ Successfully converted ProRes RAW to RGBA pixel data!" << std::endl;
        
        // Cleanup
        CGContextRelease(context);
        CGImageRelease(imageRef);
        
        std::cerr << "🎉 ProRes RAW frame extraction completed successfully!" << std::endl;
        return 0; // Success!
    }
}

void free_frame_data(unsigned char* data) {
    if (data) {
        free(data);
    }
}

} // extern "C"