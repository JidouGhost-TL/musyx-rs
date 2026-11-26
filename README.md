# musyx-rs 🎼

**Zero-dependency, pure Rust parser, ADPCM decoder, software synthesizer, and golden-tested emulator for the Factor 5 MusyX Sound System (Game Boy Advance).**

[![Crates.io](https://img.shields.io/crates/v/musyx.svg)](https://crates.io/crates/musyx)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](#features)

The **MusyX Sound System** was created by Chris Hülsbeck and the Factor 5 audio engineering team at **Factor 5**. Revered for its advanced digital signal processing capabilities, MusyX powered high-end GBA soundtracks including *Rayman Advance*, *Star Wars: Jedi Power Battles*, *Contra Advance*, and *Top Gear Rally*.

`musyx-rs` provides a clean-room, reverse-engineered implementation designed for audio preservation, DAWs, PC source ports, and emulators.

> **Looking for a ready-to-use CLI?** You can scan GBA ROMs, decode ADPCM/PCM WAV samples, and compile SoundFont 2.04 (`.sf2`) banks directly from the command line using our **[gbakit](https://github.com/JidouGhost-TL/gbakit)** toolkit (`gbakit audio scan`, `gbakit audio export`, `gbakit audio soundfont`).

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

To ensure 100% mathematical accuracy without guesswork (SHA-256 manifest in `tests/golden/manifest.json`):
1. **Tier 1 (Bit-Exact 4-Bit ADPCM & Sample Headers)**: 1,024-sample (`2,048`-byte) ADPCM decompression golden including full `+32767` / `-32768` saturation boundaries, plus 3 *Rayman Advance* sample fixtures and 16-byte header-block parsing.
2. **Tier 2 (Driver Versions & Hardware Rates)**: Version detection (`MusyXVersion::V1_0` / `V2_0`) and GBA Timer 0 rate constants.
3. **Tier 3 (Bit-Exact ARM Mixer & DirectSound DMA FIFO)**: Strict byte-for-byte (`assert_eq!`) verification of 128-frame stereo 16-bit intermediate mixing (`512` bytes, ARM `0x0804E210`) and DirectSound 8-bit DMA FIFO clamping (`256` bytes, ARM `0x0804E064`).

---

## Benchmarks

```sh
cargo bench --bench bench_audio
```

| Benchmark | Throughput |
| --- | ---: |
| `MusyXAdpcm::decode_to_slice` (1,024 samples) | ~839 MSamples/s |
| `AccuracySynth::mix_voice_pcm16` (1:1 pitch, 1,024f) | ~2.13 GSamples/s |
| `AccuracySynth::mix_voice_pcm16` (resampled `0x1500`, 1,024f) | ~455 MSamples/s |
| `AccuracySynth::mix_voice_pcm8` (1:1 pitch, 1,024f) | ~2.10 GSamples/s |
| `AccuracySynth::mix_voice_pcm8` (resampled `0x1500`, 1,024f) | ~489 MSamples/s |

---

## Community & Support

Come discuss the wonders of GBA RE, modding, etc. — also the best way to report issues or bugs, or ask for help on your own project: https://discord.gg/khwxFSVNrS

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
