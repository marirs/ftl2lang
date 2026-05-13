use crate::error::AppError;
use crate::ftl::splice::splice_translations;
use crate::ftl::walk::walk_source;
use crate::translator::Translator;
use std::path::Path;

/// Translate the textual content of a single .ftl file end-to-end.
///
/// `path` is reported in parse errors so users can see which file failed.
/// Pass `None` when the source did not come from a file on disk (in-memory
/// strings, tests).
pub async fn translate_file_content<T: Translator + ?Sized>(
    src: &str,
    source_lang: &str,
    target_lang: &str,
    translator: &T,
    path: Option<&Path>,
) -> Result<String, AppError> {
    let spans = walk_source(src, path)?;
    if spans.is_empty() {
        // Fast path: nothing to translate. Return src unchanged without
        // running it through the serializer, which may normalize
        // whitespace. This is intentional: a file with no translatable
        // spans (comments-only, all-placeable messages) should produce
        // byte-identical output.
        return Ok(src.to_string());
    }
    let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
    let translations = translator.translate_batch(&texts, source_lang, target_lang).await?;
    splice_translations(src, &spans, &translations, path)
}
