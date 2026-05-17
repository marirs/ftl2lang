//! "Query" subcommands: --list-translators and --list-langs.
//!
//! These print human-readable info to stdout and are invoked before any
//! config or network work. Kept here (rather than inline in main.rs) so the
//! formatting is unit-testable.

use crate::lang::display_name;
use crate::translator::deepl::DEEPL_SUPPORTED_LANGS;

/// All ISO 639-1 codes we have a friendly display name for. This is also the
/// "known set" we list for backends that accept any code (google, gtranslate).
const KNOWN_CODES: &[&str] = &[
    "en", "de", "fr", "es", "it", "pt", "nl", "pl", "ru", "ja", "ko", "zh", "ta", "hi", "th", "vi",
    "id", "ar", "he", "fa", "tr",
];

/// Render the `--list-translators` output.
pub fn render_translators() -> String {
    let mut out = String::new();
    out.push_str("Available translator backends:\n\n");
    out.push_str("  deepl       Highest quality on supported European + JA/KO/ZH.\n");
    out.push_str("              Requires [deepl].api_key in config.\n");
    out.push_str("  google      Google Cloud Translation v3. 130+ languages incl.\n");
    out.push_str("              TA/HI/TH/AR/HE/FA. Requires [google].api_key + project_id.\n");
    out.push_str("  gtranslate  Unofficial free endpoint. No key. Best-effort;\n");
    out.push_str("              may break without notice. Default when nothing else is\n");
    out.push_str("              configured.\n");
    out.push('\n');
    out.push_str("Select via `--translator <name>` or by setting `default_translator`\n");
    out.push_str("in ~/.config/ftl2lang/config.toml.\n");
    out
}

/// Render the `--list-langs` output: one section per backend.
pub fn render_languages() -> String {
    let mut out = String::new();

    // DeepL — fixed list of supported codes, formatted as "code  Name".
    out.push_str(&format!(
        "[deepl] ({} languages)\n",
        DEEPL_SUPPORTED_LANGS.len()
    ));
    let mut deepl_sorted: Vec<&&str> = DEEPL_SUPPORTED_LANGS.iter().collect();
    deepl_sorted.sort();
    for code in deepl_sorted {
        out.push_str(&format!("  {:<6}{}\n", code, display_name(code)));
    }
    out.push('\n');

    // Google — claims 130+; we print only the codes we have friendly names
    // for, with a footer telling the user the API supports more.
    out.push_str(&format!(
        "[google] ({} listed; backend accepts any BCP-47 code)\n",
        KNOWN_CODES.len()
    ));
    for code in KNOWN_CODES {
        out.push_str(&format!("  {:<6}{}\n", code, display_name(code)));
    }
    out.push_str("  ...   (Google Cloud Translation v3 supports 130+ languages;\n");
    out.push_str("        see https://cloud.google.com/translate/docs/languages)\n\n");

    // gtranslate — same coverage as google (it IS Google's backend).
    out.push_str("[gtranslate] (same coverage as google)\n");
    out.push_str("  Accepts any code that translate.googleapis.com recognizes.\n");

    out
}
