use crate::error::AppError;
use whatlang::{detect, Lang};

pub struct DetectedLanguage {
    pub code: String,
    pub confidence: f64,
}

/// Joins all provided text samples and uses whatlang to identify the language.
/// Returns an error if the slice is empty or whatlang cannot make a determination.
/// The returned `code` is an ISO 639-1 two-letter tag (e.g. "en", "de"), or "und"
/// for languages we haven't mapped yet — callers should treat "und" as "user must
/// supply --from explicitly".
pub fn detect_source_language(texts: &[String]) -> Result<DetectedLanguage, AppError> {
    if texts.is_empty() {
        return Err(AppError::Other("cannot detect language: no text provided".into()));
    }
    let combined = texts.join(" ");
    let info = detect(&combined)
        .ok_or_else(|| AppError::Other("language detection failed".into()))?;
    Ok(DetectedLanguage {
        code: lang_to_iso639_1(info.lang()).to_string(),
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
