# O3 AI Implementation Plan – BRAW Support for Spacedrive

> Version: 2025-06-14  
> Author: o3 – OpenAI Coding Assistant  
> Scope: End-to-end roadmap for adding BlackmagicRAW (BRAW) support to Spacedrive.

---

## 1. Goal
Enable seamless handling of `.braw` files inside Spacedrive, including:
1. Accurate file-type detection (extension + magic bytes).
2. Robust metadata extraction.
3. High-performance thumbnail generation.
4. (Stretch) Preview & playback hooks for future UI work.

## 2. Guiding Principles
* Follow Spacedrive's existing modular architecture.  
* Rust-first, SDK isolated into its own crate (`crates/braw`).  
* Cross-platform (macOS, Windows, Linux) binary distribution strategy.  
* Async, memory-safe, and non-blocking processing.  
* Respect BlackmagicRAW SDK licensing terms.  
* Comply with user repository rules: reusable code, `.gitignore`, `ai-instructions.md`, data in `.data/`.

## 3. Deliverables
1. `crates/braw` – FFI bindings + safe wrapper.
2. Updates to `crates/file-ext` for detection.
3. `media-metadata` integration with `braw` feature flag.
4. Thumbnail pipeline extension.
5. Comprehensive tests & docs.
6. CI updates + pre-built SDK artifacts (where licensing allows).

## 4. Work Breakdown Structure

### Phase 0 — Project Hygiene
* Add/extend `.gitignore` (SDK binaries ⇒ ignored, `.data/` for temp assets).  
* Append BRAW context to `ai-instructions.md` (memory bank).

### Phase 1 — Research & SDK Scaffolding
1. Download latest BlackmagicRAW SDK; store archives in `.data/sdk` (git-ignored).
2. Explore headers & samples; map required symbols.
3. Prototype `bindgen` script in `braw/build.rs` to generate Rust FFI.
4. Validate linking on all OS targets via minimal `cargo test`.

### Phase 2 — File-Type Detection
1. Extend `VideoExtension` enum with `Braw`.  
2. Implement magic-byte probe (likely `42 52 41 57` – verify with samples).
3. Unit tests with sample files in `.data/samples`.

### Phase 3 — Safe Wrapper & Metadata API
1. Build `BrawFile` abstraction (open, close, drop).
2. Expose `BrawMetadata` struct mirroring common & camera-specific tags.  
3. Convert SDK error codes → `thiserror`.
4. Add async helper that offloads heavy I/O to blocking thread pool.

### Phase 4 — Thumbnail Extraction
1. Use SDK's frame decode to grab nearest I-frame or first frame.  
2. Transcode to RGB using SDK utilities; convert to `image::DynamicImage`.
3. Integrate into thumbnail service with either on-demand or background queue.

### Phase 5 — Media-Metadata Integration
1. Add `braw` feature gate in workspace.
2. Hook extraction path in `media-metadata` factory.
3. Extend Prisma schema / migration if new fields emerge (e.g., ISO, LUT info).

### Phase 6 — Testing & QA
* Unit: detection, metadata, thumbnail.  
* Integration: cross-platform sample corpus, corrupted file scenarios.  
* Performance: decode latency, memory consumption per GB.  
* Fuzzing: feed malformed headers to ensure safe failure.

### Phase 7 — Packaging & Distribution
1. Amend CI matrix: build with `--features braw` when SDK present.  
2. For OSS builds without SDK, compile with `--no-default-features` to exclude BRAW.  
3. Ship README section on enabling BRAW, license caveats, and environment variables for SDK path overrides.

## 5. Timeline (Realistic Buffer)
| Week | Focus | Key Milestones |
|------|-------|----------------|
| 1 | Ph 0-1 | SDK linked, FFI generated, CI smoke-green |
| 2 | Ph 2 | Detection passes unit tests |
| 3 | Ph 3 | Metadata extracted for sample clips |
| 4 | Ph 4 | Thumbnails display in UI |
| 5 | Ph 5 | Database stores BRAW metadata |
| 6 | Ph 6-7 | QA, docs, release PR |

Slack time is included; risks may extend schedule by 1–2 wks.

## 6. Risk Register
| Risk | Impact | Mitigation |
|------|--------|-----------|
| SDK license changes | High | Keep feature optional; build only when user sets `BRAW_SDK_PATH`. |
| Platform-specific linking errors | Medium | Separate CI job per target; containerised builds with known toolchains. |
| Large file RAM usage | Medium | Stream decode, drop frames after thumbnail, use block cache. |
| SDK API instability | Low | Encapsulate all unsafe calls behind narrow surface; integration tests per SDK update. |

## 7. Open Questions
1. Preferred user experience for failed decode (fallback icon vs blank).  
2. Whether to expose RAW parameter controls in Phase 1 or defer.  
3. Distribution of `.dylib/.dll/.so` – embed or user-installed?

## 8. Next Steps
* [ ] Confirm sample file set & licensing.  
* [ ] Verify magic bytes across multiple camera firmware versions.  
* [ ] Draft `crates/braw` crate skeleton & push branch `feature/braw`.

---

*End of document.* 