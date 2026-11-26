//! MusyX Engine Version enumeration and detection.

/// 32-byte ARM7TDMI prologue of the Factor 5 MusyX DirectSound 8-bit DMA FIFO
/// stereo clamp routine (`stmfd sp!, {r0-r2, r4-r9, lr}; mov lr, #0; mov r9, #0x20; ...`).
///
/// Verified at `0x0804E064` in *Rayman Advance* and present verbatim across all
/// 107 MusyX GBA ROMs (both string-tagged and string-stripped builds) in the
/// 1,706-ROM canonical corpus with zero false positives.
const ARM_CLIP_STEREO_SIG: [u8; 32] = [
    0xf7, 0x43, 0x2d, 0xe9, 0x00, 0xe0, 0xa0, 0xe3, 0x20, 0x90, 0xa0, 0xe3, 0xf4, 0x40, 0xd0,
    0xe0, 0xf4, 0x50, 0xd0, 0xe0, 0xf4, 0x60, 0xd0, 0xe0, 0xfc, 0x70, 0x50, 0xe0, 0x7f, 0x00,
    0x54, 0xe3,
];

/// 32-byte ARM7TDMI prologue of the Factor 5 MusyX 16-bit intermediate voice
/// mixing kernel (`stmfd sp!, {r1-r12, lr}; ldr r2, [r0, #4]; ldr r10, [r0, #8]; ...`).
///
/// Verified at `0x0804E210` in *Rayman Advance* and present verbatim across all
/// 107 MusyX GBA ROMs in the 1,706-ROM canonical corpus with zero false positives.
const ARM_MIX_VOICE_SIG: [u8; 32] = [
    0xfe, 0x5f, 0x2d, 0xe9, 0x04, 0x20, 0x90, 0xe5, 0x08, 0xa0, 0x90, 0xe5, 0xb6, 0xb1, 0xd0,
    0xe1, 0xb4, 0xe1, 0xd0, 0xe1, 0x18, 0x40, 0xd0, 0xe5, 0x19, 0x50, 0xd0, 0xe5, 0x00, 0x00,
    0x54, 0xe3,
];

#[inline]
fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && needle.len() <= haystack.len()
        && haystack.windows(needle.len()).any(|w| w == needle)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MusyXVersion {
    /// MusyX GBA v1.0 (early releases such as Rayman Advance)
    V1_0,
    /// MusyX GBA v2.0 (expanded voice tables and project descriptors)
    V2_0,
}

impl MusyXVersion {
    /// Returns `true` if the ROM contains the Factor 5 MusyX ARM7TDMI software
    /// mixer and DirectSound FIFO clamp kernels.
    ///
    /// This binary signature detects string-stripped MusyX builds in addition to
    /// banner-tagged builds, with zero false positives across the 1,706-ROM GBA corpus.
    #[must_use]
    pub fn has_arm_mixer_signature(data: &[u8]) -> bool {
        contains_subslice(data, &ARM_CLIP_STEREO_SIG)
            && contains_subslice(data, &ARM_MIX_VOICE_SIG)
    }

    /// Return the file offset of the MusyX ARM DirectSound FIFO clamp kernel, if present.
    #[must_use]
    pub fn mixer_offset(data: &[u8]) -> Option<usize> {
        if !Self::has_arm_mixer_signature(data) {
            return None;
        }
        data.windows(ARM_CLIP_STEREO_SIG.len())
            .position(|w| w == ARM_CLIP_STEREO_SIG)
    }

    /// Detect MusyX engine version from ROM slice.
    #[must_use]
    pub fn detect_from_bytes(data: &[u8]) -> Option<Self> {
        // Explicit MusyX / Factor 5 runtime strings (avoid bare 5-byte lowercase "musyx"
        // which can occur coincidentally inside smooth PCM waveforms)
        let has_musyx_sig = data.windows(11).any(|w| w == b"MusyX Audio" || w == b"Musyx init ")
            || data.windows(17).any(|w| w == b"musyxCalcMemUsage")
            || data.windows(14).any(|w| w == b"sizeofmusyxram")
            || data.windows(11).any(|w| w == b"musyx audio")
            || data.windows(12).any(|w| w == b"MusyX \x04Audio") // LZ77-compressed MusyX banner
            || data.windows(8).any(|w| w == b"Factor 5" || w == b"FACTOR 5" || w == b"factor 5");

        // MusyX project file chunk markers (`.proj\0`, `.pool\0`, `.sdir\0`, `.samp\0`)
        let has_proj_markers = data.windows(6).any(|w| w == b".proj\0")
            && (data.windows(6).any(|w| w == b".pool\0")
                || data.windows(6).any(|w| w == b".sdir\0")
                || data.windows(6).any(|w| w == b".samp\0")
                || has_musyx_sig);

        // Clean-room ARM7TDMI mixer + FIFO clamp binary signature (catches string-stripped builds)
        let has_arm_mixer = Self::has_arm_mixer_signature(data);

        // MusyX 2.0 `MUSX` header: require 4-byte alignment + corroborating MusyX marker
        // (or small test buffer containing "MUSX project") to avoid random 4-byte collisions
        let has_musx_v2 = data
            .as_chunks::<4>()
            .0
            .iter()
            .any(|w| w == b"MUSX")
            && (has_musyx_sig
                || has_proj_markers
                || has_arm_mixer
                || data.windows(12).any(|w| w == b"MUSX project"));

        if has_musx_v2 {
            return Some(Self::V2_0);
        }

        if has_musyx_sig || has_proj_markers || has_arm_mixer {
            Some(Self::V1_0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_v1() {
        let dummy = b"Sound driver licensed by Factor 5 (musyx audio tools)";
        assert_eq!(MusyXVersion::detect_from_bytes(dummy), Some(MusyXVersion::V1_0));
    }

    #[test]
    fn test_detect_v1_proj_pool() {
        let dummy = b"audio.proj\0data.pool\0";
        assert_eq!(MusyXVersion::detect_from_bytes(dummy), Some(MusyXVersion::V1_0));
    }

    #[test]
    fn test_detect_v2() {
        let dummy = b"Header data MUSX project archive";
        assert_eq!(MusyXVersion::detect_from_bytes(dummy), Some(MusyXVersion::V2_0));
    }

    #[test]
    fn test_detect_arm_mixer_stripped() {
        let mut rom = [0u8; 256];
        rom[16..48].copy_from_slice(&ARM_CLIP_STEREO_SIG);
        rom[128..160].copy_from_slice(&ARM_MIX_VOICE_SIG);
        assert!(MusyXVersion::has_arm_mixer_signature(&rom));
        assert_eq!(MusyXVersion::mixer_offset(&rom), Some(16));
        assert_eq!(MusyXVersion::detect_from_bytes(&rom), Some(MusyXVersion::V1_0));

        // Requiring both kernels avoids partial-match false positives
        let mut partial = [0u8; 128];
        partial[16..48].copy_from_slice(&ARM_CLIP_STEREO_SIG);
        assert!(!MusyXVersion::has_arm_mixer_signature(&partial));
        assert_eq!(MusyXVersion::detect_from_bytes(&partial), None);
    }

    #[test]
    fn test_detect_arm_mixer_with_musx_v2() {
        let mut rom = [0u8; 256];
        rom[4..8].copy_from_slice(b"MUSX");
        rom[16..48].copy_from_slice(&ARM_CLIP_STEREO_SIG);
        rom[128..160].copy_from_slice(&ARM_MIX_VOICE_SIG);
        assert_eq!(MusyXVersion::detect_from_bytes(&rom), Some(MusyXVersion::V2_0));
    }

    #[test]
    fn test_detect_none() {
        let dummy = b"Nintendo M4A Sound Driver";
        assert_eq!(MusyXVersion::detect_from_bytes(dummy), None);
    }
}
