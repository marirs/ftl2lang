use crate::error::AppError;
use whatlang::{detect, Lang};

#[derive(Debug, Clone)]
pub struct DetectedLanguage {
    /// ISO 639-1 two-letter code (e.g. "en", "de"). Never "und" — callers
    /// that receive an `Ok` can pass `code` straight to a translator backend.
    pub code: String,
    /// whatlang confidence in `[0.0, 1.0]`. Detection on short text often
    /// falls below ~0.70; the CLI should treat low-confidence results as
    /// "user must confirm or supply --from explicitly."
    pub confidence: f64,
}

/// Joins all provided text samples and uses whatlang to identify the language.
///
/// Returns `Err` when:
/// - the slice is empty,
/// - whatlang cannot identify any language (very short or noisy input), or
/// - whatlang identifies a language not in our backend-supported map.
///
/// The third case is collapsed into a single error rather than returning
/// `code = "und"` so callers cannot accidentally pass an undetermined code
/// to a translator backend.
pub fn detect_source_language(texts: &[&str]) -> Result<DetectedLanguage, AppError> {
    if texts.is_empty() {
        return Err(AppError::Other(
            "cannot detect language: no text provided".into(),
        ));
    }
    let combined = texts.join(" ");
    let info =
        detect(&combined).ok_or_else(|| AppError::Other("language detection failed".into()))?;
    let code = lang_to_iso639_1(info.lang());
    if code == "und" {
        return Err(AppError::Other(format!(
            "detected language {:?} is not in the supported set; pass --from explicitly",
            info.lang()
        )));
    }
    Ok(DetectedLanguage {
        code: code.to_string(),
        confidence: info.confidence(),
    })
}

/// Maps whatlang's internal `Lang` enum to ISO 639-1 two-letter codes.
/// Only the subset of languages supported by our translation backends is listed;
/// everything else falls through to "und" (ISO 639-3 "undetermined").
// v3 differs: whatlang bumped from 0.16 → 0.18 during Task 0 review; variant
// names are unchanged but the crate added several new langs — they all hit `_`.
fn lang_to_iso639_1(lang: Lang) -> &'static str {
    match lang {
        Lang::Eng => "en",
        Lang::Deu => "de",
        Lang::Fra => "fr",
        Lang::Spa => "es",
        Lang::Ita => "it",
        Lang::Por => "pt",
        Lang::Nld => "nl",
        Lang::Pol => "pl",
        Lang::Rus => "ru",
        Lang::Jpn => "ja",
        Lang::Kor => "ko",
        Lang::Cmn => "zh",
        Lang::Tam => "ta",
        Lang::Hin => "hi",
        Lang::Tha => "th",
        Lang::Vie => "vi",
        Lang::Ind => "id",
        Lang::Ara => "ar",
        Lang::Heb => "he",
        Lang::Pes => "fa",
        Lang::Tur => "tr",
        _ => "und",
    }
}
