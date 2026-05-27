//! # `mcd-core` — codec framework for `@brashkie/media-codecs`
//!
//! This crate defines the core abstractions every codec in the Kryx ecosystem
//! implements: the [`Codec`] trait, the [`Encoder`] / [`Decoder`] sub-traits,
//! the runtime [`CodecRegistry`], and the codec descriptors that drive
//! discovery from JavaScript.
//!
//! It is intentionally **pure Rust** — no Node, no napi, no Zig. The
//! `codecs-node` crate wraps these types for napi-rs consumers, and future
//! Zig-backed codecs will hook in via the same traits.
//!
//! ## Modules
//!
//! - [`codec`] — [`Codec`], [`Encoder`], [`Decoder`] traits and their config types.
//! - [`registry`] — global [`CodecRegistry`] for runtime lookup by name/id.
//! - [`pcm`] — built-in PCM codecs (s16le, s32le, f32le, f64le).
//! - [`error`] — [`CodecError`] discriminated error type.
//! - [`utils`] — shared internal helpers.
//!
//! ## Quick start
//!
//! ```no_run
//! use mcd_core::{registry::global_registry, codec::DecodeRequest};
//! use bytes::Bytes;
//!
//! # async fn run() -> Result<(), mcd_core::CodecError> {
//! let reg = global_registry();
//! let mut decoder = reg.create_decoder("pcm_s16le", Default::default())?;
//!
//! let raw = Bytes::from_static(&[0x00, 0x10, 0x00, 0x20]);
//! let frame = decoder.decode(DecodeRequest::new(raw)).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(unsafe_op_in_unsafe_fn)]

pub mod codec;
pub mod error;
pub mod pcm;
pub mod registry;
pub mod utils;

pub use error::{CodecError, CodecErrorKind};

/// Crate version, exposed for diagnostics.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ─── Smoke tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn version_constant_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn registry_has_builtin_pcm() {
        let reg = registry::global_registry();
        assert!(reg.find("pcm_s16le").is_some());
        assert!(reg.find("pcm_f32le").is_some());
    }

    #[tokio::test]
    async fn end_to_end_pcm_decode() {
        use crate::codec::DecodeRequest;
        use bytes::Bytes;

        let reg = registry::global_registry();
        let mut dec = match reg.create_decoder("pcm_s16le", Default::default()) {
            Ok(d) => d,
            Err(e) => panic!("create_decoder failed: {e}"),
        };

        let sample: [u8; 4] = [0x00, 0x10, 0x00, 0x20];
        let frame = dec
            .decode(DecodeRequest::new(Bytes::from(sample.to_vec())))
            .await
            .expect("decode ok");

        assert_eq!(frame.payload.len(), 4);
        assert!(frame.is_keyframe);
    }
}