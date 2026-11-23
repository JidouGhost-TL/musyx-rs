use std::time::Instant;
use musyx::{AccuracySynth, MusyXAdpcm, MusyXVoice};

fn main() {
    println!("=== musyx-rs Audio Performance Benchmarks ===");

    // 1. ADPCM decode_to_slice
    let adpcm_input = include_bytes!("../tests/golden/adpcm_input.bin");
    let mut out_pcm = vec![0i16; 1024];

    // Warmup
    for _ in 0..1000 {
        MusyXAdpcm::decode_to_slice(adpcm_input, &mut out_pcm);
    }

    let iters = 100_000;
    let start = Instant::now();
    let mut total_samples = 0;
    for _ in 0..iters {
        total_samples += MusyXAdpcm::decode_to_slice(adpcm_input, &mut out_pcm);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = (total_samples as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "MusyXAdpcm::decode_to_slice (1024 samples): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );

    // 2. AccuracySynth::mix_voice_pcm16 (1:1 pitch)
    let pcm16_data = vec![1234i16; 2048];
    let mut out_l = vec![0i16; 1024];
    let mut out_r = vec![0i16; 1024];
    let mut voice_1to1 = MusyXVoice::new(0, 0x1000, 255, 128);

    for _ in 0..1000 {
        voice_1to1.position_fixed = 0;
        voice_1to1.is_active = true;
        AccuracySynth::mix_voice_pcm16(&pcm16_data, &mut voice_1to1, &mut out_l, &mut out_r);
    }

    let start = Instant::now();
    for _ in 0..iters {
        voice_1to1.position_fixed = 0;
        voice_1to1.is_active = true;
        AccuracySynth::mix_voice_pcm16(&pcm16_data, &mut voice_1to1, &mut out_l, &mut out_r);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = ((iters * 1024) as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "AccuracySynth::mix_voice_pcm16 (1:1 pitch, 1024 frames): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );

    // 3. AccuracySynth::mix_voice_pcm16 (resampled pitch 0x1500)
    let mut voice_resample = MusyXVoice::new(0, 0x1500, 255, 128);
    let start = Instant::now();
    for _ in 0..iters {
        voice_resample.position_fixed = 0;
        voice_resample.is_active = true;
        AccuracySynth::mix_voice_pcm16(&pcm16_data, &mut voice_resample, &mut out_l, &mut out_r);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = ((iters * 1024) as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "AccuracySynth::mix_voice_pcm16 (resampled 0x1500, 1024 frames): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );

    // 4. AccuracySynth::mix_voice_pcm8 (1:1 pitch)
    let pcm8_data = vec![64i8; 2048];
    for _ in 0..1000 {
        voice_1to1.position_fixed = 0;
        voice_1to1.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_1to1, &mut out_l, &mut out_r);
    }
    let start = Instant::now();
    for _ in 0..iters {
        voice_1to1.position_fixed = 0;
        voice_1to1.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_1to1, &mut out_l, &mut out_r);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = ((iters * 1024) as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "AccuracySynth::mix_voice_pcm8 (1:1 pitch, 1024 frames): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );

    // 5. AccuracySynth::mix_voice_pcm8 (resampled pitch 0x1500)
    let start = Instant::now();
    for _ in 0..iters {
        voice_resample.position_fixed = 0;
        voice_resample.is_active = true;
        AccuracySynth::mix_voice_pcm8(&pcm8_data, &mut voice_resample, &mut out_l, &mut out_r);
    }
    let elapsed = start.elapsed();
    let ns_per_iter = elapsed.as_nanos() as f64 / iters as f64;
    let msamples_per_sec = ((iters * 1024) as f64 / elapsed.as_secs_f64()) / 1_000_000.0;
    println!(
        "AccuracySynth::mix_voice_pcm8 (resampled 0x1500, 1024 frames): {:.2} ns/iter ({:.2} MSamples/sec)",
        ns_per_iter, msamples_per_sec
    );
}
