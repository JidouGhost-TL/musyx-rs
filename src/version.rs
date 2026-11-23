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
        // MusyX 2.0 uses MUSX project headers
        if data.windows(4).any(|w| w == b"MUSX") {
            return Some(Self::V2_0);
        }

        // MusyX 1.0 signatures
        let is_v1 = data.windows(5).any(|w| w == b"musyx" || w == b"MusyX")
            || data.windows(8).any(|w| w == b"Factor 5" || w == b"FACTOR 5")
            || data.windows(15).any(|w| w == b"sizeofmusyxram");

        if is_v1 {
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
