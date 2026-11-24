//! Factor 5 MusyX 4-bit ADPCM / DPCM sample decompressor.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use crate::error::Result;

pub struct MusyXAdpcm;

/// Precomputed 16-entry delta lookup table for MusyX 4-bit DPCM/ADPCM nibbles.
///
/// For nibbles 0..7: positive deltas `(nibble * 2 + 1) * 32`
/// For nibbles 8..15: negative deltas `-((nibble & 7) * 2 + 1) * 32`
pub const ADPCM_DELTA_TABLE: [i16; 16] = [
    32, 96, 160, 224, 288, 352, 416, 480,
    -32, -96, -160, -224, -288, -352, -416, -480,
];

impl MusyXAdpcm {
    /// Decode a 4-bit ADPCM nibble stream into an output slice of 16-bit signed linear PCM.
    ///
    /// Suitable for `#![no_std]` environments without an allocator.
    /// Returns the number of samples decoded into `out`.
    pub fn decode_to_slice(input: &[u8], out: &mut [i16]) -> usize {
        let mut predictor: i32 = 0;
        let mut step_index: i32 = 0;

        let (out_chunks, out_rem) = out.as_chunks_mut::<2>();
        let pairs = out_chunks.len().min(input.len());

        for (&byte, chunk) in input[..pairs].iter().zip(&mut out_chunks[..pairs]) {
            let nibble_hi = (byte >> 4) & 0x0F;
            chunk[0] = Self::decode_nibble(nibble_hi, &mut predictor, &mut step_index);
            let nibble_lo = byte & 0x0F;
            chunk[1] = Self::decode_nibble(nibble_lo, &mut predictor, &mut step_index);
        }

        let mut idx = pairs * 2;
        if !out_rem.is_empty() && pairs < input.len() {
            let byte = input[pairs];
            let nibble_hi = (byte >> 4) & 0x0F;
            out_rem[0] = Self::decode_nibble(nibble_hi, &mut predictor, &mut step_index);
            idx += 1;
        }

        idx
    }

    /// Decode a 4-bit ADPCM nibble stream into 16-bit signed linear PCM.
    #[cfg(feature = "alloc")]
    pub fn decode(input: &[u8], sample_count: usize) -> Result<Vec<i16>> {
        let cap = sample_count.min(input.len().saturating_mul(2));
        let mut out = alloc::vec![0i16; cap];
        let decoded = Self::decode_to_slice(input, &mut out);
        out.truncate(decoded);
        Ok(out)
    }

    /// Decode a single 4-bit nibble into a 16-bit linear PCM sample.
    #[inline(always)]
    pub fn decode_nibble(nibble: u8, predictor: &mut i32, _step_index: &mut i32) -> i16 {
        let delta = ADPCM_DELTA_TABLE[(nibble & 0x0F) as usize];
        let p = (*predictor as i16).saturating_add(delta);
        *predictor = p as i32;
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_to_slice() {
        let data = [0x00];
        let mut out = [0i16; 2];
        let decoded = MusyXAdpcm::decode_to_slice(&data, &mut out);
        assert_eq!(decoded, 2);
        assert_eq!(out[0], 32);
        assert_eq!(out[1], 64);

        let sign_data = [0x80];
        let mut sign_out = [0i16; 2];
        let decoded_sign = MusyXAdpcm::decode_to_slice(&sign_data, &mut sign_out);
        assert_eq!(decoded_sign, 2);
        assert_eq!(sign_out[0], -32);
        assert_eq!(sign_out[1], 0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_empty_decode() {
        let pcm = MusyXAdpcm::decode(&[], 0).unwrap();
        assert!(pcm.is_empty());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_zero_nibbles() {
        // Nibble 0: positive delta = (0*2+1)*32 = 32
        // Two zero nibbles in one byte 0x00
        let data = [0x00];
        let pcm = MusyXAdpcm::decode(&data, 2).unwrap();
        assert_eq!(pcm.len(), 2);
        assert_eq!(pcm[0], 32);
        assert_eq!(pcm[1], 64);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_sign_nibbles() {
        // Nibble 0x8: sign=true, mag=0 -> delta = -32
        // Nibble 0x0: sign=false, mag=0 -> delta = +32
        let data = [0x80];
        let pcm = MusyXAdpcm::decode(&data, 2).unwrap();
        assert_eq!(pcm.len(), 2);
        assert_eq!(pcm[0], -32);
        assert_eq!(pcm[1], 0);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_sample_count_truncation() {
        let data = [0x12, 0x34];
        let pcm = MusyXAdpcm::decode(&data, 3).unwrap();
        assert_eq!(pcm.len(), 3);

        // Huge sample_count must not OOM-panic; capped to input.len() * 2
        let capped = MusyXAdpcm::decode(&data, usize::MAX).unwrap();
        assert_eq!(capped.len(), 4);
    }
}
