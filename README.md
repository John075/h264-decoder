# H.264 Decoder – v0.1 TODO List

---

## Module: `bitstream`

> Focus on stateless parsing utilities and NALU interpreters.

- [x] Implement `BitReader` (peek, rewind, read_bits, read_ue/se)
- [x] Parse Annex B NALU start codes (`0x000001` / `0x00000001`)
- [ ] Add support for length-prefixed NALUs (MP4-style)
- [ ] Parse NALU header → `Nalu { nal_ref_idc, nal_unit_type }`
- [ ] Parse Sequence Parameter Set → `Sps` struct
- [ ] Parse Picture Parameter Set → `Pps` struct
- [ ] Parse Slice Header → `SliceHeader` struct
- [ ] Add metadata-dump mode (print SPS/PPS/slice info to stdout)

---

## Module: `parser`

> Wraps bitstream parsing, manages NALU stream state.

- [ ] Feed raw byte stream → sequence of parsed NALUs
- [ ] Maintain active SPS/PPS by ID
- [ ] Return slice-ready structs: (SliceHeader, Sps, Pps)
- [ ] Filter unsupported features early (e.g., CABAC, B-frames)
- [ ] Provide ordered frames to the decoder
- [ ] Implement basic stream-level error handling and recovery

---

## Module: `decoder`

> Responsible for actual picture decoding (I/P slices, Baseline profile).

- [ ] Implement CAVLC entropy decoder
- [ ] Decode I-slices:
  - [ ] Intra prediction (4x4, 16x16, chroma)
  - [ ] Dequantization + inverse 4x4 transform
  - [ ] Reconstruct macroblocks into YUV buffer
- [ ] Decode P-slices:
  - [ ] Motion vector parsing
  - [ ] Forward motion compensation
  - [ ] Handle skipped macroblocks
- [ ] Implement simple reference frame buffer (last frame only)
- [ ] Perform frame-level reconstruction
- [ ] Write raw `.yuv` output (I420 format)

---

## Module: `cli`

> Entry point, connects everything, enables testing and visualization.

- [ ] Read input file (`--input`) and output path (`--output`)
- [ ] CLI flag: `--dump` (print SPS/PPS/slice info)
- [ ] CLI flag: `--stats` (frame count, resolution, bitrate)
- [ ] CLI help screen (`--help`)
- [ ] Print errors and warnings
- [ ] Optionally show decode time per frame

---

## Final Integration & Validation

- [ ] Create test files:
  - [ ] I-frame only stream
  - [ ] I/P-frame Baseline Profile stream
  - [ ] Length-prefixed stream (MP4-style)
- [ ] Compare `.yuv` output against FFmpeg reference
- [ ] Confirm compatibility with SPS/PPS from real-world bitstreams
- [ ] Tag `v0.1` release once I/P decoding and YUV output are stable
