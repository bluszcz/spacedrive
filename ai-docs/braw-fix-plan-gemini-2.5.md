# Revised BRAW Error Fix & Implementation Plan

**Timestamp:** 2025-06-16 20:00 UTC
**Author:** Gemini 2.5 Pro
**Context:** This plan revises and expands upon the previous `braw-error-fix-plan.md`, incorporating deeper insights from `blackmagic-raw-rs` to provide a more detailed, code-centric roadmap for fixing the `VideoThumbnailGenerationFailed` runtime error.

---

## 1. Objective
To eliminate the `VideoThumbnailGenerationFailed` error by fully implementing native BRAW frame extraction, ensuring that real, high-quality thumbnails are generated for BRAW files when the Blackmagic RAW SDK is present.

---

## 2. Core Problem & Architectural North Star

The current error, "Invalid BRAW file format or corrupted file," is a **symptom of our stub implementation**, not an actual issue with the BRAW files. Our immediate goal is to replace this misleading error with a fully functional SDK pipeline.

Our architecture will follow the **best practices** identified from [blackmagic-raw-rs](https://github.com/sportsball-ai/blackmagic-raw-rs):
- **Factory/Builder Pattern:** For safe and ergonomic SDK initialization.
- **Asynchronous Callback/Job System:** To integrate the SDK's C++-style async model into Rust's `async/await` paradigm.
- **Explicit Resource Format Control:** To ensure we get pixel data in the exact format we need.

---

## 3. Phased Implementation Plan

### Phase 1: True SDK Initialization & Codec Creation (The Foundation)
**Goal:** Make `BrawSdk::new()` successfully initialize the factory *and* the codec, returning a fully operational SDK object.

| Step | Action | Key Code/Pseudo-Code |
| :--- | :--- | :--- |
| **1.1** | **Update `build.rs`** | Enable `bindgen` to generate *real bindings* from `BlackmagicRawAPI.h` when the `native-ffi` feature is enabled. |
| **1.2** | **Implement `BrawSdk::new()`** | In `sdk.rs`, replace the stub initialization. The logic should create and store both the factory and codec handles. |
| | | ```rust |
| | | let factory: *mut IBlackmagicRawFactory = CreateBlackmagicRawFactoryInstance(); |
| | | if factory.is_null() { return Err(BrawError::SdkInitializationFailed); } |
| | | |
| | | let mut codec: *mut IBlackmagicRaw = std::ptr::null_mut(); |
| | | let result = ((*(*factory).vtable_).CreateCodec.unwrap())(factory, &mut codec); |
| | | if result != 0 { return Err(BrawError::CodecCreationFailed(result)); } |
| | | |
| | | self.initialized = true; |
| | | ``` |
| **1.3** | **Implement `Drop`** | Implement `Drop` for `BrawSdk` to correctly call `Release()` on the codec and factory handles, preventing resource leaks. This is critical for stability. |
| | | ```rust |
| | | fn drop(&mut self) { |
| | |     if !self.codec.is_null() { unsafe { ((*(*self.codec).vtable_).Release.unwrap())(self.codec); } } |
| | |     if !self.factory.is_null() { unsafe { ((*(*self.factory).vtable_).Release.unwrap())(self.factory); } } |
| | | } |
| | | ``` |

### Phase 2: End-to-End Frame Extraction (Vertical Slice)
**Goal:** Implement a fully working `extract_frame(0)` that returns a real `DynamicImage` from a BRAW file.

| Step | Action | Key Code/Pseudo-Code |
| :--- | :--- | :--- |
| **2.1** | **Implement `open_clip`** | In `BrawSdk::open_clip()`, use the real `self.codec.OpenClip()` method. |
| **2.2** | **Create Callback Bridge**| Create a `BrawCallbackHandler` struct that uses a `tokio::sync::oneshot::channel` to bridge the C++ callback to our `async` function. |
| **2.3** | **Implement Callbacks** | Implement the `IBlackmagicRawCallback` trait for our handler. |
| | | **`ReadComplete`**: `frame.SetResourceFormat(resourceFormat_RGBA_U8)`. Then, `frame.CreateJobDecodeAndProcessFrame().Submit()`. |
| | | **`ProcessComplete`**: Send the resulting `IBlackmagicRawProcessedImage` through the `oneshot` sender. Handle and send errors too. |
| **2.4** | **Implement `extract_frame`** | This public function will orchestrate the process. |
| | | ```rust |
| | | pub async fn extract_frame(&self, frame_idx: u64) -> Result<DynamicImage, BrawError> { |
| | |     let (tx, rx) = oneshot::channel(); |
| | |     let callback = BrawCallbackHandler::new(tx); |
| | |     self.codec.SetCallback(callback.as_com_ptr())?; |
| | |     self.clip.CreateJobReadFrame(frame_idx)?.Submit()?; |
| | |     self.codec.FlushJobs()?; |
| | | |
| | |     let processed_image = rx.await??; // Await result from callback |
| | |     convert_processed_to_dynamic(processed_image) |
| | | } |
| | | ``` |
| **2.5** | **Create Image Converter**| Write the `convert_processed_to_dynamic` helper to safely read the raw buffer from the `IBlackmagicRawProcessedImage` and create an `image::DynamicImage`. |

### Phase 3: Integration & Error Handling Refinement
**Goal:** Connect the working `extract_frame` into the thumbnail pipeline and replace misleading error messages.

| Step | Action | Notes |
| :--- | :--- | :--- |
| **3.1** | **Update Thumbnailer** | In `crates/braw/src/thumbnail.rs`, replace the placeholder generation with a call to our new, working `extract_frame` method. |
| **3.2** | **Refine `BrawError` Enum**| Add specific error variants: `SdkNotInitialized`, `CodecCreationFailed(i32)`, `ClipOpenFailed(i32)`, `FrameExtractionFailed(String)`. |
| **3.3** | **Update Heavy Lifting** | In `heavy-lifting`, map these new, specific `BrawError`s to `NonCriticalThumbnailerError`. The log should now show *why* it failed (e.g., "SDK Not Found" or "Clip Open Failed"), not "Invalid Format". |

---

## 4. Acceptance Criteria
- **Success:** Running Spacedrive via `./spacedrive_bluszcz.sh` generates **real thumbnails** for `.braw` files, visible in the UI.
- **No False Errors:** The log is free of `VideoThumbnailGenerationFailed` errors for valid BRAW files.
- **Graceful Fallback:** Running `cargo run` without the SDK installed compiles successfully and logs a clear, understandable message (e.g., "BRAW SDK not found, using placeholder thumbnails") instead of a format error.
- **Tests:** A new unit test that extracts a frame from a sample `.braw` file and verifies the image dimensions passes successfully.

---
*This revised plan provides a more granular, code-first roadmap that directly addresses the root causes of the error while building a more robust and maintainable `sd-braw` crate.*
