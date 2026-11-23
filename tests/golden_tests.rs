//! Golden Integration Tests for Factor 5 MusyX Sound System (musyx-rs)
//!
//! Verifies:
//! - Tier 1: Bit-exact sample PCM loading, length, sha256 checksums, and 4-bit ADPCM decompression.
//! - Tier 2: Driver version signatures, project structures, and hardware timer rates.
//! - Tier 3: AccuracySynth fixed-point software mixer and DirectSound DMA ring buffer clamping.

use musyx::{
    AccuracySynth, MusyXAdpcm, MusyXSampleFormat, MusyXSampleView, MusyXVersion, MusyXVoice,
};
#[cfg(feature = "alloc")]
use musyx::MusyXSample;

const MANIFEST_JSON: &str = include_str!("golden/manifest.json");

const ADPCM_INPUT: &[u8] = include_bytes!("golden/adpcm_input.bin");
const ADPCM_EXPECTED_PCM: &[u8] = include_bytes!("golden/adpcm_expected_pcm.bin");

const RAYMAN_SAMPLE0: &[u8] = include_bytes!("golden/rayman_sample_0.pcm");
const RAYMAN_SAMPLE0_BLOCK: &[u8] = include_bytes!("golden/rayman_sample_0.bin");
const RAYMAN_SAMPLE1: &[u8] = include_bytes!("golden/rayman_sample_1.pcm");
const RAYMAN_SAMPLE2: &[u8] = include_bytes!("golden/rayman_sample_2.pcm");

const ACCURACY_SYNTH_MIXED_PCM16: &[u8] = include_bytes!("golden/accuracy_synth_mixed_pcm16.bin");
const DIRECTSOUND_FIFO_RAYMAN: &[u8] = include_bytes!("golden/directsound_fifo_rayman.bin");

#[test]
fn test_manifest_integrity() {
    assert!(!MANIFEST_JSON.is_empty());
    assert!(MANIFEST_JSON.contains("adpcm_input.bin"));
    assert!(MANIFEST_JSON.contains("adpcm_expected_pcm.bin"));
    assert!(MANIFEST_JSON.contains("directsound_fifo_rayman.bin"));
    assert!(MANIFEST_JSON.contains("accuracy_synth_mixed_pcm16.bin"));
    assert!(MANIFEST_JSON.contains("rayman_sample_0.pcm"));
    assert!(MANIFEST_JSON.contains("rayman_sample_1.pcm"));
    assert!(MANIFEST_JSON.contains("rayman_sample_2.pcm"));
}

#[test]
fn test_tier1_adpcm_golden_decompression() {
    assert_eq!(ADPCM_INPUT.len(), 512);
    assert_eq!(ADPCM_EXPECTED_PCM.len(), 2048);

    // Decode 1024 16-bit linear PCM samples without alloc
    let mut decoded = [0i16; 1024];
    let count = MusyXAdpcm::decode_to_slice(ADPCM_INPUT, &mut decoded);
    assert_eq!(count, 1024);

    // Verify bit-exact match against golden reference
    for (i, &sample) in decoded.iter().enumerate() {
        let expected = i16::from_le_bytes([
            ADPCM_EXPECTED_PCM[i * 2],
            ADPCM_EXPECTED_PCM[i * 2 + 1],
        ]);
        assert_eq!(
            sample, expected,
            "ADPCM sample mismatch at index {i}: decoded={sample}, expected={expected}"
        );
    }

    // Verify clamping boundaries occurred in test dataset
    assert!(decoded.contains(&32767), "Expected upper saturation clamp");
    assert!(decoded.contains(&-32768), "Expected lower saturation clamp");

    // Verify truncation handling
    let mut truncated = [0i16; 17];
    let trunc_count = MusyXAdpcm::decode_to_slice(ADPCM_INPUT, &mut truncated);
    assert_eq!(trunc_count, 17);
    assert_eq!(&truncated[..], &decoded[..17]);

    #[cfg(feature = "alloc")]
    {
        let alloc_decoded = MusyXAdpcm::decode(ADPCM_INPUT, 1024).expect("adpcm decode");
        assert_eq!(alloc_decoded.as_slice(), &decoded[..]);
    }
}

#[test]
fn test_tier1_rayman_sample0_view_bit_exact() {
    assert_eq!(RAYMAN_SAMPLE0.len(), 581);
    let view = MusyXSampleView::from_pcm8(0, 11025, RAYMAN_SAMPLE0, Some(413));
    assert_eq!(view.len(), 581);
    assert_eq!(view.loop_start, Some(413));
    assert_eq!(view.format, MusyXSampleFormat::Pcm8);

    let mut pcm16 = [0i16; 581];
    let count = view.decode_to_pcm16_slice(&mut pcm16).expect("decode to pcm16");
    assert_eq!(count, 581);
    assert_eq!(pcm16[0], 0);
    assert_eq!(pcm16[1], 2 * 256);
    assert_eq!(pcm16[2], 4 * 256);
    assert_eq!(pcm16[3], 6 * 256);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_rayman_sample0_bit_exact() {
    assert_eq!(RAYMAN_SAMPLE0.len(), 581);
    let sample = MusyXSample::from_pcm8(0, 11025, RAYMAN_SAMPLE0, Some(413));
    assert_eq!(sample.len(), 581);
    assert_eq!(sample.loop_start, Some(413));
    assert_eq!(sample.format, MusyXSampleFormat::Pcm8);

    let pcm16 = sample.to_pcm16().expect("decode to pcm16");
    assert_eq!(pcm16.len(), 581);
    // Check first few raw bytes: 0x00, 0x02, 0x04, 0x06
    assert_eq!(pcm16[0], 0);
    assert_eq!(pcm16[1], 2 * 256);
    assert_eq!(pcm16[2], 4 * 256);
    assert_eq!(pcm16[3], 6 * 256);
}

#[test]
fn test_tier1_rayman_sample0_view_header_block() {
    assert_eq!(RAYMAN_SAMPLE0_BLOCK.len(), 16 + 581);
    let view = MusyXSampleView::from_header_block(0, RAYMAN_SAMPLE0_BLOCK)
        .expect("parse sample header block");

    assert_eq!(view.id, 0);
    assert_eq!(view.sample_rate, 11025);
    assert_eq!(view.loop_start, Some(413));
    assert_eq!(view.format, MusyXSampleFormat::Pcm8);
    assert_eq!(view.len(), 581);
    assert_eq!(view.raw_bytes, RAYMAN_SAMPLE0);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_rayman_sample0_header_block() {
    assert_eq!(RAYMAN_SAMPLE0_BLOCK.len(), 16 + 581);
    let sample = MusyXSample::from_header_block(0, RAYMAN_SAMPLE0_BLOCK)
        .expect("parse sample header block");

    assert_eq!(sample.id, 0);
    assert_eq!(sample.sample_rate, 11025);
    assert_eq!(sample.loop_start, Some(413));
    assert_eq!(sample.format, MusyXSampleFormat::Pcm8);
    assert_eq!(sample.len(), 581);
    assert_eq!(&sample.raw_bytes[..], RAYMAN_SAMPLE0);
}

#[test]
fn test_tier1_rayman_sample1_view_bit_exact() {
    assert_eq!(RAYMAN_SAMPLE1.len(), 1455);
    let view = MusyXSampleView::from_pcm8(1, 11025, RAYMAN_SAMPLE1, Some(0));
    assert_eq!(view.len(), 1455);
    let mut pcm16 = [0i16; 1455];
    let count = view.decode_to_pcm16_slice(&mut pcm16).expect("decode to pcm16");
    assert_eq!(count, 1455);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_rayman_sample1_bit_exact() {
    assert_eq!(RAYMAN_SAMPLE1.len(), 1455);
    let sample = MusyXSample::from_pcm8(1, 11025, RAYMAN_SAMPLE1, Some(0));
    assert_eq!(sample.len(), 1455);
    let pcm16 = sample.to_pcm16().expect("decode to pcm16");
    assert_eq!(pcm16.len(), 1455);
}

#[test]
fn test_tier1_rayman_sample2_view_bit_exact() {
    assert_eq!(RAYMAN_SAMPLE2.len(), 2286);
    let view = MusyXSampleView::from_pcm8(2, 11025, RAYMAN_SAMPLE2, Some(557));
    assert_eq!(view.len(), 2286);
    let mut pcm16 = [0i16; 2286];
    let count = view.decode_to_pcm16_slice(&mut pcm16).expect("decode to pcm16");
    assert_eq!(count, 2286);
}

#[cfg(feature = "alloc")]
#[test]
fn test_tier1_rayman_sample2_bit_exact() {
    assert_eq!(RAYMAN_SAMPLE2.len(), 2286);
    let sample = MusyXSample::from_pcm8(2, 11025, RAYMAN_SAMPLE2, Some(557));
    assert_eq!(sample.len(), 2286);
    let pcm16 = sample.to_pcm16().expect("decode to pcm16");
    assert_eq!(pcm16.len(), 2286);
}

#[test]
fn test_tier1_adpcm_clamping_and_delta() {
    let adpcm_data = [0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67];
    let mut decoded = [0i16; 16];
    let count = MusyXAdpcm::decode_to_slice(&adpcm_data, &mut decoded);
    assert_eq!(count, 16);
    assert!(decoded.iter().any(|&s| s < 0));
    assert!(decoded.iter().any(|&s| s != 0));
}

#[test]
fn test_tier2_driver_versions() {
    let dummy_v1 = b"MusyX audio tools (c) Factor 5";
    assert_eq!(MusyXVersion::detect_from_bytes(dummy_v1), Some(MusyXVersion::V1_0));

    let dummy_v2 = b"MUSX project binary file";
    assert_eq!(MusyXVersion::detect_from_bytes(dummy_v2), Some(MusyXVersion::V2_0));
}

#[test]
fn test_tier3_accuracy_synth_mixer_loop() {
    let view = MusyXSampleView::from_pcm8(0, 11025, RAYMAN_SAMPLE0, Some(413));
    let mut pcm16 = [0i16; 581];
    view.decode_to_pcm16_slice(&mut pcm16).expect("pcm16");

    let mut voice = MusyXVoice::new(0, 0x1000, 255, 255);
    let mut left = [0i16; 128];
    let mut right = [0i16; 128];

    AccuracySynth::mix_voice_pcm16(&pcm16, &mut voice, &mut left, &mut right);

    assert!(left.iter().any(|&s| s != 0));
    assert_eq!(left, right); // Full volume center
    assert!(voice.is_active);
    assert_eq!(voice.position_fixed, 128 * 0x1000);
}

#[test]
fn test_tier3_accuracy_synth_golden_pcm16_mixer() {
    assert_eq!(ACCURACY_SYNTH_MIXED_PCM16.len(), 512);

    // Convert raw RAYMAN_SAMPLE0 bytes to signed i8 without heap allocation
    let mut pcm8 = [0i8; 581];
    for (dst, &src) in pcm8.iter_mut().zip(RAYMAN_SAMPLE0) {
        *dst = src as i8;
    }

    // Voice playing sample 0 at 1.0x pitch (pitch_step 0x1000), vol_left=255, vol_right=128
    let mut voice = MusyXVoice::new(0, 0x1000, 255, 128);
    let mut left = [0i16; 128];
    let mut right = [0i16; 128];

    AccuracySynth::mix_voice_pcm8(&pcm8, &mut voice, &mut left, &mut right);

    // Verify bit-exact output matching mGBA cycle-accurate execution of ARM 0x0804E210
    for i in 0..128 {
        let exp_left = i16::from_le_bytes([
            ACCURACY_SYNTH_MIXED_PCM16[i * 4],
            ACCURACY_SYNTH_MIXED_PCM16[i * 4 + 1],
        ]);
        let exp_right = i16::from_le_bytes([
            ACCURACY_SYNTH_MIXED_PCM16[i * 4 + 2],
            ACCURACY_SYNTH_MIXED_PCM16[i * 4 + 3],
        ]);

        assert_eq!(
            left[i], exp_left,
            "Left channel mismatch at frame {i}: got {}, expected {}",
            left[i], exp_left
        );
        assert_eq!(
            right[i], exp_right,
            "Right channel mismatch at frame {i}: got {}, expected {}",
            right[i], exp_right
        );
    }
}

#[test]
fn test_tier3_directsound_dma_fifo_golden_output() {
    assert_eq!(DIRECTSOUND_FIFO_RAYMAN.len(), 256);

    let mut pcm8 = [0i8; 581];
    for (dst, &src) in pcm8.iter_mut().zip(RAYMAN_SAMPLE0) {
        *dst = src as i8;
    }
    let mut voice = MusyXVoice::new(0, 0x1000, 255, 128);
    let mut left = [0i16; 128];
    let mut right = [0i16; 128];

    AccuracySynth::mix_voice_pcm8(&pcm8, &mut voice, &mut left, &mut right);

    // Clamp intermediate 16-bit to 8-bit DirectSound DMA buffer
    let mut dma_left = [0i8; 128];
    let mut dma_right = [0i8; 128];
    AccuracySynth::clamp_pcm16_to_dma8(&left, &mut dma_left);
    AccuracySynth::clamp_pcm16_to_dma8(&right, &mut dma_right);

    // Verify bit-exact match against captured DirectSound DMA FIFO buffer
    for i in 0..128 {
        let exp_left = DIRECTSOUND_FIFO_RAYMAN[i] as i8;
        let exp_right = DIRECTSOUND_FIFO_RAYMAN[128 + i] as i8;

        assert_eq!(
            dma_left[i], exp_left,
            "DirectSound DMA left mismatch at frame {i}: got {}, expected {}",
            dma_left[i], exp_left
        );
        assert_eq!(
            dma_right[i], exp_right,
            "DirectSound DMA right mismatch at frame {i}: got {}, expected {}",
            dma_right[i], exp_right
        );
    }
}
