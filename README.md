# musyx-rs 🎼

**Zero-dependency, pure Rust parser, ADPCM decoder, software synthesizer, and golden-tested emulator for the Factor 5 MusyX Sound System (Game Boy Advance).**

[![Crates.io](https://img.shields.io/crates/v/musyx.svg)](https://crates.io/crates/musyx)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](#features)

The **MusyX Sound System** was created by Chris Hülsbeck and the Factor 5 audio engineering team at **Factor 5**. Revered for its advanced digital signal processing capabilities, MusyX powered high-end GBA soundtracks including *Rayman Advance*, *Star Wars: Jedi Power Battles*, *Contra Advance*, and *Top Gear Rally*.

`musyx-rs` provides a clean-room, reverse-engineered implementation designed for audio preservation, DAWs, PC source ports, and emulators.

---

## Features

- **Zero External Dependencies**: Pure Rust with no dependencies in `[dependencies]`.
- **`#![no_std]` Compatible**: Ready for embedded targets and WASM runtimes (`alloc` required for dynamic soundpool parsing).
- **Deterministic 4-Bit ADPCM Decoder**: Bit-exact integer arithmetic decompression matching Factor 5's ARM assembly routines.
- **Dual Synthesis Engines**:
  - **`AccuracySynth`**: Emulates the original fixed-point math, 16-bit intermediate voice mixing, and hardware sample rates, matching the ARM driver's DMA ring buffer.
  - **`StudioSynth`**: Renders at 48,000 Hz, 32-bit floating-point precision with 64-tap band-limited sinc interpolation and true stereo panning.
- **Golden Tested**: Every ADPCM block, envelope slope, and event dispatcher is verified against cycle-accurate ARM execution traces captured from mGBA.

---

## Quickstart

### Zero-Allocation 4-bit ADPCM Decompression (`#![no_std]`)

```rust
use musyx::MusyXAdpcm;

let adpcm_nibbles: &[u8] = &[0x01, 0x23, 0x89, 0xAB];
let mut pcm16_output = [0i16; 8];

let decoded_count = MusyXAdpcm::decode_to_slice(adpcm_nibbles, &mut pcm16_output);
assert_eq!(decoded_count, 8);
```

### Zero-Allocation Synthesizer Mixing

```rust
use musyx::{AccuracySynth, MusyXSampleView, MusyXVoice};

let pcm_bytes: &[u8] = &[0x00, 0x10, 0x20, 0x10, 0x00, 0xF0, 0xE0, 0xF0];
let sample = MusyXSampleView::from_pcm8(0, 11025, pcm_bytes, None);

let mut voice = MusyXVoice::new(0, 0x1000, 255, 255);
let mut pcm16_buf = [0i16; 64];
let mut left = [0i16; 64];
let mut right = [0i16; 64];

sample.decode_to_pcm16_slice(&mut pcm16_buf).unwrap();
AccuracySynth::mix_voice_pcm16(&pcm16_buf, &mut voice, &mut left, &mut right);
```

---

## Golden Test Methodology

To ensure 100% mathematical accuracy without guesswork:
1. **Tier 1 (Bit-Exact ADPCM)**: Decompressed sample buffers are bit-matched against the ARM decompressor output in mGBA.
2. **Tier 2 (Sequencer State Transitions)**: Voice allocations, MIDI controller changes, pitch bends, and ADSR envelopes are asserted against the ARM driver's RAM state at every 60 Hz frame.
3. **Tier 3 (End-to-End Stream Correlation)**: The software mixer output is correlated against raw PCM dumped from hardware DMA registers (`REG_FIFO_A` and `REG_FIFO_B`).

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
