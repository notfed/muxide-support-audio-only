use muxide::api::{AacProfile, AudioCodec, MuxerBuilder};

/// Helper to create a valid AAC ADTS frame with LC profile
fn make_aac_lc_frame() -> Vec<u8> {
    // ADTS header for AAC-LC, 48kHz, stereo, frame length 9
    vec![
        0xff, 0xf1, // Sync word + MPEG-4, no CRC
        0x50, // Profile (LC=1), sample rate index (48kHz=3), channel config starts
        0x80, // Channel config (stereo=2), frame length high bits
        0x01, // Frame length low bits
        0x3f, 0xfc, // Buffer fullness, frame count
        0xaa, 0xbb, // Payload (dummy data)
    ]
}

/// Helper to create valid Opus packets
fn make_opus_packet() -> Vec<u8> {
    // Simple Opus packet with TOC byte (config 16, stereo, 20ms)
    vec![0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

#[test]
fn audio_only_aac_build() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let _muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .build()?;

    // Successfully building is the test
    Ok(())
}

#[test]
fn audio_only_opus_build() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let _muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Opus, 48000, 2)
        .build()?;

    // Successfully building is the test
    Ok(())
}

#[test]
fn missing_all_tracks_error() {
    let mut buffer = Vec::new();
    let result = MuxerBuilder::new(&mut buffer).build();

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err, muxide::api::MuxerError::MissingTrackConfig));
    }
}

#[test]
fn audio_only_write_audio() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .build()?;

    // Write some audio frames
    let audio_frame = make_aac_lc_frame();
    muxer.write_audio(0.0, &audio_frame)?;
    muxer.write_audio(0.021, &audio_frame)?; // ~21ms later (1024 samples @ 48kHz)
    muxer.write_audio(0.042, &audio_frame)?;

    muxer.finish()?;

    // Verify we got an MP4 file
    assert!(!buffer.is_empty());
    // Check for ftyp box
    assert_eq!(&buffer[4..8], b"ftyp");

    Ok(())
}

#[test]
fn audio_only_aac_fast_start() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .with_fast_start(true)
        .build()?;

    let audio_frame = make_aac_lc_frame();
    for i in 0..10 {
        let pts = i as f64 * 0.021; // ~21ms per frame
        muxer.write_audio(pts, &audio_frame)?;
    }

    muxer.finish()?;

    assert!(!buffer.is_empty());
    // Verify fast-start: moov should come before mdat
    // Find moov and mdat positions
    let moov_pos = buffer.windows(4).position(|w| w == b"moov");
    let mdat_pos = buffer.windows(4).position(|w| w == b"mdat");

    assert!(moov_pos.is_some(), "moov box not found");
    assert!(mdat_pos.is_some(), "mdat box not found");
    assert!(
        moov_pos.unwrap() < mdat_pos.unwrap(),
        "fast-start: moov should come before mdat"
    );

    Ok(())
}

#[test]
fn audio_only_aac_standard_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .with_fast_start(false)
        .build()?;

    let audio_frame = make_aac_lc_frame();
    for i in 0..10 {
        let pts = i as f64 * 0.021;
        muxer.write_audio(pts, &audio_frame)?;
    }

    muxer.finish()?;

    assert!(!buffer.is_empty());
    // In standard mode, mdat comes before moov
    let moov_pos = buffer.windows(4).position(|w| w == b"moov");
    let mdat_pos = buffer.windows(4).position(|w| w == b"mdat");

    assert!(moov_pos.is_some(), "moov box not found");
    assert!(mdat_pos.is_some(), "mdat box not found");
    assert!(
        mdat_pos.unwrap() < moov_pos.unwrap(),
        "standard mode: mdat should come before moov"
    );

    Ok(())
}

#[test]
fn audio_only_opus() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Opus, 48000, 2)
        .build()?;

    let opus_packet = make_opus_packet();
    // Opus at 48kHz, typical packet is 20ms (960 samples)
    for i in 0..10 {
        let pts = i as f64 * 0.020; // 20ms per packet
        muxer.write_audio(pts, &opus_packet)?;
    }

    muxer.finish()?;

    assert!(!buffer.is_empty());
    assert_eq!(&buffer[4..8], b"ftyp");

    Ok(())
}

#[test]
fn audio_only_with_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .with_metadata(
            muxide::api::Metadata::new()
                .with_title("Audio Only Test")
                .with_language("eng"),
        )
        .build()?;

    let audio_frame = make_aac_lc_frame();
    muxer.write_audio(0.0, &audio_frame)?;
    muxer.write_audio(0.021, &audio_frame)?;

    muxer.finish()?;

    assert!(!buffer.is_empty());

    Ok(())
}

#[test]
fn audio_only_stats() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 44100, 2)
        .build()?;

    let audio_frame = make_aac_lc_frame();
    for i in 0..5 {
        let pts = i as f64 * 0.023; // ~23ms per frame (1024 samples @ 44.1kHz)
        muxer.write_audio(pts, &audio_frame)?;
    }

    let stats = muxer.finish_with_stats()?;

    assert_eq!(stats.audio_frames, 5);
    assert_eq!(stats.video_frames, 0);
    assert!(stats.duration_secs > 0.0);
    assert!(stats.bytes_written > 0);

    Ok(())
}

#[test]
fn audio_only_empty_file() -> Result<(), Box<dyn std::error::Error>> {
    // Test that we can create an audio-only MP4 with no samples
    let mut buffer = Vec::new();
    let muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .build()?;

    muxer.finish()?;

    // Should have a valid MP4 structure even with no samples
    assert!(!buffer.is_empty());
    assert_eq!(&buffer[4..8], b"ftyp");

    Ok(())
}

#[test]
fn video_write_without_video_track_fails() {
    let mut buffer = Vec::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .build()
        .unwrap();

    // Try to write video when only audio track is configured
    let dummy_video = vec![0, 0, 0, 1, 0x65, 0x88, 0x84];
    let result = muxer.write_video(0.0, &dummy_video, true);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        muxide::api::MuxerError::VideoNotConfigured
    ));
}

#[test]
fn audio_only_various_sample_rates() -> Result<(), Box<dyn std::error::Error>> {
    // Test different AAC sample rates
    for (sample_rate, samples_per_frame) in &[(44100, 1024), (48000, 1024), (32000, 1024)] {
        let mut buffer = Vec::new();
        let mut muxer = MuxerBuilder::new(&mut buffer)
            .audio(AudioCodec::Aac(AacProfile::Lc), *sample_rate, 2)
            .build()?;

        let audio_frame = make_aac_lc_frame();
        let frame_duration = *samples_per_frame as f64 / *sample_rate as f64;

        for i in 0..5 {
            let pts = i as f64 * frame_duration;
            muxer.write_audio(pts, &audio_frame)?;
        }

        muxer.finish()?;

        assert!(!buffer.is_empty(), "Failed for sample rate {}", sample_rate);
    }

    Ok(())
}
