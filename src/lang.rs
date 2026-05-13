/// Normalise a BCP-47-ish language tag:
/// - strips surrounding whitespace
/// - lowercases the primary subtag (e.g. "DE" → "de")
/// - uppercases the region subtag when present (e.g. "pt-br" → "pt-BR")
///
/// Only the first `-` separator is treated as the lang/region boundary; further
/// subtags (script, variant, …) are left untouched for now.
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
        "en" => "English",
        "de" => "German",
        "fr" => "French",
        "es" => "Spanish",
        "it" => "Italian",
        "pt" => "Portuguese",
        "nl" => "Dutch",
        "pl" => "Polish",
        "ru" => "Russian",
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "ta" => "Tamil",
        "hi" => "Hindi",
        "th" => "Thai",
        "vi" => "Vietnamese",
        "id" => "Indonesian",
        "ar" => "Arabic",
        "he" => "Hebrew",
        "fa" => "Persian",
        "tr" => "Turkish",
        _ => code,
    }
    .to_string()
}
