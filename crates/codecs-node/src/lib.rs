//! napi-rs bindings for `@brashkie/media-codecs`.
//!
//! Exposes the codec registry, decoders, and encoders to the JavaScript layer.
//! Each `#[napi]` block on the Rust side becomes a class or function in
//! `index.d.ts`.

#![deny(clippy::all)]

use mcd_core::codec::{
    CodecConfig as CoreConfig, DecodeRequest, DecodedFrame, Decoder as CoreDecoder, EncodeRequest,
    EncodedPacket, Encoder as CoreEncoder, MediaKind, SampleFormat,
};
use mcd_core::registry::global_registry;
use mcd_core::CodecError;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::Mutex;

// ─── Module version ─────────────────────────────────────────────────────────

/// Returns the version of the linked native addon.
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ─── Error conversion ───────────────────────────────────────────────────────

fn to_napi(err: CodecError) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("[{}] {}", err.kind(), err.message()),
    )
}

// ─── Config (JS-facing) ─────────────────────────────────────────────────────

/// Configuration options for instantiating a codec.
#[napi(object)]
pub struct JsCodecConfig {
    /// Audio sample rate in Hz (e.g. 48000).
    pub sample_rate: Option<u32>,
    /// Number of channels (e.g. 2 for stereo).
    pub channels: Option<u32>,
    /// Sample format string: "s16", "s32", "f32", "f64", "s16p", "f32p".
    pub sample_format: Option<String>,
    /// Bitrate hint for encoders (bits/sec).
    pub bitrate: Option<u32>,
    /// Codec-specific extra data.
    pub extra_data: Option<Buffer>,
}

impl TryFrom<JsCodecConfig> for CoreConfig {
    type Error = Error;

    fn try_from(c: JsCodecConfig) -> Result<Self> {
        let sample_format = match c.sample_format.as_deref() {
            None => None,
            Some("s16") => Some(SampleFormat::S16),
            Some("s32") => Some(SampleFormat::S32),
            Some("f32") => Some(SampleFormat::F32),
            Some("f64") => Some(SampleFormat::F64),
            Some("s16p") => Some(SampleFormat::S16Planar),
            Some("f32p") => Some(SampleFormat::F32Planar),
            Some(other) => {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("unknown sample format: {other}"),
                ))
            }
        };
        let channels = match c.channels {
            None => None,
            Some(n) if n <= u16::MAX as u32 => Some(n as u16),
            Some(n) => {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("channel count {n} exceeds u16::MAX"),
                ))
            }
        };

        Ok(CoreConfig {
            sample_rate: c.sample_rate,
            channels,
            sample_format,
            bitrate: c.bitrate,
            extra_data: c
                .extra_data
                .map(|b| bytes::Bytes::copy_from_slice(b.as_ref())),
        })
    }
}

// ─── Descriptor (JS-facing) ─────────────────────────────────────────────────

/// Static description of a codec.
#[napi(object)]
pub struct JsCodecDescriptor {
    /// Short stable id (`"pcm_s16le"`, `"opus"`).
    pub name: String,
    /// Long human-readable name.
    pub long_name: String,
    /// "audio" / "video" / "subtitle".
    pub kind: String,
    /// Whether this codec can decode.
    pub can_decode: bool,
    /// Whether this codec can encode.
    pub can_encode: bool,
    /// Whether decoding is bit-exact.
    pub is_lossless: bool,
    /// Whether this codec uses hardware acceleration.
    pub is_hardware: bool,
}

fn kind_string(k: MediaKind) -> String {
    k.as_str().to_string()
}

// ─── Frame / packet (JS-facing) ─────────────────────────────────────────────

/// A decoded frame returned by `JsDecoder.decode` or accepted by `JsEncoder.encode`.
#[napi(object)]
pub struct JsDecodedFrame {
    /// Raw sample/pixel data.
    pub payload: Buffer,
    /// Presentation timestamp.
    pub pts: i64,
    /// Decode timestamp.
    pub dts: i64,
    /// Whether this is a sync point.
    pub is_keyframe: bool,
    /// Duration in codec-defined units.
    pub duration: i64,
}

impl From<DecodedFrame> for JsDecodedFrame {
    fn from(f: DecodedFrame) -> Self {
        Self {
            payload: Buffer::from(f.payload.to_vec()),
            pts: f.pts,
            dts: f.dts,
            is_keyframe: f.is_keyframe,
            duration: f.duration,
        }
    }
}

/// An encoded packet returned by `JsEncoder.encode` or accepted by `JsDecoder.decode`.
#[napi(object)]
pub struct JsEncodedPacket {
    /// Encoded bitstream.
    pub payload: Buffer,
    /// Presentation timestamp.
    pub pts: i64,
    /// Decode timestamp.
    pub dts: i64,
    /// Sync point.
    pub is_keyframe: bool,
    /// Duration in codec-defined units.
    pub duration: i64,
}

impl From<EncodedPacket> for JsEncodedPacket {
    fn from(p: EncodedPacket) -> Self {
        Self {
            payload: Buffer::from(p.payload.to_vec()),
            pts: p.pts,
            dts: p.dts,
            is_keyframe: p.is_keyframe,
            duration: p.duration,
        }
    }
}

// ─── Registry (JS-facing) ───────────────────────────────────────────────────

/// JavaScript handle to the global codec registry.
#[napi]
pub struct JsCodecRegistry {}

#[napi]
impl JsCodecRegistry {
    /// Build a new handle. (All instances share the same global registry.)
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    /// Return the list of all registered codec names (sorted).
    #[napi]
    pub fn names(&self) -> Vec<String> {
        global_registry()
            .names()
            .into_iter()
            .map(String::from)
            .collect()
    }

    /// Return descriptors filtered by media kind.
    /// `kind` must be `"audio"`, `"video"`, `"subtitle"`, or `null`.
    #[napi]
    pub fn list(&self, kind: Option<String>) -> Result<Vec<JsCodecDescriptor>> {
        let filter = match kind.as_deref() {
            None | Some("") => None,
            Some("audio") => Some(MediaKind::Audio),
            Some("video") => Some(MediaKind::Video),
            Some("subtitle") => Some(MediaKind::Subtitle),
            Some(other) => {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("unknown kind: {other}"),
                ))
            }
        };

        Ok(global_registry()
            .list(filter)
            .into_iter()
            .map(|d| JsCodecDescriptor {
                name: d.name.to_string(),
                long_name: d.long_name.to_string(),
                kind: kind_string(d.kind),
                can_decode: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::DECODE),
                can_encode: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::ENCODE),
                is_lossless: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::LOSSLESS),
                is_hardware: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::HARDWARE),
            })
            .collect())
    }

    /// Find a codec by name. Returns `null` if not found.
    #[napi]
    pub fn find(&self, name: String) -> Option<JsCodecDescriptor> {
        global_registry().find(&name).map(|r| {
            let d = r.descriptor;
            JsCodecDescriptor {
                name: d.name.to_string(),
                long_name: d.long_name.to_string(),
                kind: kind_string(d.kind),
                can_decode: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::DECODE),
                can_encode: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::ENCODE),
                is_lossless: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::LOSSLESS),
                is_hardware: d
                    .capabilities
                    .contains(mcd_core::codec::CodecCapabilities::HARDWARE),
            }
        })
    }
}

impl Default for JsCodecRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Decoder (JS-facing) ────────────────────────────────────────────────────

type DecoderHandle = Arc<Mutex<Box<dyn CoreDecoder>>>;

/// Stateful decoder. Build with [`createDecoder`].
#[napi]
pub struct JsDecoder {
    name: String,
    inner: DecoderHandle,
}

#[napi]
impl JsDecoder {
    /// Codec name (e.g. `"pcm_s16le"`).
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Decode one packet and return the resulting frame.
    #[napi]
    pub async fn decode(
        &self,
        data: Buffer,
        pts: Option<i64>,
    ) -> Result<JsDecodedFrame> {
        let req = if let Some(p) = pts {
            DecodeRequest::with_pts(bytes::Bytes::copy_from_slice(data.as_ref()), p)
        } else {
            DecodeRequest::new(bytes::Bytes::copy_from_slice(data.as_ref()))
        };

        let mut guard = self.inner.lock().await;
        let frame = guard.decode(req).await.map_err(to_napi)?;
        Ok(frame.into())
    }

    /// Flush any buffered frames at end of stream.
    #[napi]
    pub async fn flush(&self) -> Result<Vec<JsDecodedFrame>> {
        let mut guard = self.inner.lock().await;
        let frames = guard.flush().await.map_err(to_napi)?;
        Ok(frames.into_iter().map(Into::into).collect())
    }

    /// Reset internal state (e.g. on seek).
    #[napi]
    pub async fn reset(&self) -> Result<()> {
        let mut guard = self.inner.lock().await;
        guard.reset().await.map_err(to_napi)
    }
}

// ─── Encoder (JS-facing) ────────────────────────────────────────────────────

type EncoderHandle = Arc<Mutex<Box<dyn CoreEncoder>>>;

/// Stateful encoder. Build with [`createEncoder`].
#[napi]
pub struct JsEncoder {
    name: String,
    inner: EncoderHandle,
}

#[napi]
impl JsEncoder {
    /// Codec name (e.g. `"pcm_s16le"`).
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Encode one decoded frame and return the resulting packet.
    #[napi]
    pub async fn encode(&self, frame: JsDecodedFrame) -> Result<JsEncodedPacket> {
        let mut df = DecodedFrame::new(
            bytes::Bytes::copy_from_slice(frame.payload.as_ref()),
            frame.pts,
        );
        df.dts = frame.dts;
        df.is_keyframe = frame.is_keyframe;
        df.duration = frame.duration;

        let mut guard = self.inner.lock().await;
        let pkt = guard
            .encode(EncodeRequest::new(df))
            .await
            .map_err(to_napi)?;
        Ok(pkt.into())
    }

    /// Flush any buffered packets.
    #[napi]
    pub async fn flush(&self) -> Result<Vec<JsEncodedPacket>> {
        let mut guard = self.inner.lock().await;
        let packets = guard.flush().await.map_err(to_napi)?;
        Ok(packets.into_iter().map(Into::into).collect())
    }

    /// Reset internal state.
    #[napi]
    pub async fn reset(&self) -> Result<()> {
        let mut guard = self.inner.lock().await;
        guard.reset().await.map_err(to_napi)
    }
}

// ─── Factories ──────────────────────────────────────────────────────────────

/// Build a decoder for the given codec name.
#[napi(js_name = "createDecoder")]
pub fn create_decoder(name: String, config: Option<JsCodecConfig>) -> Result<JsDecoder> {
    let cfg = match config {
        Some(c) => CoreConfig::try_from(c)?,
        None => CoreConfig::empty(),
    };
    let dec = global_registry()
        .create_decoder(&name, cfg)
        .map_err(to_napi)?;
    Ok(JsDecoder {
        name,
        inner: Arc::new(Mutex::new(dec)),
    })
}

/// Build an encoder for the given codec name.
#[napi(js_name = "createEncoder")]
pub fn create_encoder(name: String, config: Option<JsCodecConfig>) -> Result<JsEncoder> {
    let cfg = match config {
        Some(c) => CoreConfig::try_from(c)?,
        None => CoreConfig::empty(),
    };
    let enc = global_registry()
        .create_encoder(&name, cfg)
        .map_err(to_napi)?;
    Ok(JsEncoder {
        name,
        inner: Arc::new(Mutex::new(enc)),
    })
}
