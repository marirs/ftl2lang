use super::Translator;
use crate::error::AppError;
use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use indicatif::ProgressBar;
use reqwest::Client;
use std::time::Duration;

/// Maximum number of concurrent in-flight requests against the unofficial
/// endpoint. Higher values risk a soft IP ban; lower values are slow on
/// large files. 4 matches the plan and the conventional "be polite" value
/// for unofficial Google endpoints.
const GTRANSLATE_CONCURRENCY: usize = 4;

/// Unofficial Google translate endpoint. Best-effort, may break without notice.
/// One HTTP request per text; bounded concurrency to avoid rate-limiting.
pub struct GtranslateTranslator {
    client: Client,
    verbose: bool,
}

impl GtranslateTranslator {
    pub fn new() -> Self {
        // v3 differs: a browser-like User-Agent is required for the
        // unofficial endpoint to respond. Revisit if Google hardens it.
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("Mozilla/5.0 (compatible; ftl2lang/0.1)")
            .build()
            .expect("reqwest client build");
        Self {
            client,
            verbose: false,
        }
    }

    /// Enable per-request diagnostic logging to stderr.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
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
        progress: Option<&ProgressBar>,
    ) -> Result<Vec<String>, AppError> {
        // Materialise inputs as owned Strings so the per-future closure is
        // free of borrowed references from `texts`; this keeps the higher-
        // rank lifetimes that `buffered` requires happy.
        let src = source_lang.to_string();
        let tgt = target_lang.to_string();
        let owned: Vec<String> = texts.iter().map(|t| t.to_string()).collect();

        // `buffered(N)` runs at most N futures concurrently and preserves
        // input order, so the returned Vec aligns with `texts`. We drain it
        // with a while-loop so the progress bar can tick after each yield.
        if self.verbose {
            eprintln!(
                "gtranslate: {} request(s) {}→{}, up to {} concurrent",
                texts.len(),
                source_lang,
                target_lang,
                GTRANSLATE_CONCURRENCY
            );
        }

        let mut stream = stream::iter(
            owned
                .into_iter()
                .map(|text| translate_one(&self.client, text, src.clone(), tgt.clone())),
        )
        .buffered(GTRANSLATE_CONCURRENCY);

        let mut out = Vec::with_capacity(texts.len());
        while let Some(res) = stream.next().await {
            out.push(res?);
            if let Some(bar) = progress {
                bar.inc(1);
            }
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
    let segments = body.get(0).and_then(|v| v.as_array()).ok_or_else(|| {
        AppError::Api(format!(
            "gtranslate: unexpected response shape; body={}",
            body
        ))
    })?;

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
