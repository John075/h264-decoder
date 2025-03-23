# h264-decoder
A memory-safe, high-performance H.264 decoder written in Rust, eliminating the need for C/C++ FFI bindings
---

## Project Setup
- [ ] Create workspace structure (bitstream, parser, decoder, cli)
- [ ] Minimal CLI interface with file input
- [ ] Define core structs: Frame, NALU, SPS, PPS

---

## Bitstream Parsing
- [ ] Implement bit-level reader (bit-by-bit with peek/back)
- [ ] Handle NALU start code parsing (0x000001 / 0x00000001)
- [ ] Parse and categorize NALU types
- [ ] Implement Exp-Golomb decoding (unsigned + signed)
- [ ] Parse SPS (resolution, profile, chroma_format, etc.)
- [ ] Parse PPS (CABAC, transform flags, etc.)
- [ ] Parse slice headers (type, frame_num, ref_idx, etc.)
- [ ] Support for length-prefixed NALU streams (MP4-style)
- [ ] Reuse SPS/PPS across frames
- [ ] Implement basic error handling and stream recovery
- [ ] Add metadata dump mode (output SPS/PPS/slice info to stdout)

---

## Baseline Profile – I-Frames Only
- [ ] Parse I-slices
- [ ] Implement CAVLC entropy decoding
- [ ] Implement 4x4 and 16x16 intra prediction (luma)
- [ ] Implement chroma intra prediction
- [ ] Implement dequantization
- [ ] Implement inverse 4x4 transform
- [ ] Handle frame padding and boundary alignment
- [ ] Reconstruct macroblocks into YUV frame buffer
- [ ] Write raw .yuv output (I420)

---

## Add P-Frame Support
- [ ] Parse P-slices and inter macroblocks
- [ ] Parse motion vectors
- [ ] Implement forward motion compensation
- [ ] Implement reference frame buffering
- [ ] Handle skipped macroblocks
- [ ] Add inter macroblock reconstruction

---

## Add B-Frame Support
- [ ] Parse B-slices and reference lists
- [ ] Implement bi-directional motion compensation
- [ ] Add decoded picture buffer (DPB) and frame reordering
- [ ] Support Picture Order Count (POC) logic

---

## CABAC Support (Main Profile)
- [ ] Implement CABAC arithmetic decoder core
- [ ] Build context model table and mapping
- [ ] Decode macroblock types, motion data, and coefficients
- [ ] Integrate CABAC into the slice decoding path

---

## High Profile – Phase 1
- [ ] Parse and handle `transform_8x8_mode_flag`
- [ ] Add support for 8x8 luma transform
- [ ] Implement inverse 8x8 transform
- [ ] Parse and apply custom scaling matrices
- [ ] Support chroma QP scaling and offset
- [ ] Reject unsupported chroma formats (4:2:2, 4:4:4)

---

## Output + Tooling
- [ ] Implement CLI: `decode_h264 input.h264 -o output.yuv`
- [ ] Add YUV → RGB converter (optional PNG frame dumps)
- [ ] Add performance logging (decode time, frame count, resolution)
- [ ] Build a test harness with sample .h264 clips
- [ ] Implement playback timing based on SPS timing info

---

## Polishing
- [ ] Implement deblocking filter
- [ ] Add multi-threaded slice/frame decoding
- [ ] Enable real-time decode from stream (stdin / TCP)
- [ ] Compare output to FFmpeg (PSNR or frame diff)
- [ ] Add simple logging/debug mode (frame count, slice info, type)

---

## Testing & Validation
- [ ] Unit test: bitstream reader
- [ ] Unit test: Exp-Golomb parser
- [ ] Unit test: SPS/PPS parsing
- [ ] Unit test: motion vectors
- [ ] Validate output against FFmpeg-generated .yuv
- [ ] Create a known-good sample test suite
- [ ] Build a bitstream compatibility test matrix (baseline, main, high)
- [ ] Stub or handle unsupported features cleanly (e.g., slice groups/FMO)

---

## Final Checks
- [ ] Add example usage and include a sample .h264 test case
- [ ] Provide CLI help (`--help`) and usage examples
- [ ] Update README with benchmark results and build instructions
- [ ] Document compatibility and known limitations
- [ ] Tag `v0.1` MVP release (I/P with CAVLC)
- [ ] Tag `v1.0` full release (I/P/B with CABAC and High Profile Phase 1)

---

## Continuous Integration / Deployment
- [ ] Set up a CI pipeline (e.g., GitHub Actions, Travis CI) to run tests, linting (rustfmt/Clippy), and build checks on every commit

---

## Documentation & Developer Experience
- [ ] Write detailed developer documentation (design docs, architecture diagrams, API documentation)
- [ ] Create a CONTRIBUTING.md file and a CODE_OF_CONDUCT
- [ ] Add inline code comments and public API documentation

---

## Licensing & Project Setup
- [ ] Update the README with an overview, roadmap, and installation instructions

---

## Demo & Outreach
- [ ] Create a demo video or animated GIF showcasing key features and performance benchmarks
- [ ] Write technical article explaining the design and implementation
---

## Post-v1.0 Roadmap
- [ ] Outline future features and improvements (e.g., advanced profiling, additional profile support)
- [ ] Create an issues board or project board detailing long-term plans and enhancements
