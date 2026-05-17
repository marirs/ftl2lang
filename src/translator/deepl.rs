use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;
use indicatif::ProgressBar;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// All language codes that DeepL supports (lowercase BCP-47 primary tags).
///
/// v3 differs: we gate on this list before issuing any request so callers
/// receive a clear `supports() == false` rather than an opaque HTTP 400 from
/// the DeepL API for unsupported targets (e.g. "ta", "hi").
pub const DEEPL_SUPPORTED_LANGS: &[&str] = &[
    "ar", "bg", "cs", "da", "de", "el", "en", "es", "et", "fi", "fr", "hu", "id", "it", "ja", "ko",
    "lt", "lv", "nb", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv", "tr", "uk", "zh",
];

/// DeepL caps a single `/translate` request at 50 texts. Larger batches are
/// chunked across multiple sequential requests; the trait contract that the
/// returned `Vec` matches input length and order is preserved by concatenating
/// chunk results in order.
const DEEPL_MAX_TEXTS_PER_REQUEST: usize = 50;

pub struct DeeplTranslator {
    client: Client,
    api_key: String,
    /// Base URL for the DeepL REST API (without trailing slash).
    /// Defaults to the free-tier endpoint; override for pro or mock servers.
    api_url: String,
    verbose: bool,
}

impl DeeplTranslator {
    /// `api_url` defaults to the free-tier endpoint when `None`.
    pub fn new(api_key: String, api_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client build");
        Self {
            client,
            api_key,
            api_url: api_url.unwrap_or_else(|| "https://api-free.deepl.com/v2".into()),
            verbose: false,
        }
    }

    /// Enable per-request diagnostic logging to stderr.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    async fn translate_chunk(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, AppError> {
        let url = format!("{}/translate", self.api_url.trim_end_matches('/'));

        // DeepL's form API accepts repeated `text` fields for batch requests.
        // We do NOT set `tag_handling=xml`: the FTL walker has already
        // stripped placeables, so DeepL only sees literal text. Activating
        // the XML parser on plain text would mangle stray `<`, `>`, or `&`
        // characters that appear naturally inside translation units.
        let mut form: Vec<(&str, String)> = Vec::new();
        for t in texts {
            form.push(("text", (*t).to_string()));
        }
        form.push(("source_lang", source_lang.to_uppercase()));
        form.push(("target_lang", target_lang.to_uppercase()));
        form.push(("preserve_formatting", "1".into()));

        // Retry up to 3 times with exponential back-off (1 s → 2 s) on
        // transient network or 5xx errors. 4xx (auth, quota, oversize) fail
        // immediately because `error_for_status` propagates them.
        let mut delay_ms = 1000u64;
        let mut last_err: Option<AppError> = None;
        for attempt in 1..=3 {
            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
                .form(&form)
                .send()
                .await;

            match resp.and_then(|r| r.error_for_status()) {
                Ok(r) => {
                    let parsed: DeeplResponse = r
                        .json()
                        .await
                        .map_err(|e| AppError::Api(format!("deepl body: {}", e)))?;
                    return Ok(parsed.translations.into_iter().map(|t| t.text).collect());
                }
                Err(e) if attempt < 3 => {
                    last_err = Some(AppError::Api(format!("deepl request: {}", e)));
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                }
                Err(e) => return Err(AppError::Api(format!("deepl request: {}", e))),
            }
        }
        // Defensive: the `attempt < 3` arm above always sets `last_err`, and
        // the `attempt == 3` arm returns directly; this is unreachable in
        // practice but avoids `unreachable!()` so the invariant doesn't have
        // to be visually verified against the loop bounds.
        Err(last_err.unwrap_or_else(|| AppError::Api("deepl: retries exhausted".into())))
    }
}

#[derive(Deserialize)]
struct DeeplResponse {
    translations: Vec<DeeplTranslation>,
}

#[derive(Deserialize)]
struct DeeplTranslation {
    text: String,
}

#[async_trait]
impl Translator for DeeplTranslator {
    fn name(&self) -> &'static str {
        "deepl"
    }

    fn supports(&self, target_lang: &str) -> bool {
        // Normalise "pt-BR" → "pt" before checking the supported-lang list.
        let base = target_lang
            .split('-')
            .next()
            .unwrap_or(target_lang)
            .to_lowercase();
        DEEPL_SUPPORTED_LANGS.contains(&base.as_str())
    }

    async fn translate_batch(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
        progress: Option<&ProgressBar>,
    ) -> Result<Vec<String>, AppError> {
        if self.verbose {
            let chunks = texts.len().div_ceil(DEEPL_MAX_TEXTS_PER_REQUEST);
            eprintln!(
                "deepl: {} text(s) {}→{} in {} request(s) (≤{} per request)",
                texts.len(),
                source_lang,
                target_lang,
                chunks,
                DEEPL_MAX_TEXTS_PER_REQUEST
            );
        }
        let mut out = Vec::with_capacity(texts.len());
        for (i, chunk) in texts.chunks(DEEPL_MAX_TEXTS_PER_REQUEST).enumerate() {
            if self.verbose {
                eprintln!("deepl:   request {} — {} text(s)", i + 1, chunk.len());
            }
            let translated = self
                .translate_chunk(chunk, source_lang, target_lang)
                .await?;
            out.extend(translated);
            if let Some(bar) = progress {
                bar.inc(chunk.len() as u64);
            }
        }
        Ok(out)
    }
}
