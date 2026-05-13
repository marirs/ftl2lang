use crate::error::AppError;
use async_trait::async_trait;
use indicatif::ProgressBar;

pub mod deepl;
pub mod factory;
pub mod google;
pub mod gtranslate;
pub mod mock;

/// A translation backend (DeepL, Google Cloud, gtranslate, ...).
///
/// The trait is async because every real implementation makes an HTTP request.
/// `Send + Sync` is required so trait objects can cross `.await` points in the
/// pipeline.
#[async_trait]
pub trait Translator: Send + Sync {
    /// Stable backend identifier used in logs, cache keys, and the side-car
    /// file (e.g. `"deepl"`, `"google"`, `"gtranslate"`, `"mock"`).
    fn name(&self) -> &'static str;

    /// Whether this backend supports `target_lang` (an already-normalized
    /// BCP-47 primary tag, e.g. `"de"` or `"pt-BR"`). Implementations should
    /// be permissive when in doubt — gating happens once before any request
    /// is made.
    fn supports(&self, target_lang: &str) -> bool;

    /// Translate a batch of texts from `source_lang` to `target_lang`.
    ///
    /// # Contract
    ///
    /// On `Ok`, the returned `Vec` has exactly `texts.len()` entries, one per
    /// input in the same order. The splice step relies on this length and
    /// order to map translations back to AST spans; an implementation that
    /// filters, reorders, or merges inputs will produce corrupted output.
    ///
    /// # Progress reporting
    ///
    /// If `progress` is `Some`, implementations should call `bar.inc(n)`
    /// after completing each internal chunk or request so the user sees
    /// movement on long translations. Granularity is implementation-defined
    /// (gtranslate: one tick per text; DeepL/Google: one tick per chunk of
    /// up to 50 / 100 texts) but the sum of all increments MUST equal
    /// `texts.len()` on success. The bar is created and managed by the
    /// caller; impls must not call `finish()` on it.
    async fn translate_batch(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
        progress: Option<&ProgressBar>,
    ) -> Result<Vec<String>, AppError>;
}
