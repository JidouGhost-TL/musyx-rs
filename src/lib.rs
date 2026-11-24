//! # musyx-rs
//!
//! Pure Rust, zero-dependency parser, ADPCM decoder, software synthesizer, and golden-tested
//! emulator for the **Factor 5 MusyX Sound System** on the Game Boy Advance.
//!
//! MusyX was created by Factor 5 (Chris Hülsbeck and the Factor 5 audio engineering team),
//! providing high-end audio capabilities across the GBA library, including
//! *Rayman Advance*, *Star Wars: Jedi Power Battles*, *Contra Advance*, and *Top Gear Rally*.
//!
//! ## Features
//! - `#![no_std]` compatible (allocator required for dynamic soundpool parsing).
//! - Deterministic 4-bit ADPCM / DPCM sample decompression.
//! - Dual synthesis pipelines:
//!   - `AccuracySynth`: Bit-exact fixed-point software mixer matching the ARM driver's DMA ring buffer.
//!   - `StudioSynth`: Audiophile 48 kHz 32-bit floating point rendering with band-limited sinc interpolation.
//! - Golden test suite verified against cycle-accurate mGBA ARM driver execution traces.
//!
//! ## Quickstart
//!
//! ```rust
//! use musyx::{AccuracySynth, MusyXAdpcm, MusyXVoice};
//!
//! // Zero-allocation 4-bit ADPCM decompression (`#![no_std]`)
//! let adpcm_nibbles: &[u8] = &[0x01, 0x23, 0x89, 0xAB];
//! let mut pcm16 = [0i16; 8];
//! let decoded = MusyXAdpcm::decode_to_slice(adpcm_nibbles, &mut pcm16);
//! assert_eq!(decoded, 8);
//!
//! // Mix into stereo 16-bit buffers
//! let mut voice = MusyXVoice::new(0, 0x1000, 255, 255);
//! let mut left = [0i16; 8];
//! let mut right = [0i16; 8];
//! AccuracySynth::mix_voice_pcm16(&pcm16, &mut voice, &mut left, &mut right);
//! assert_eq!(left, right);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod adpcm;
pub mod error;
pub mod sample;
pub mod sequence;
pub mod synth;
pub mod version;

pub use adpcm::MusyXAdpcm;
pub use error::{Error, Result};
pub use sample::{MusyXSample, MusyXSampleFormat, MusyXSampleView};
pub use sequence::{MusyXEvent, MusyXPattern, MusyXSong};
pub use synth::{AccuracySynth, MusyXVoice, StudioSynth};
pub use version::MusyXVersion;
