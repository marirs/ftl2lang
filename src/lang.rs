/// Normalise a BCP-47-ish language tag for the two-component shapes this CLI
/// actually accepts ("de", "pt-BR"):
/// - strips surrounding whitespace
/// - lowercases the primary subtag
/// - uppercases everything after the first `-` (treating it as a region)
///
/// Inputs with script or variant subtags (e.g. "zh-Hant", "en-US-POSIX") have
/// every component after the first hyphen uppercased — which is wrong for
/// scripts. This is acceptable here because none of the translation backends
/// nor `whatlang` produce such tags, but if that changes, this helper needs
/// real BCP-47 parsing (e.g. via the `oxilangtag` crate).
pub fn normalize(code: &str) -> String {
    let trimmed = code.trim();
    if let Some((lang, region)) = trimmed.split_once('-') {
        format!("{}-{}", lang.to_lowercase(), region.to_uppercase())
    } else {
        trimmed.to_lowercase()
    }
}

/// Return a human-readable English name for a normalised language code, falling
/// back to the code itself when it is not in the built-in table.
///
/// Only the primary subtag is used for the lookup so that, e.g., "pt-BR" →
/// "Portuguese" rather than a miss.
pub fn display_name(code: &str) -> String {
    let lower = code.to_lowercase();
    let base = lower.split('-').next().unwrap_or(&lower);
    match base {
        // Asian
        "hi" => "Hindi",
        "id" => "Indonesian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "ta" => "Tamil",
        "th" => "Thai",
        "vi" => "Vietnamese",
        "zh" => "Chinese",
        // Middle Eastern
        "ar" => "Arabic",
        "fa" => "Persian",
        "he" => "Hebrew",
        "tr" => "Turkish",
        // European (DeepL coverage + a few more)
        "bg" => "Bulgarian",
        "cs" => "Czech",
        "da" => "Danish",
        "de" => "German",
        "el" => "Greek",
        "en" => "English",
        "es" => "Spanish",
        "et" => "Estonian",
        "fi" => "Finnish",
        "fr" => "French",
        "hu" => "Hungarian",
        "it" => "Italian",
        "lt" => "Lithuanian",
        "lv" => "Latvian",
        "nb" => "Norwegian Bokmål",
        "nl" => "Dutch",
        "pl" => "Polish",
        "pt" => "Portuguese",
        "ro" => "Romanian",
        "ru" => "Russian",
        "sk" => "Slovak",
        "sl" => "Slovenian",
        "sv" => "Swedish",
        "uk" => "Ukrainian",
        _ => code,
    }
    .to_string()
}
