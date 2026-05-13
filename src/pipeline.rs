use crate::error::AppError;
use crate::ftl::splice::splice_translations;
use crate::ftl::walk::walk_source;
use crate::translator::Translator;

/// Translate the textual content of a single .ftl file end-to-end.
pub async fn translate_file_content<T: Translator + ?Sized>(
    src: &str,
    source_lang: &str,
    target_lang: &str,
    translator: &T,
) -> Result<String, AppError> {
    let spans = walk_source(src, None)?;
    if spans.is_empty() {
        return Ok(src.to_string());
    }
    let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
    let translations = translator.translate_batch(&texts, source_lang, target_lang).await?;
    splice_translations(src, &spans, &translations, None)
}
