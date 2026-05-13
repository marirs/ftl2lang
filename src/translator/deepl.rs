use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// All language codes that DeepL supports (lowercase BCP-47 primary tags).
///
/// v3 differs: we gate on this list before issuing any request so callers
/// receive a clear `supports() == false` rather than an opaque HTTP 400 from
/// the DeepL API for unsupported targets (e.g. "ta", "hi").
pub const DEEPL_SUPPORTED_LANGS: &[&str] = &[
    "ar", "bg", "cs", "da", "de", "el", "en", "es", "et", "fi", "fr", "hu", "id", "it", "ja",
    "ko", "lt", "lv", "nb", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv", "tr", "uk", "zh",
];

pub struct DeeplTranslator {
    client: Client,
    api_key: String,
    /// Base URL for the DeepL REST API (without trailing slash).
    /// Defaults to the free-tier endpoint; override for pro or mock servers.
    api_url: String,
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
        }
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
    ) -> Result<Vec<String>, AppError> {
        let url = format!("{}/translate", self.api_url.trim_end_matches('/'));

        // DeepL's form API accepts repeated `text` fields for batch requests.
        let mut form: Vec<(&str, String)> = Vec::new();
        for t in texts {
            form.push(("text", (*t).to_string()));
        }
        form.push(("source_lang", source_lang.to_uppercase()));
        form.push(("target_lang", target_lang.to_uppercase()));
        // Preserve whitespace / line breaks inside translation units.
        form.push(("preserve_formatting", "1".into()));
        // Treat XML/HTML tags as opaque so FTL markup is not mangled.
        form.push(("tag_handling", "xml".into()));

        // Retry up to 3 times with exponential back-off (1 s → 2 s) on
        // transient network or 5xx errors. 4xx (auth, quota) fail immediately
        // on the third attempt path because `error_for_status` propagates them.
        let mut delay_ms = 1000u64;
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
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                    // Discard the error; we will retry.
                    let _ = e;
                }
                Err(e) => return Err(AppError::Api(format!("deepl request: {}", e))),
            }
        }
        unreachable!()
    }
}
