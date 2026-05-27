//! Global codec registry.
//!
//! [`CodecRegistry`] is the runtime lookup table that maps codec names
//! (`"pcm_s16le"`, `"opus"`, ...) to factories that build encoders and
//! decoders on demand.
//!
//! The crate ships a singleton [`global_registry`] populated with all the
//! built-in codecs (currently only PCM). Future packages and user code can
//! register additional codecs via [`CodecRegistry::register`].

use crate::codec::{
    CodecCapabilities, CodecConfig, CodecDescriptor, Decoder, Encoder, MediaKind,
};
use crate::error::{CodecError, CodecResult};
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// ─── Factory types ──────────────────────────────────────────────────────────

/// Factory that produces a decoder instance.
pub type DecoderFactory = Arc<dyn Fn(&CodecConfig) -> CodecResult<Box<dyn Decoder>> + Send + Sync>;

/// Factory that produces an encoder instance.
pub type EncoderFactory = Arc<dyn Fn(&CodecConfig) -> CodecResult<Box<dyn Encoder>> + Send + Sync>;

// ─── Registered entry ───────────────────────────────────────────────────────

/// One entry in the [`CodecRegistry`].
#[derive(Clone)]
pub struct RegisteredCodec {
    /// Static descriptor.
    pub descriptor: &'static CodecDescriptor,
    /// Decoder factory (None if encode-only).
    pub decoder_factory: Option<DecoderFactory>,
    /// Encoder factory (None if decode-only).
    pub encoder_factory: Option<EncoderFactory>,
}

impl std::fmt::Debug for RegisteredCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredCodec")
            .field("descriptor", &self.descriptor)
            .field("has_decoder", &self.decoder_factory.is_some())
            .field("has_encoder", &self.encoder_factory.is_some())
            .finish()
    }
}

// ─── Registry ───────────────────────────────────────────────────────────────

/// Map of codec name → registered codec.
#[derive(Default)]
pub struct CodecRegistry {
    entries: RwLock<HashMap<&'static str, RegisteredCodec>>,
}

impl CodecRegistry {
    /// Build an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a codec under its descriptor name. Overrides any existing
    /// entry with the same name.
    pub fn register(&self, entry: RegisteredCodec) {
        let mut guard = self.entries.write();
        guard.insert(entry.descriptor.name, entry);
    }

    /// Remove a codec by name. Returns `true` if it existed.
    pub fn unregister(&self, name: &str) -> bool {
        let mut guard = self.entries.write();
        guard.remove(name).is_some()
    }

    /// Look up a codec by name without instantiating.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<RegisteredCodec> {
        self.entries.read().get(name).cloned()
    }

    /// Return all registered codec names, sorted.
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        let guard = self.entries.read();
        let mut names: Vec<_> = guard.keys().copied().collect();
        names.sort_unstable();
        names
    }

    /// Return all descriptors filtered by [`MediaKind`].
    #[must_use]
    pub fn list(&self, kind: Option<MediaKind>) -> Vec<&'static CodecDescriptor> {
        let guard = self.entries.read();
        let mut out: Vec<_> = guard
            .values()
            .filter(|e| kind.map_or(true, |k| e.descriptor.kind == k))
            .map(|e| e.descriptor)
            .collect();
        out.sort_by_key(|d| d.name);
        out
    }

    /// Build a decoder instance from a registered codec.
    pub fn create_decoder(
        &self,
        name: &str,
        config: CodecConfig,
    ) -> CodecResult<Box<dyn Decoder>> {
        let entry = self.find(name).ok_or_else(|| CodecError::not_found(name))?;
        let factory = entry.decoder_factory.ok_or_else(|| {
            CodecError::unsupported(format!("codec '{name}' has no decoder"))
                .with_context(name.to_owned())
        })?;
        if !entry.descriptor.capabilities.contains(CodecCapabilities::DECODE) {
            return Err(CodecError::unsupported(format!(
                "codec '{name}' is not declared as a decoder"
            ))
            .with_context(name.to_owned()));
        }
        factory(&config)
    }

    /// Build an encoder instance from a registered codec.
    pub fn create_encoder(
        &self,
        name: &str,
        config: CodecConfig,
    ) -> CodecResult<Box<dyn Encoder>> {
        let entry = self.find(name).ok_or_else(|| CodecError::not_found(name))?;
        let factory = entry.encoder_factory.ok_or_else(|| {
            CodecError::unsupported(format!("codec '{name}' has no encoder"))
                .with_context(name.to_owned())
        })?;
        if !entry.descriptor.capabilities.contains(CodecCapabilities::ENCODE) {
            return Err(CodecError::unsupported(format!(
                "codec '{name}' is not declared as an encoder"
            ))
            .with_context(name.to_owned()));
        }
        factory(&config)
    }
}

// ─── Global singleton ───────────────────────────────────────────────────────

static GLOBAL: Lazy<ArcSwap<CodecRegistry>> = Lazy::new(|| {
    let reg = CodecRegistry::new();
    // Register all built-in codecs.
    crate::pcm::register_all(&reg);
    ArcSwap::from_pointee(reg)
});

/// Access the process-wide global registry.
///
/// Returns an `Arc` so callers can clone freely without locking.
#[must_use]
pub fn global_registry() -> Arc<CodecRegistry> {
    GLOBAL.load_full()
}

/// Replace the global registry with a fresh one. Intended for testing.
pub fn replace_global(reg: CodecRegistry) {
    GLOBAL.store(Arc::new(reg));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::MediaKind;

    #[test]
    fn empty_registry_finds_nothing() {
        let reg = CodecRegistry::new();
        assert!(reg.find("anything").is_none());
        assert_eq!(reg.names().len(), 0);
    }

    #[test]
    fn unregister_returns_existence() {
        let reg = CodecRegistry::new();
        // Register a dummy entry by hand.
        static DESC: CodecDescriptor = CodecDescriptor {
            name: "test",
            long_name: "Test codec",
            kind: MediaKind::Audio,
            capabilities: CodecCapabilities::DECODE,
        };
        reg.register(RegisteredCodec {
            descriptor: &DESC,
            decoder_factory: Some(Arc::new(|_| {
                Err(CodecError::internal("dummy"))
            })),
            encoder_factory: None,
        });
        assert!(reg.unregister("test"));
        assert!(!reg.unregister("test"));
    }

    #[test]
    fn global_registry_has_pcm() {
        let reg = global_registry();
        assert!(reg.find("pcm_s16le").is_some());
        assert!(reg.find("pcm_s32le").is_some());
        assert!(reg.find("pcm_f32le").is_some());
        assert!(reg.find("pcm_f64le").is_some());
    }

    #[test]
    fn list_filters_by_kind() {
        let reg = global_registry();
        let audio = reg.list(Some(MediaKind::Audio));
        assert!(!audio.is_empty());
        assert!(audio.iter().all(|d| d.kind == MediaKind::Audio));

        let video = reg.list(Some(MediaKind::Video));
        assert_eq!(video.len(), 0); // none yet
    }

    #[test]
    fn create_decoder_not_found() {
        let reg = global_registry();
        let err = match reg.create_decoder("does-not-exist", CodecConfig::empty()) {
            Ok(_) => panic!("expected error for unknown codec"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), crate::error::CodecErrorKind::NotFound);
    }

    #[test]
    fn names_are_sorted() {
        let reg = global_registry();
        let names = reg.names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }
}