/// Audio-only MP4 muxing demonstration
///
/// This example creates an audio-only MP4 file with AAC audio.
/// Run with: cargo run --example audio_only_demo
///
/// The output file can be tested in:
/// - QuickTime Player (macOS)
/// - VLC Media Player
/// - Chrome/Firefox web browsers
/// - ffprobe (to inspect structure)
///
/// Expected output: A valid MP4 file with only an audio track

use muxide::api::{AacProfile, AudioCodec, Metadata, MuxerBuilder};
use std::fs::File;

/// Helper to create a minimal valid AAC ADTS frame
fn make_aac_frame() -> Vec<u8> {
    // ADTS header for AAC-LC, 48kHz, stereo
    // This is a minimal valid ADTS frame with a small payload
    vec![
        0xff, 0xf1, // Syncword (12 bits) + MPEG-4 + no CRC
        0x50, // Profile (LC) + sample rate index (48kHz=3)
        0x80, // Channel config (stereo=2) + frame length high bits
        0x01, // Frame length low bits
        0x3f, 0xfc, // Buffer fullness + number of frames
        // Payload (dummy data - in real usage, this would be actual AAC audio data)
        0x21, 0x10, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating audio-only MP4 file...");

    let file = File::create("audio_only_demo.mp4")?;

    // Create an audio-only muxer (no video track)
    let mut muxer = MuxerBuilder::new(file)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .with_metadata(
            Metadata::new()
                .with_title("Audio-Only Demo")
                .with_language("eng")
                .with_current_time(),
        )
        .with_fast_start(true) // moov before mdat for web playback
        .build()?;

    // Write 100 audio frames (~2.1 seconds @ 48kHz with 1024 samples per frame)
    let audio_frame = make_aac_frame();
    let frame_duration = 1024.0 / 48000.0; // ~21.3ms per frame

    for i in 0..100 {
        let pts = i as f64 * frame_duration;
        muxer.write_audio(pts, &audio_frame)?;

        if (i + 1) % 25 == 0 {
            println!("  Written {} frames...", i + 1);
        }
    }

    // Finalize and get statistics
    let stats = muxer.finish_with_stats()?;

    println!("\nSuccess! Created audio_only_demo.mp4");
    println!("  Audio frames: {}", stats.audio_frames);
    println!("  Video frames: {} (audio-only)", stats.video_frames);
    println!("  Duration: {:.2}s", stats.duration_secs);
    println!("  File size: {} bytes", stats.bytes_written);

    println!("\nValidation:");
    println!("  1. Play in QuickTime: open audio_only_demo.mp4");
    println!("  2. Play in VLC: vlc audio_only_demo.mp4");
    println!("  3. Inspect: ffprobe audio_only_demo.mp4");
    println!("  4. Web: Open in Chrome/Firefox (drag and drop)");

    Ok(())
}
