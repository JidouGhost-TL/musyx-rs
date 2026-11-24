//! Software synthesis engines for MusyX.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A single active playback voice in the MusyX software synthesizer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MusyXVoice {
    pub sample_id: u32,
    /// Fixed-point position in sample: integer part in high bits, 12-bit fractional phase in low bits.
    pub position_fixed: u32,
    /// Phase increment per output sample (12-bit fixed point fractional step).
    pub pitch_step: u32,
    /// Panning and volume (0..255).
    pub vol_left: u8,
    pub vol_right: u8,
    pub is_active: bool,
}

impl MusyXVoice {
    pub fn new(sample_id: u32, pitch_step: u32, vol_left: u8, vol_right: u8) -> Self {
        Self {
            sample_id,
            position_fixed: 0,
            pitch_step,
            vol_left,
            vol_right,
            is_active: true,
        }
    }
}

/// Bit-exact software mixer matching the ARM driver's DMA ring buffer loop.
///
/// Implements the exact fixed-point linear interpolation algorithm reverse-engineered
/// from the MusyX driver assembly at `0x0804E300`:
/// ```text
///   delta = next_sample - curr_sample
///   frac = pos & 0x0FFF
///   interpolated = curr_sample + ((delta * frac) >> 12)
///   left = (interpolated * vol_left) >> 8
///   right = (interpolated * vol_right) >> 8
/// ```
#[derive(Debug, Clone)]
pub struct AccuracySynth {
    pub sample_rate: u32,
    pub channels: u8,
    #[cfg(feature = "alloc")]
    pub voices: Vec<MusyXVoice>,
}

impl Default for AccuracySynth {
    fn default() -> Self {
        Self::new(13379)
    }
}

impl AccuracySynth {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 32,
            #[cfg(feature = "alloc")]
            voices: Vec::new(),
        }
    }

    /// Add an active voice to the synthesizer.
    #[cfg(feature = "alloc")]
    pub fn add_voice(&mut self, voice: MusyXVoice) {
        self.voices.push(voice);
    }

    /// Bit-exact linear interpolation mixer step.
    #[inline(always)]
    pub fn interpolate_sample(curr: i16, next: i16, frac: u16) -> i16 {
        let delta = (next as i32) - (curr as i32);
        let frac_part = (frac & 0x0FFF) as i32;
        (curr as i32 + ((delta * frac_part) >> 12)) as i16
    }

    /// Mix a single voice into stereo 16-bit PCM buffer.
    pub fn mix_voice_pcm16(
        pcm_data: &[i16],
        voice: &mut MusyXVoice,
        out_left: &mut [i16],
        out_right: &mut [i16],
    ) {
        if !voice.is_active || pcm_data.is_empty() {
            return;
        }

        let len = out_left.len().min(out_right.len());
        if len == 0 {
            return;
        }
        let pcm_len = pcm_data.len();
        let out_left = &mut out_left[..len];
        let out_right = &mut out_right[..len];

        let vol_l = voice.vol_left as i32;
        let vol_r = voice.vol_right as i32;

        // 1:1 pitch step fast path
        if voice.pitch_step == 0x1000 && (voice.position_fixed & 0x0FFF) == 0 {
            let start_pos = (voice.position_fixed >> 12) as usize;
            if start_pos >= pcm_len {
                voice.is_active = false;
                return;
            }
            let count = len.min(pcm_len - start_pos);
            let pcm_sub = &pcm_data[start_pos..start_pos + count];
            let out_l_sub = &mut out_left[..count];
            let out_r_sub = &mut out_right[..count];

            for ((out_l, out_r), &curr) in out_l_sub.iter_mut().zip(out_r_sub.iter_mut()).zip(pcm_sub) {
                let l_sample = ((curr as i32 * vol_l) >> 8) as i16;
                let r_sample = ((curr as i32 * vol_r) >> 8) as i16;

                *out_l = out_l.saturating_add(l_sample);
                *out_r = out_r.saturating_add(r_sample);
            }

            voice.position_fixed = voice.position_fixed.wrapping_add((count as u32).wrapping_shl(12));
            if count < len {
                voice.is_active = false;
            }
            return;
        }

        // Resampling path: eliminate bounds checks on output slices via zip
        for (out_l, out_r) in out_left.iter_mut().zip(out_right.iter_mut()) {
            let int_pos = (voice.position_fixed >> 12) as usize;

            if int_pos >= pcm_len {
                voice.is_active = false;
                break;
            }

            let curr = pcm_data[int_pos];
            let next = if int_pos + 1 < pcm_len {
                pcm_data[int_pos + 1]
            } else {
                curr
            };

            let frac = (voice.position_fixed & 0x0FFF) as i32;
            let delta = (next as i32) - (curr as i32);
            let sample = (curr as i32 + ((delta * frac) >> 12)) as i16;

            let l_sample = ((sample as i32 * vol_l) >> 8) as i16;
            let r_sample = ((sample as i32 * vol_r) >> 8) as i16;

            *out_l = out_l.saturating_add(l_sample);
            *out_r = out_r.saturating_add(r_sample);

            voice.position_fixed = voice.position_fixed.wrapping_add(voice.pitch_step);
        }
    }

    /// Mix an 8-bit PCM voice into stereo 16-bit intermediate buffer, matching ARM driver `0x0804E210`.
    pub fn mix_voice_pcm8(
        pcm8_data: &[i8],
        voice: &mut MusyXVoice,
        out_left: &mut [i16],
        out_right: &mut [i16],
    ) {
        if !voice.is_active || pcm8_data.is_empty() {
            return;
        }

        let len = out_left.len().min(out_right.len());
        if len == 0 {
            return;
        }
        let pcm_len = pcm8_data.len();
        let out_left = &mut out_left[..len];
        let out_right = &mut out_right[..len];

        let vol_l = if voice.vol_left > 0 {
            voice.vol_left as i32 + 1
        } else {
            0
        };
        let vol_r = if voice.vol_right > 0 {
            voice.vol_right as i32 + 1
        } else {
            0
        };

        // 1:1 pitch step fast path
        if voice.pitch_step == 0x1000 && (voice.position_fixed & 0x0FFF) == 0 {
            let start_pos = (voice.position_fixed >> 12) as usize;
            if start_pos >= pcm_len {
                voice.is_active = false;
                return;
            }
            let count = len.min(pcm_len - start_pos);
            let pcm_sub = &pcm8_data[start_pos..start_pos + count];
            let out_l_sub = &mut out_left[..count];
            let out_r_sub = &mut out_right[..count];

            for ((out_l, out_r), &curr) in out_l_sub.iter_mut().zip(out_r_sub.iter_mut()).zip(pcm_sub) {
                let s = curr as i32;
                let l_sample = ((s * vol_l) >> 8) as i16;
                let r_sample = ((s * vol_r) >> 8) as i16;

                *out_l = out_l.saturating_add(l_sample);
                *out_r = out_r.saturating_add(r_sample);
            }

            voice.position_fixed = voice.position_fixed.wrapping_add((count as u32).wrapping_shl(12));
            if count < len {
                voice.is_active = false;
            }
            return;
        }

        // Resampling path: eliminate bounds checks on output slices via zip
        for (out_l, out_r) in out_left.iter_mut().zip(out_right.iter_mut()) {
            let int_pos = (voice.position_fixed >> 12) as usize;

            if int_pos >= pcm_len {
                voice.is_active = false;
                break;
            }

            let curr = pcm8_data[int_pos] as i32;
            let next = if int_pos + 1 < pcm_len {
                pcm8_data[int_pos + 1] as i32
            } else {
                curr
            };

            let frac = (voice.position_fixed & 0x0FFF) as i32;
            let delta = next - curr;
            let interp = curr + ((delta * frac) >> 12);

            let l_sample = ((interp * vol_l) >> 8) as i16;
            let r_sample = ((interp * vol_r) >> 8) as i16;

            *out_l = out_l.saturating_add(l_sample);
            *out_r = out_r.saturating_add(r_sample);

            voice.position_fixed = voice.position_fixed.wrapping_add(voice.pitch_step);
        }
    }

    /// Clamp 16-bit intermediate buffer to 8-bit DirectSound DMA FIFO buffer, matching ARM driver `0x0804E064`.
    pub fn clamp_pcm16_to_dma8(input: &[i16], out: &mut [i8]) {
        let len = input.len().min(out.len());
        for (&inp, outp) in input[..len].iter().zip(&mut out[..len]) {
            *outp = inp.clamp(-128, 127) as i8;
        }
    }

    /// Mix multiple voices matching a sample into stereo 16-bit intermediate buffers without allocation.
    pub fn mix_voices_pcm16(
        voices: &mut [MusyXVoice],
        pcm_data: &[i16],
        sample_id: u32,
        out_left: &mut [i16],
        out_right: &mut [i16],
    ) {
        for voice in voices.iter_mut() {
            if !voice.is_active || voice.sample_id != sample_id {
                continue;
            }
            Self::mix_voice_pcm16(pcm_data, voice, out_left, out_right);
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StudioSynth {
    pub sample_rate: u32,
    pub sinc_quality: u8,
}

impl StudioSynth {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            sinc_quality: 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_zero_frac() {
        let curr = 1000;
        let next = 2000;
        assert_eq!(AccuracySynth::interpolate_sample(curr, next, 0), 1000);
    }

    #[test]
    fn test_interpolate_half_frac() {
        let curr = 1000;
        let next = 2000;
        // 0x800 is half of 0x1000 (4096)
        assert_eq!(AccuracySynth::interpolate_sample(curr, next, 0x800), 1500);
    }

    #[test]
    fn test_mix_voice_stereo() {
        let pcm = [1024i16, 2048i16, 3072i16];
        let mut voice = MusyXVoice::new(0, 0x1000, 255, 128); // 1.0 step, max left, half right
        let mut left = [0i16; 3];
        let mut right = [0i16; 3];

        AccuracySynth::mix_voice_pcm16(&pcm, &mut voice, &mut left, &mut right);

        // 1024 * 255 >> 8 = 1020
        assert_eq!(left[0], 1020);
        // 1024 * 128 >> 8 = 512
        assert_eq!(right[0], 512);
        // 2048 * 255 >> 8 = 2040
        assert_eq!(left[1], 2040);
        // 2048 * 128 >> 8 = 1024
        assert_eq!(right[1], 1024);
    }
}
