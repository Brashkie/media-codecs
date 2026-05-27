//! Codec traits — [`Codec`], [`Encoder`], [`Decoder`].
//!
//! Every codec in the Kryx ecosystem implements [`Codec`] (metadata) and at
//! least one of [`Encoder`] or [`Decoder`] (the actual work).
//!
//! Codecs are async because real-world implementations (and especially future
//! Zig-backed ones) may schedule work onto a thread pool. PCM is synchronous
//! in practice but still exposes the async surface for uniformity.

use crate::error::CodecResult;
use async_trait::async_trait;
use bitflags::bitflags;
use bytes::Bytes;

// ─── MediaKind ──────────────────────────────────────────────────────────────

/// What kind of media a codec handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    /// Audio codecs (PCM, Opus, AAC, ...).
    Audio,
    /// Video codecs (H.264, AV1, VP9, ...).
    Video,
    /// Subtitle codecs (SRT, WebVTT, ASS, ...).
    Subtitle,
}

impl MediaKind {
    /// Stable kebab-case identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            MediaKind::Audio => "audio",
            MediaKind::Video => "video",
            MediaKind::Subtitle => "subtitle",
        }
    }
}

// ─── Capabilities ───────────────────────────────────────────────────────────

bitflags! {
    /// What a codec can do.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CodecCapabilities: u32 {
        /// Can decode bitstreams of this codec.
        const DECODE = 1 << 0;
        /// Can encode raw samples into this codec.
        const ENCODE = 1 << 1;
        /// Decoded output is lossless w.r.t. the encoded input.
        const LOSSLESS = 1 << 2;
        /// Uses hardware acceleration (GPU, dedicated ASIC).
        const HARDWARE = 1 << 3;
        /// Stateless — does not retain context between frames.
        const STATELESS = 1 << 4;
    }
}

// ─── Sample / pixel formats ─────────────────────────────────────────────────

/// Raw audio sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SampleFormat {
    /// 16-bit signed integer, interleaved, little-endian.
    S16,
    /// 32-bit signed integer, interleaved, little-endian.
    S32,
    /// 32-bit float, interleaved, native endian.
    F32,
    /// 64-bit float, interleaved, native endian.
    F64,
    /// 16-bit signed integer, planar (one buffer per channel).
    S16Planar,
    /// 32-bit float, planar (one buffer per channel).
    F32Planar,
}

impl SampleFormat {
    /// Bytes per single sample (one channel).
    #[must_use]
    pub const fn bytes_per_sample(self) -> usize {
        match self {
            SampleFormat::S16 | SampleFormat::S16Planar => 2,
            SampleFormat::S32 | SampleFormat::F32 | SampleFormat::F32Planar => 4,
            SampleFormat::F64 => 8,
        }
    }

    /// Whether samples are stored planar (one buffer per channel) vs interleaved.
    #[must_use]
    pub const fn is_planar(self) -> bool {
        matches!(self, SampleFormat::S16Planar | SampleFormat::F32Planar)
    }

    /// Stable kebab-case identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SampleFormat::S16 => "s16",
            SampleFormat::S32 => "s32",
            SampleFormat::F32 => "f32",
            SampleFormat::F64 => "f64",
            SampleFormat::S16Planar => "s16p",
            SampleFormat::F32Planar => "f32p",
        }
    }
}

// ─── Codec descriptor ───────────────────────────────────────────────────────

/// Static metadata about a codec — name, family, kind, capabilities.
///
/// This is what the [`crate::registry::CodecRegistry`] indexes and exposes
/// to JS callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecDescriptor {
    /// Stable lowercase identifier (`"pcm_s16le"`, `"opus"`, `"h264"`).
    pub name: &'static str,
    /// Human-readable long name (`"PCM signed 16-bit little-endian"`).
    pub long_name: &'static str,
    /// Whether this is audio / video / subtitle.
    pub kind: MediaKind,
    /// What this codec can do (encode / decode / hardware / ...).
    pub capabilities: CodecCapabilities,
}

// ─── Codec trait ────────────────────────────────────────────────────────────

/// Static metadata-only trait every codec implements.
///
/// All codecs must also implement [`std::fmt::Debug`] so that callers can
/// use `Result::unwrap` / `Result::expect` on `create_decoder` / `create_encoder`
/// results (these return `Box<dyn Decoder>` / `Box<dyn Encoder>`).
pub trait Codec: std::fmt::Debug + Send + Sync + 'static {
    /// Return the static descriptor.
    fn descriptor(&self) -> &'static CodecDescriptor;
}

// ─── Config (shared) ────────────────────────────────────────────────────────

/// Options passed when instantiating an encoder or decoder.
///
/// Codecs may ignore options they don't understand; required-but-missing
/// options return [`crate::CodecErrorKind::Unsupported`].
#[derive(Debug, Clone, Default)]
pub struct CodecConfig {
    /// Audio sample rate in Hz (e.g. `48_000`).
    pub sample_rate: Option<u32>,
    /// Number of channels (e.g. `2` for stereo).
    pub channels: Option<u16>,
    /// Sample format for raw audio.
    pub sample_format: Option<SampleFormat>,
    /// Bitrate hint for encoders (bits/sec).
    pub bitrate: Option<u32>,
    /// Codec-specific extra data (`SpecificConfig`, `csd-0`, ...).
    pub extra_data: Option<Bytes>,
}

impl CodecConfig {
    /// Empty config — every field `None`.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            sample_rate: None,
            channels: None,
            sample_format: None,
            bitrate: None,
            extra_data: None,
        }
    }

    /// Helper: required sample rate, or fall back to a default.
    #[must_use]
    pub fn sample_rate_or(&self, default: u32) -> u32 {
        self.sample_rate.unwrap_or(default)
    }

    /// Helper: required channel count, or 1 (mono).
    #[must_use]
    pub fn channels_or_mono(&self) -> u16 {
        self.channels.unwrap_or(1)
    }
}

// ─── Frames ─────────────────────────────────────────────────────────────────

/// A decoded frame produced by a [`Decoder`] or fed to an [`Encoder`].
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Raw sample/pixel data.
    pub payload: Bytes,
    /// Presentation timestamp (codec-defined units).
    pub pts: i64,
    /// Decode timestamp; may equal `pts` for codecs without B-frames.
    pub dts: i64,
    /// Whether this is a keyframe / IDR / sync frame.
    pub is_keyframe: bool,
    /// Frame duration in PTS units (or 0 if unknown).
    pub duration: i64,
}

impl DecodedFrame {
    /// Construct a new decoded frame.
    #[must_use]
    pub fn new(payload: Bytes, pts: i64) -> Self {
        Self {
            payload,
            pts,
            dts: pts,
            is_keyframe: false,
            duration: 0,
        }
    }
}

/// An encoded packet produced by an [`Encoder`] or fed to a [`Decoder`].
#[derive(Debug, Clone)]
pub struct EncodedPacket {
    /// Encoded bitstream.
    pub payload: Bytes,
    /// Presentation timestamp.
    pub pts: i64,
    /// Decode timestamp.
    pub dts: i64,
    /// Sync point — decoder can start here cleanly.
    pub is_keyframe: bool,
    /// Duration in PTS units.
    pub duration: i64,
}

impl EncodedPacket {
    /// Construct a new encoded packet.
    #[must_use]
    pub fn new(payload: Bytes, pts: i64) -> Self {
        Self {
            payload,
            pts,
            dts: pts,
            is_keyframe: false,
            duration: 0,
        }
    }
}

// ─── Decode / Encode requests ──────────────────────────────────────────────

/// Single decode call input.
#[derive(Debug, Clone)]
pub struct DecodeRequest {
    /// Encoded bytes to decode.
    pub data: Bytes,
    /// Optional PTS hint.
    pub pts: Option<i64>,
}

impl DecodeRequest {
    /// Build a request from raw bytes.
    #[must_use]
    pub fn new(data: Bytes) -> Self {
        Self { data, pts: None }
    }

    /// Build a request with a PTS hint.
    #[must_use]
    pub fn with_pts(data: Bytes, pts: i64) -> Self {
        Self {
            data,
            pts: Some(pts),
        }
    }
}

/// Single encode call input.
#[derive(Debug, Clone)]
pub struct EncodeRequest {
    /// Raw samples / pixels to encode.
    pub frame: DecodedFrame,
}

impl EncodeRequest {
    /// Wrap a decoded frame.
    #[must_use]
    pub fn new(frame: DecodedFrame) -> Self {
        Self { frame }
    }
}

// ─── Decoder + Encoder traits ──────────────────────────────────────────────

/// Stateful decoder for one stream.
///
/// Instances are typically created via
/// [`crate::registry::CodecRegistry::create_decoder`].
#[async_trait]
pub trait Decoder: Codec + Send {
    /// Decode a single packet into a frame.
    ///
    /// May return an empty payload for codecs that need more data
    /// (e.g. multi-packet frames). Use [`Decoder::flush`] at end of stream.
    async fn decode(&mut self, req: DecodeRequest) -> CodecResult<DecodedFrame>;

    /// Flush any buffered frames and clean up state.
    async fn flush(&mut self) -> CodecResult<Vec<DecodedFrame>> {
        Ok(Vec::new())
    }

    /// Reset internal state (e.g. on seek). Default: no-op.
    async fn reset(&mut self) -> CodecResult<()> {
        Ok(())
    }
}

/// Stateful encoder for one stream.
#[async_trait]
pub trait Encoder: Codec + Send {
    /// Encode a single frame into a packet.
    async fn encode(&mut self, req: EncodeRequest) -> CodecResult<EncodedPacket>;

    /// Flush any buffered packets at end of stream.
    async fn flush(&mut self) -> CodecResult<Vec<EncodedPacket>> {
        Ok(Vec::new())
    }

    /// Reset internal state. Default: no-op.
    async fn reset(&mut self) -> CodecResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_format_sizes() {
        assert_eq!(SampleFormat::S16.bytes_per_sample(), 2);
        assert_eq!(SampleFormat::F32.bytes_per_sample(), 4);
        assert_eq!(SampleFormat::F64.bytes_per_sample(), 8);
        assert!(SampleFormat::F32Planar.is_planar());
        assert!(!SampleFormat::F32.is_planar());
    }

    #[test]
    fn capabilities_combine() {
        let caps = CodecCapabilities::DECODE | CodecCapabilities::ENCODE;
        assert!(caps.contains(CodecCapabilities::DECODE));
        assert!(caps.contains(CodecCapabilities::ENCODE));
        assert!(!caps.contains(CodecCapabilities::HARDWARE));
    }

    #[test]
    fn config_helpers() {
        let mut cfg = CodecConfig::empty();
        assert_eq!(cfg.sample_rate_or(48000), 48000);
        assert_eq!(cfg.channels_or_mono(), 1);

        cfg.sample_rate = Some(44_100);
        cfg.channels = Some(2);
        assert_eq!(cfg.sample_rate_or(48000), 44_100);
        assert_eq!(cfg.channels_or_mono(), 2);
    }

    #[test]
    fn frame_defaults_dts_to_pts() {
        let f = DecodedFrame::new(Bytes::from_static(b"x"), 90_000);
        assert_eq!(f.pts, 90_000);
        assert_eq!(f.dts, 90_000);
        assert!(!f.is_keyframe);
    }

    #[test]
    fn packet_defaults_dts_to_pts() {
        let p = EncodedPacket::new(Bytes::from_static(b"x"), 100);
        assert_eq!(p.pts, 100);
        assert_eq!(p.dts, 100);
    }

    #[test]
    fn decode_request_constructors() {
        let r1 = DecodeRequest::new(Bytes::from_static(b"a"));
        assert!(r1.pts.is_none());

        let r2 = DecodeRequest::with_pts(Bytes::from_static(b"b"), 42);
        assert_eq!(r2.pts, Some(42));
    }

    #[test]
    fn media_kind_strings() {
        assert_eq!(MediaKind::Audio.as_str(), "audio");
        assert_eq!(MediaKind::Video.as_str(), "video");
        assert_eq!(MediaKind::Subtitle.as_str(), "subtitle");
    }
}