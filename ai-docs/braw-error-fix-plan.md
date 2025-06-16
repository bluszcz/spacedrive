# BRAW Runtime Error Fix Plan

**Timestamp:** 2025-06-16 19:30 UTC
**Author:** o3-AI Assistant
**Context:** Repeated runtime warnings:
```
VideoThumbnailGenerationFailed("…/A007_10190951_C001.braw", "Failed to open BRAW file: Invalid BRAW file format or corrupted file")
```

---

## 1  Problem Statement
`BrawSdk::open_clip()` currently fails on real BRAW files, causing thumbnail generation to log *Invalid BRAW file format or corrupted file*. Our stub returns this error because real SDK calls are not yet wired and resource format is unspecified.

---

## 2  Root-Cause Analysis (Summary)
| Potential Cause | Evidence | Fix Strategy |
|-----------------|----------|--------------|
| **SDK not truly initialised** – we return `initialized: false` even when SDK present | `initialize_native_sdk()` logs *loaded with stub bindings – native decoding is disabled* | Replace placeholder bindings with real ones; set `initialized = true` when factory+codec are valid |
| **Codec not created** – `codec` pointer is `null_mut()` | `open_clip()` check fails | Create codec via `factory.create_codec()` (see `blackmagic-raw-rs`) |
| **Resource format not specified** | No `set_resource_format` in our code | After reading frame, call `frame.set_resource_format(ResourceFormat::FORMAT_RGBAU8)` |
| **Two-stage job flow missing** | We attempt direct frame access | Implement read→decode job pipeline |

---

## 3  Fix Roadmap

### 3.1  Short-Term (Day 0-2)
1. **Integrate real bindings**
   - Enable `bindgen` against actual SDK headers (`BlackmagicRawAPI.h`).
   - Add `#[cfg(feature="real-braw-sdk")]` block; compile locally with env `BRAW_SDK_PATH`.
2. **Factory→Codec Pipeline**
   - `BrawSdk::initialize_native_sdk()`:
     ```rust
     factory = CreateBlackmagicRawFactoryInstance();
     codec   = (*factory).CreateCodec()?; // pseudo
     initialized = true;
     ```
3. **Clip Opening Logic**
   - Use `codec.open_clip(path)` APIs instead of test pattern.
4. **Graceful Fallback**
   - If SDK missing, *keep* stub but emit `SdkUnavailable`, not *InvalidFormat*.

### 3.2  Mid-Term (Day 3-5)
1. **Two-Stage Frame Extraction** (see *Lessons from blackmagic-raw-rs*)
   - Implement `create_job_read_frame` → `create_job_decode_and_process_frame`.
   - Provide async wrapper with `oneshot` callback.
2. **Resource Format Control**
   - Add `BrawResourceFormat` enum and default to `RGBAU8`.
3. **Unit Tests**
   - Use sample clip in `.data/test-clips/` to assert `extract_frame(0)` returns 3×W×H buffer.

### 3.3  Long-Term (Week 2)
1. **Performance Optimisation** – background thread pool, cached metadata.
2. **16-bit Support** – expose `RgbaU16` path for high-quality previews.
3. **Comprehensive Error Mapping** – convert SDK error codes to `BrawError` variants.

---

## 4  Acceptance Criteria
- `generate_braw_thumbnail()` produces a real image (non-placeholder) for at least three sample BRAW files on macOS.
- No `VideoThumbnailGenerationFailed` errors for valid BRAW files.
- Stub path still compiles & works on systems without SDK.
- Unit test `test_extract_real_frame` passes.

---

## 5  Task Breakdown & Ownership
| Task | Owner | ETA |
|------|-------|-----|
| Real SDK linking & codec creation | **bluszcz** | Day 1 |
| Clip open + read/decode jobs | **o3-AI assistant** | Day 2 |
| Resource format enum + helper | **o3-AI assistant** | Day 2 |
| Unit tests with sample clips | **bluszcz** | Day 4 |
| Docs update & CI matrix | **o3-AI assistant** | Day 5 |

---

## 6  References
- [blackmagic-raw-rs README](https://github.com/sportsball-ai/blackmagic-raw-rs) – job/ callback pattern.
- Internal doc: *BRAW Frame Extraction: Lessons from blackmagic-raw-rs* (§ in `ai-instructions.md`).

---

*End of plan.*
