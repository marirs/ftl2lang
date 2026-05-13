use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Google Cloud Translation v3 supports 100+ texts per request and ~30KB
/// total request size. We pick 100 as a conservative chunk size to stay
/// well under both bounds. Larger files are split into multiple sequential
/// requests; the returned Vec preserves input order.
const GOOGLE_MAX_TEXTS_PER_REQUEST: usize = 100;

pub struct GoogleTranslator {
    client: Client,
    api_key: String,
    project_id: String,
}

impl GoogleTranslator {
    pub fn new(api_key: String, project_id: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client build");
        Self { client, api_key, project_id }
    }

    async fn translate_chunk(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, AppError> {
        let url = format!(
            "https://translation.googleapis.com/v3/projects/{}:translateText?key={}",
            self.project_id, self.api_key
        );
        let body = TranslateRequest {
            contents: texts.iter().map(|t| (*t).to_string()).collect(),
            source_language_code: source_lang.into(),
            target_language_code: target_lang.into(),
            mime_type: "text/plain",
        };

        let mut delay_ms = 1000u64;
        let mut last_err: Option<AppError> = None;
        for attempt in 1..=3 {
            let resp = self.client.post(&url).json(&body).send().await;
            match resp.and_then(|r| r.error_for_status()) {
                Ok(r) => {
                    let parsed: TranslateResponse = r
                        .json()
                        .await
                        .map_err(|e| AppError::Api(format!("google body: {}", e)))?;
                    return Ok(parsed.translations.into_iter().map(|t| t.translated_text).collect());
                }
                Err(e) if attempt < 3 => {
                    last_err = Some(AppError::Api(format!("google request: {}", e)));
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2;
                }
                Err(e) => return Err(AppError::Api(format!("google request: {}", e))),
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::Api("google: retries exhausted".into())))
    }
}

#[derive(Serialize)]
struct TranslateRequest {
    contents: Vec<String>,
    #[serde(rename = "sourceLanguageCode")]
    source_language_code: String,
    #[serde(rename = "targetLanguageCode")]
    target_language_code: String,
    #[serde(rename = "mimeType")]
    mime_type: &'static str,
}

#[derive(Deserialize)]
struct TranslateResponse {
    translations: Vec<TranslationItem>,
}

#[derive(Deserialize)]
struct TranslationItem {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[async_trait]
impl Translator for GoogleTranslator {
    fn name(&self) -> &'static str {
        "google"
    }

    fn supports(&self, _target_lang: &str) -> bool {
        // Google Cloud Translation covers 130+ languages; rather than maintain
        // a stale list, we accept any code and let the API itself surface
        // unsupported targets via 4xx (handled by the retry loop).
        true
    }

    async fn translate_batch(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, AppError> {
        let mut out = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(GOOGLE_MAX_TEXTS_PER_REQUEST) {
            let translated = self.translate_chunk(chunk, source_lang, target_lang).await?;
            out.extend(translated);
        }
        Ok(out)
    }
}
