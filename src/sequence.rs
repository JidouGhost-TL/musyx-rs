//! MusyX Song sequences and MIDI events.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MusyXEvent {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    ProgramChange { channel: u8, program: u8 },
    PitchWheel { channel: u8, bend: i16 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    WaitTicks(u16),
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MusyXPattern {
    pub id: u32,
    #[cfg(feature = "alloc")]
    pub events: Vec<MusyXEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MusyXSong {
    pub id: u32,
    pub tempo: u16,
    #[cfg(feature = "alloc")]
    pub patterns: Vec<MusyXPattern>,
}
