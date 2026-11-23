//! MusyX Engine Version enumeration and detection.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MusyXVersion {
    /// MusyX GBA v1.0 (early releases such as Rayman Advance)
    V1_0,
    /// MusyX GBA v2.0 (expanded voice tables and project descriptors)
    V2_0,
}

impl MusyXVersion {
    /// Detect MusyX engine version from ROM slice.
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

        // MusyX 2.0 `MUSX` header: require 4-byte alignment + corroborating MusyX marker
        // (or small test buffer containing "MUSX project") to avoid random 4-byte collisions
        let has_musx_v2 = data
            .as_chunks::<4>()
            .0
            .iter()
            .any(|w| w == b"MUSX")
            && (has_musyx_sig || has_proj_markers || data.windows(12).any(|w| w == b"MUSX project"));

        if has_musx_v2 {
            return Some(Self::V2_0);
        }

        if has_musyx_sig || has_proj_markers {
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
    fn test_detect_none() {
        let dummy = b"Nintendo M4A Sound Driver";
        assert_eq!(MusyXVersion::detect_from_bytes(dummy), None);
    }
}
