//! Error types for MusyX parsing and synthesis.

use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    InvalidMagic,
    UnsupportedVersion(u8),
    UnexpectedEof,
    InvalidProject,
    InvalidPool,
    InvalidSample(u32),
    CorruptedAdpcm,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Invalid MusyX container magic"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported MusyX version: {v}"),
            Self::UnexpectedEof => write!(f, "Unexpected end of file while parsing MusyX"),
            Self::InvalidProject => write!(f, "Invalid MusyX project header"),
            Self::InvalidPool => write!(f, "Invalid MusyX soundpool header"),
            Self::InvalidSample(id) => write!(f, "Invalid sample index {id}"),
            Self::CorruptedAdpcm => write!(f, "Corrupted MusyX 4-bit ADPCM sample stream"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
