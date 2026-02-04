use muxide::api::{AacProfile, AudioCodec, MuxerBuilder};

#[test]
fn audio_only_fragmented_builds() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::<u8>::new();
    let _muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .new_with_fragment()?;

    // Successfully building is the test
    Ok(())
}

#[test]
fn audio_only_fragmented_init_segment() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::<u8>::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .new_with_fragment()?;

    let init = muxer.init_segment();

    // Check ftyp
    assert_eq!(&init[4..8], b"ftyp");
    
    // Find moov
    let ftyp_size = u32::from_be_bytes(init[0..4].try_into().unwrap()) as usize;
    assert_eq!(&init[ftyp_size + 4..ftyp_size + 8], b"moov");

    Ok(())
}

#[test]
fn audio_only_fragmented_write_audio() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::<u8>::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Aac(AacProfile::Lc), 48000, 2)
        .new_with_fragment()?;

    // Get init segment
    let _init = muxer.init_segment();

    // Write audio samples (in timescale units, 90000Hz)
    let audio_data = vec![0xaa, 0xbb, 0xcc, 0xdd]; // Dummy audio data
    muxer.write_audio(0, &audio_data)?;
    muxer.write_audio(1920, &audio_data)?; // ~21ms later at 90kHz

    // Flush segment
    let segment = muxer.flush_segment();
    assert!(segment.is_some());

    let seg = segment.unwrap();
    // Check moof
    assert_eq!(&seg[4..8], b"moof");

    // Find mdat
    let moof_size = u32::from_be_bytes(seg[0..4].try_into().unwrap()) as usize;
    assert_eq!(&seg[moof_size + 4..moof_size + 8], b"mdat");

    Ok(())
}

#[test]
fn audio_only_fragmented_opus() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::<u8>::new();
    let mut muxer = MuxerBuilder::new(&mut buffer)
        .audio(AudioCodec::Opus, 48000, 2)
        .new_with_fragment()?;

    let init = muxer.init_segment();
    assert!(!init.is_empty());

    Ok(())
}

#[test]
fn no_tracks_configured_error() {
    let mut buffer = Vec::<u8>::new();
    let result = MuxerBuilder::new(&mut buffer).new_with_fragment();

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        muxide::api::MuxerError::MissingTrackConfig
    ));
}
