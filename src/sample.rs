//! MusyX Sample structures and descriptors.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusyXSampleFormat {
    Pcm8,
    Pcm16,
    Adpcm4,
}

/// Borrowed zero-allocation view of a MusyX sample descriptor and audio payload.
///
/// Enables `#![no_std]` applications to play samples directly from ROM or static buffers
/// without requiring a heap allocator (`feature = "alloc"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MusyXSampleView<'a> {
    pub id: u32,
    pub format: MusyXSampleFormat,
    pub sample_rate: u32,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
    pub raw_bytes: &'a [u8],
}

impl<'a> MusyXSampleView<'a> {
    pub const fn new(id: u32, format: MusyXSampleFormat, sample_rate: u32, raw_bytes: &'a [u8]) -> Self {
        Self {
            id,
            format,
            sample_rate,
            loop_start: None,
            loop_end: None,
            raw_bytes,
        }
    }

    /// Construct a MusyX 8-bit PCM sample view from raw byte slice.
    pub const fn from_pcm8(id: u32, sample_rate: u32, data: &'a [u8], loop_start: Option<u32>) -> Self {
        Self {
            id,
            format: MusyXSampleFormat::Pcm8,
            sample_rate,
            loop_start,
            loop_end: None,
            raw_bytes: data,
        }
    }

    /// Parse a standard 16-byte MusyX sample header followed by sample payload.
    ///
    /// Panic-free: performs bounds check and direct byte array conversion without `.unwrap()`.
    pub fn from_header_block(id: u32, data: &'a [u8]) -> crate::error::Result<Self> {
        if data.len() < 16 {
            return Err(crate::error::Error::UnexpectedEof);
        }
        let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let loop_raw = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let loop_start = if loop_raw == 0xFFFF_FFFF {
            None
        } else {
            Some(loop_raw)
        };
        let sample_rate = u16::from_le_bytes([data[8], data[9]]) as u32;
        let fmt_raw = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let format = match fmt_raw {
            0 => MusyXSampleFormat::Pcm8,
            1 => MusyXSampleFormat::Pcm16,
            2 => MusyXSampleFormat::Adpcm4,
            _ => MusyXSampleFormat::Pcm8,
        };

        let payload_len = length.min(data.len() - 16);
        let raw_bytes = &data[16..16 + payload_len];

        Ok(Self {
            id,
            format,
            sample_rate,
            loop_start,
            loop_end: None,
            raw_bytes,
        })
    }

    /// Decode or expand sample data into a caller-provided 16-bit PCM buffer.
    ///
    /// Operates without heap allocation, suitable for `#![no_std]` embedded execution.
    /// Returns the number of 16-bit samples written into `out`.
    pub fn decode_to_pcm16_slice(&self, out: &mut [i16]) -> crate::error::Result<usize> {
        match self.format {
            MusyXSampleFormat::Pcm8 => {
                let count = self.raw_bytes.len().min(out.len());
                for (dst, &src) in out.iter_mut().zip(&self.raw_bytes[..count]) {
                    *dst = (src as i8 as i16) << 8;
                }
                Ok(count)
            }
            MusyXSampleFormat::Pcm16 => {
                let sample_count = (self.raw_bytes.len() / 2).min(out.len());
                for (i, dst) in out[..sample_count].iter_mut().enumerate() {
                    let b0 = self.raw_bytes[i * 2];
                    let b1 = self.raw_bytes[i * 2 + 1];
                    *dst = i16::from_le_bytes([b0, b1]);
                }
                Ok(sample_count)
            }
            MusyXSampleFormat::Adpcm4 => {
                let decoded = crate::adpcm::MusyXAdpcm::decode_to_slice(self.raw_bytes, out);
                Ok(decoded)
            }
        }
    }

    /// Convert sample to signed 16-bit PCM for synthesis.
    #[cfg(feature = "alloc")]
    pub fn to_pcm16(&self) -> crate::error::Result<Vec<i16>> {
        match self.format {
            MusyXSampleFormat::Pcm8 => {
                // Expand signed 8-bit to 16-bit by shifting left 8 bits
                Ok(self.raw_bytes.iter().map(|&b| (b as i8 as i16) << 8).collect())
            }
            MusyXSampleFormat::Pcm16 => {
                let sample_count = self.raw_bytes.len() / 2;
                let mut out = Vec::with_capacity(sample_count);
                for i in 0..sample_count {
                    let b0 = self.raw_bytes[i * 2];
                    let b1 = self.raw_bytes[i * 2 + 1];
                    out.push(i16::from_le_bytes([b0, b1]));
                }
                Ok(out)
            }
            MusyXSampleFormat::Adpcm4 => {
                crate::adpcm::MusyXAdpcm::decode(self.raw_bytes, self.raw_bytes.len().saturating_mul(2))
            }
        }
    }

    pub const fn len(&self) -> usize {
        self.raw_bytes.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.raw_bytes.is_empty()
    }

    #[cfg(feature = "alloc")]
    pub fn to_owned_sample(&self) -> MusyXSample {
        MusyXSample {
            id: self.id,
            format: self.format,
            sample_rate: self.sample_rate,
            loop_start: self.loop_start,
            loop_end: self.loop_end,
            raw_bytes: self.raw_bytes.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MusyXSample {
    pub id: u32,
    pub format: MusyXSampleFormat,
    pub sample_rate: u32,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
    #[cfg(feature = "alloc")]
    pub raw_bytes: Vec<u8>,
}

impl MusyXSample {
    #[cfg(feature = "alloc")]
    pub fn new(id: u32, format: MusyXSampleFormat, sample_rate: u32, raw_bytes: Vec<u8>) -> Self {
        Self {
            id,
            format,
            sample_rate,
            loop_start: None,
            loop_end: None,
            raw_bytes,
        }
    }

    /// Construct a MusyX 8-bit PCM sample from raw byte slice.
    #[cfg(feature = "alloc")]
    pub fn from_pcm8(id: u32, sample_rate: u32, data: &[u8], loop_start: Option<u32>) -> Self {
        Self {
            id,
            format: MusyXSampleFormat::Pcm8,
            sample_rate,
            loop_start,
            loop_end: None,
            raw_bytes: data.to_vec(),
        }
    }

    /// Parse a standard 16-byte MusyX sample header followed by sample payload.
    ///
    /// Panic-free: delegates to `MusyXSampleView::from_header_block`.
    #[cfg(feature = "alloc")]
    pub fn from_header_block(id: u32, data: &[u8]) -> crate::error::Result<Self> {
        MusyXSampleView::from_header_block(id, data).map(|v| v.to_owned_sample())
    }

    /// Borrowed view of this sample.
    #[cfg(feature = "alloc")]
    pub fn as_view(&self) -> MusyXSampleView<'_> {
        MusyXSampleView {
            id: self.id,
            format: self.format,
            sample_rate: self.sample_rate,
            loop_start: self.loop_start,
            loop_end: self.loop_end,
            raw_bytes: &self.raw_bytes,
        }
    }

    /// Convert sample to signed 16-bit PCM for synthesis.
    #[cfg(feature = "alloc")]
    pub fn to_pcm16(&self) -> crate::error::Result<Vec<i16>> {
        self.as_view().to_pcm16()
    }

    #[cfg(feature = "alloc")]
    pub fn len(&self) -> usize {
        self.raw_bytes.len()
    }

    #[cfg(feature = "alloc")]
    pub fn is_empty(&self) -> bool {
        self.raw_bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_view_no_alloc() {
        let pcm8 = [0x00, 0x01, 0x7f, 0x80, 0xff];
        let view = MusyXSampleView::from_pcm8(0, 11025, &pcm8, Some(2));
        assert_eq!(view.len(), 5);
        assert!(!view.is_empty());
        assert_eq!(view.loop_start, Some(2));

        let mut out = [0i16; 5];
        let decoded = view.decode_to_pcm16_slice(&mut out).unwrap();
        assert_eq!(decoded, 5);
        assert_eq!(out[0], 0);
        assert_eq!(out[1], 256);
        assert_eq!(out[2], 127 << 8);
        assert_eq!(out[3], (-128i16) << 8);
        assert_eq!(out[4], (-1i16) << 8);
    }

    #[test]
    fn test_sample_view_header_block() {
        // Construct a synthetic 16-byte header + 4 byte payload:
        // len=4, loop=0xFFFFFFFF (None), rate=22050, fmt=0 (Pcm8)
        let mut block = [0u8; 20];
        block[0..4].copy_from_slice(&4u32.to_le_bytes());
        block[4..8].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        block[8..10].copy_from_slice(&22050u16.to_le_bytes());
        block[12..16].copy_from_slice(&0u32.to_le_bytes());
        block[16..20].copy_from_slice(&[10, 20, 30, 40]);

        let view = MusyXSampleView::from_header_block(7, &block).unwrap();
        assert_eq!(view.id, 7);
        assert_eq!(view.format, MusyXSampleFormat::Pcm8);
        assert_eq!(view.sample_rate, 22050);
        assert_eq!(view.loop_start, None);
        assert_eq!(view.len(), 4);
        assert_eq!(view.raw_bytes, &[10, 20, 30, 40]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_pcm8_to_pcm16_expansion() {
        let pcm8 = [0x00, 0x01, 0x7f, 0x80, 0xff];
        let sample = MusyXSample::from_pcm8(0, 11025, &pcm8, Some(2));
        assert_eq!(sample.len(), 5);
        assert_eq!(sample.loop_start, Some(2));
        let pcm16 = sample.to_pcm16().unwrap();
        assert_eq!(pcm16[0], 0);
        assert_eq!(pcm16[1], 256);
        assert_eq!(pcm16[2], 127 << 8);
        assert_eq!(pcm16[3], (-128i16) << 8);
        assert_eq!(pcm16[4], (-1i16) << 8);

        let view = sample.as_view();
        let roundtrip = view.to_owned_sample();
        assert_eq!(roundtrip, sample);
    }
}
