//! Built-in PCM codecs.
//!
//! PCM (Pulse-Code Modulation) is raw uncompressed audio. "Encoding" and
//! "decoding" PCM is essentially a memory copy + format-tag attachment, but
//! making it a first-class codec lets PCM streams flow through the same
//! pipeline plumbing as Opus, AAC, etc.
//!
//! ## Variants
//!
//! - `pcm_s16le` — 16-bit signed little-endian, interleaved
//! - `pcm_s32le` — 32-bit signed little-endian, interleaved
//! - `pcm_f32le` — 32-bit float little-endian, interleaved
//! - `pcm_f64le` — 64-bit float little-endian, interleaved
//!
//! All four are lossless, stateless, and CPU-bound (a few ns per sample).

use crate::codec::{
    Codec, CodecCapabilities, CodecConfig, CodecDescriptor, DecodeRequest, DecodedFrame, Decoder,
    EncodeRequest, EncodedPacket, Encoder, MediaKind, SampleFormat,
};
use crate::error::{CodecError, CodecResult};
use crate::registry::{CodecRegistry, RegisteredCodec};
use async_trait::async_trait;
use std::sync::Arc;

// ─── Descriptors ────────────────────────────────────────────────────────────

const ALL_CAPS: CodecCapabilities = CodecCapabilities::DECODE
    .union(CodecCapabilities::ENCODE)
    .union(CodecCapabilities::LOSSLESS)
    .union(CodecCapabilities::STATELESS);

static DESC_S16LE: CodecDescriptor = CodecDescriptor {
    name: "pcm_s16le",
    long_name: "PCM signed 16-bit little-endian",
    kind: MediaKind::Audio,
    capabilities: ALL_CAPS,
};

static DESC_S32LE: CodecDescriptor = CodecDescriptor {
    name: "pcm_s32le",
    long_name: "PCM signed 32-bit little-endian",
    kind: MediaKind::Audio,
    capabilities: ALL_CAPS,
};

static DESC_F32LE: CodecDescriptor = CodecDescriptor {
    name: "pcm_f32le",
    long_name: "PCM 32-bit float little-endian",
    kind: MediaKind::Audio,
    capabilities: ALL_CAPS,
};

static DESC_F64LE: CodecDescriptor = CodecDescriptor {
    name: "pcm_f64le",
    long_name: "PCM 64-bit float little-endian",
    kind: MediaKind::Audio,
    capabilities: ALL_CAPS,
};

// ─── Single PCM codec ───────────────────────────────────────────────────────

/// A PCM encoder/decoder. One instance covers exactly one [`SampleFormat`].
#[derive(Debug)]
pub struct PcmCodec {
    descriptor: &'static CodecDescriptor,
    sample_format: SampleFormat,
    /// Sample rate — currently unused by PCM (pass-through), but reserved for
    /// future variants like resampling-aware PCM or when this struct is reused
    /// as a template by lossy codecs.
    #[allow(dead_code)]
    sample_rate: u32,
    channels: u16,
    pts_counter: i64,
}

impl PcmCodec {
    fn new(
        descriptor: &'static CodecDescriptor,
        sample_format: SampleFormat,
        config: &CodecConfig,
    ) -> CodecResult<Self> {
        let sample_rate = config.sample_rate_or(48_000);
        let channels = config.channels_or_mono();

        if channels == 0 || channels > 32 {
            return Err(CodecError::unsupported(format!(
                "channel count {channels} out of range (1..=32)"
            ))
            .with_context(descriptor.name.to_owned()));
        }

        if sample_rate < 8_000 || sample_rate > 768_000 {
            return Err(CodecError::unsupported(format!(
                "sample rate {sample_rate} out of range (8000..=768000)"
            ))
            .with_context(descriptor.name.to_owned()));
        }

        Ok(Self {
            descriptor,
            sample_format,
            sample_rate,
            channels,
            pts_counter: 0,
        })
    }

    /// Bytes per audio frame (one sample per channel).
    fn bytes_per_frame(&self) -> usize {
        self.sample_format.bytes_per_sample() * self.channels as usize
    }

    /// Validate that `len` is a whole number of audio frames.
    fn validate_frame_aligned(&self, len: usize) -> CodecResult<()> {
        let stride = self.bytes_per_frame();
        if len % stride != 0 {
            return Err(CodecError::invalid_data(format!(
                "PCM data length {len} not aligned to frame stride {stride} \
                 (channels={}, sample_size={})",
                self.channels,
                self.sample_format.bytes_per_sample()
            ))
            .with_context(self.descriptor.name.to_owned()));
        }
        Ok(())
    }

    /// Compute frame duration in samples for the given byte length.
    fn samples_in(&self, len: usize) -> i64 {
        let stride = self.bytes_per_frame();
        if stride == 0 {
            return 0;
        }
        (len / stride) as i64
    }
}

impl Codec for PcmCodec {
    fn descriptor(&self) -> &'static CodecDescriptor {
        self.descriptor
    }
}

#[async_trait]
impl Decoder for PcmCodec {
    async fn decode(&mut self, req: DecodeRequest) -> CodecResult<DecodedFrame> {
        self.validate_frame_aligned(req.data.len())?;
        let samples = self.samples_in(req.data.len());
        let pts = req.pts.unwrap_or(self.pts_counter);
        self.pts_counter = pts + samples;

        let mut frame = DecodedFrame::new(req.data, pts);
        frame.dts = pts;
        frame.is_keyframe = true; // every PCM frame is a sync point
        frame.duration = samples;
        Ok(frame)
    }

    async fn reset(&mut self) -> CodecResult<()> {
        self.pts_counter = 0;
        Ok(())
    }
}

#[async_trait]
impl Encoder for PcmCodec {
    async fn encode(&mut self, req: EncodeRequest) -> CodecResult<EncodedPacket> {
        self.validate_frame_aligned(req.frame.payload.len())?;

        let mut pkt = EncodedPacket::new(req.frame.payload, req.frame.pts);
        pkt.dts = req.frame.dts;
        pkt.is_keyframe = true;
        pkt.duration = if req.frame.duration > 0 {
            req.frame.duration
        } else {
            self.samples_in(pkt.payload.len())
        };
        Ok(pkt)
    }

    async fn reset(&mut self) -> CodecResult<()> {
        self.pts_counter = 0;
        Ok(())
    }
}

// ─── Registration ───────────────────────────────────────────────────────────

/// Register all four PCM variants on the given registry.
pub fn register_all(reg: &CodecRegistry) {
    register_one(reg, &DESC_S16LE, SampleFormat::S16);
    register_one(reg, &DESC_S32LE, SampleFormat::S32);
    register_one(reg, &DESC_F32LE, SampleFormat::F32);
    register_one(reg, &DESC_F64LE, SampleFormat::F64);
}

fn register_one(
    reg: &CodecRegistry,
    descriptor: &'static CodecDescriptor,
    sample_format: SampleFormat,
) {
    let dec_desc = descriptor;
    let enc_desc = descriptor;
    let dec_fmt = sample_format;
    let enc_fmt = sample_format;

    reg.register(RegisteredCodec {
        descriptor,
        decoder_factory: Some(Arc::new(move |cfg| {
            let codec = PcmCodec::new(dec_desc, dec_fmt, cfg)?;
            Ok(Box::new(codec) as Box<dyn Decoder>)
        })),
        encoder_factory: Some(Arc::new(move |cfg| {
            let codec = PcmCodec::new(enc_desc, enc_fmt, cfg)?;
            Ok(Box::new(codec) as Box<dyn Encoder>)
        })),
    });
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::global_registry;
    use bytes::Bytes;

    fn cfg_stereo_48k() -> CodecConfig {
        CodecConfig {
            sample_rate: Some(48_000),
            channels: Some(2),
            ..CodecConfig::empty()
        }
    }

    #[tokio::test]
    async fn decode_s16le_roundtrip() {
        let reg = global_registry();
        let mut dec = reg.create_decoder("pcm_s16le", cfg_stereo_48k()).unwrap();

        // 4 bytes = 1 stereo frame at S16 (2 bytes/sample × 2 ch)
        let data = Bytes::from_static(&[0x00, 0x10, 0x00, 0x20]);
        let frame = dec.decode(DecodeRequest::new(data)).await.unwrap();

        assert_eq!(frame.payload.len(), 4);
        assert_eq!(frame.duration, 1);
        assert!(frame.is_keyframe);
    }

    #[tokio::test]
    async fn decode_unaligned_data_errors() {
        let reg = global_registry();
        let mut dec = reg.create_decoder("pcm_s16le", cfg_stereo_48k()).unwrap();

        // 3 bytes — not aligned to 4 (stereo S16)
        let data = Bytes::from_static(&[0, 0, 0]);
        let err = dec.decode(DecodeRequest::new(data)).await.unwrap_err();
        assert_eq!(err.kind(), crate::error::CodecErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn encode_decode_roundtrip_is_identity() {
        let reg = global_registry();
        let mut enc = reg.create_encoder("pcm_f32le", cfg_stereo_48k()).unwrap();
        let mut dec = reg.create_decoder("pcm_f32le", cfg_stereo_48k()).unwrap();

        // 4 stereo f32 frames = 32 bytes
        let raw: Vec<u8> = (0..32).collect();
        let original = Bytes::copy_from_slice(&raw);

        let frame_in = DecodedFrame::new(original.clone(), 100);
        let pkt = enc.encode(EncodeRequest::new(frame_in)).await.unwrap();
        assert_eq!(pkt.pts, 100);

        let frame_out = dec
            .decode(DecodeRequest::with_pts(pkt.payload.clone(), pkt.pts))
            .await
            .unwrap();
        assert_eq!(frame_out.payload, original);
        assert_eq!(frame_out.pts, 100);
    }

    #[tokio::test]
    async fn pts_counter_increments() {
        let reg = global_registry();
        let mut dec = reg.create_decoder("pcm_s16le", cfg_stereo_48k()).unwrap();

        // Each call: 8 bytes = 2 stereo frames
        let data = Bytes::from(vec![0u8; 8]);

        let f1 = dec.decode(DecodeRequest::new(data.clone())).await.unwrap();
        assert_eq!(f1.pts, 0);
        assert_eq!(f1.duration, 2);

        let f2 = dec.decode(DecodeRequest::new(data.clone())).await.unwrap();
        assert_eq!(f2.pts, 2);

        let f3 = dec.decode(DecodeRequest::new(data)).await.unwrap();
        assert_eq!(f3.pts, 4);
    }

    #[tokio::test]
    async fn reset_clears_pts() {
        let reg = global_registry();
        let mut dec = reg.create_decoder("pcm_s16le", cfg_stereo_48k()).unwrap();

        let data = Bytes::from(vec![0u8; 8]);
        dec.decode(DecodeRequest::new(data.clone())).await.unwrap();
        dec.reset().await.unwrap();

        let f = dec.decode(DecodeRequest::new(data)).await.unwrap();
        assert_eq!(f.pts, 0);
    }

    #[test]
    fn rejects_out_of_range_channels() {
        let reg = global_registry();
        let bad = CodecConfig {
            sample_rate: Some(48_000),
            channels: Some(0),
            ..CodecConfig::empty()
        };
        let err = match reg.create_decoder("pcm_s16le", bad) {
            Ok(_) => panic!("expected error for channels=0"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), crate::error::CodecErrorKind::Unsupported);
    }

    #[test]
    fn rejects_out_of_range_sample_rate() {
        let reg = global_registry();
        let bad = CodecConfig {
            sample_rate: Some(99),
            channels: Some(2),
            ..CodecConfig::empty()
        };
        let err = match reg.create_decoder("pcm_s16le", bad) {
            Ok(_) => panic!("expected error for sample_rate=99"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), crate::error::CodecErrorKind::Unsupported);
    }

    #[tokio::test]
    async fn all_four_variants_work() {
        let reg = global_registry();
        for name in ["pcm_s16le", "pcm_s32le", "pcm_f32le", "pcm_f64le"] {
            let mut dec = reg.create_decoder(name, cfg_stereo_48k()).unwrap();
            // Use 16 bytes — divisible by every stride (2,4,4,8) × 2 ch
            let data = Bytes::from(vec![0u8; 16]);
            let frame = dec.decode(DecodeRequest::new(data)).await.unwrap();
            assert!(!frame.payload.is_empty(), "{name} produced empty frame");
        }
    }

    #[tokio::test]
    async fn explicit_pts_overrides_counter() {
        let reg = global_registry();
        let mut dec = reg.create_decoder("pcm_s16le", cfg_stereo_48k()).unwrap();

        let data = Bytes::from(vec![0u8; 8]);
        let f = dec
            .decode(DecodeRequest::with_pts(data, 9_000_000))
            .await
            .unwrap();
        assert_eq!(f.pts, 9_000_000);
    }
}