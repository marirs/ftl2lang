use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;
use futures::stream::{FuturesOrdered, StreamExt};
use reqwest::Client;
use std::time::Duration;

/// Unofficial Google translate endpoint. Best-effort, may break without notice.
/// One HTTP request per text; concurrency-limited via FuturesOrdered.
pub struct GtranslateTranslator {
    client: Client,
}

impl GtranslateTranslator {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("Mozilla/5.0 (compatible; ftl2lang/0.1)")
            .build()
            .expect("reqwest client build");
        Self { client }
    }
}

impl Default for GtranslateTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Translator for GtranslateTranslator {
    fn name(&self) -> &'static str {
        "gtranslate"
    }

    fn supports(&self, _target_lang: &str) -> bool {
        true
    }

    async fn translate_batch(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>, AppError> {
        let mut futs: FuturesOrdered<_> = texts
            .iter()
            .map(|t| {
                translate_one(
                    &self.client,
                    t.to_string(),
                    source_lang.to_string(),
                    target_lang.to_string(),
                )
            })
            .collect();

        let mut out = Vec::with_capacity(texts.len());
        while let Some(res) = futs.next().await {
            out.push(res?);
        }
        Ok(out)
    }
}

async fn translate_one(
    client: &Client,
    text: String,
    src: String,
    tgt: String,
) -> Result<String, AppError> {
    let url = "https://translate.googleapis.com/translate_a/single";
    let resp = retry(3, || async {
        client
            .get(url)
            .query(&[
                ("client", "gtx"),
                ("sl", &src),
                ("tl", &tgt),
                ("dt", "t"),
                ("q", &text),
            ])
            .send()
            .await
            .and_then(|r| r.error_for_status())
    })
    .await
    .map_err(|e| AppError::Api(format!("gtranslate request: {}", e)))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Api(format!("gtranslate body: {}", e)))?;

    // Response shape: [[["translated", "original", ...], ...], ...]
    let segments = body
        .get(0)
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::Api("gtranslate: unexpected response shape".into()))?;

    let mut combined = String::new();
    for seg in segments {
        if let Some(s) = seg.get(0).and_then(|v| v.as_str()) {
            combined.push_str(s);
        }
    }
    Ok(combined)
}

async fn retry<F, Fut, T>(max_tries: usize, mut f: F) -> Result<T, reqwest::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    let mut delay_ms = 1000u64;
    for attempt in 1..=max_tries {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt == max_tries => return Err(e),
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms *= 2;
            }
        }
    }
    unreachable!()
}
