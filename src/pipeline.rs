use crate::diff::{classify, Verdict};
use crate::error::AppError;
use crate::ftl::splice::splice_translations;
use crate::ftl::walk::{walk_source, TranslatableSpan};
use crate::sidecar::{hash_text, Sidecar, SidecarEntry};
use crate::translator::Translator;
use std::collections::BTreeMap;
use std::path::Path;

/// Translate the textual content of a single .ftl file end-to-end.
///
/// `path` is reported in parse errors so users can see which file failed.
/// Pass `None` when the source did not come from a file on disk (in-memory
/// strings, tests).
pub async fn translate_file_content<T: Translator + ?Sized>(
    src: &str,
    source_lang: &str,
    target_lang: &str,
    translator: &T,
    path: Option<&Path>,
) -> Result<String, AppError> {
    let spans = walk_source(src, path)?;
    if spans.is_empty() {
        // Fast path: nothing to translate. Return src unchanged without
        // running it through the serializer, which may normalize
        // whitespace. This is intentional: a file with no translatable
        // spans (comments-only, all-placeable messages) should produce
        // byte-identical output.
        return Ok(src.to_string());
    }
    let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
    let translations = translator.translate_batch(&texts, source_lang, target_lang).await?;
    splice_translations(src, &spans, &translations, path)
}

#[derive(Debug, Default)]
pub struct Summary {
    pub new: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub orphaned: usize,
}

/// Translate a file incrementally: keep target messages whose source hasn't
/// changed since the last run (per the side-car), only call the translator
/// for new + changed messages. Returns the merged output, a summary, and the
/// updated side-car (the caller can persist it).
///
/// When `force` is true, every message is treated as NEW and the side-car
/// from a previous run is ignored.
pub async fn translate_file_incremental<T: Translator + ?Sized>(
    src: &str,
    prev_target: Option<&str>,
    prev_sidecar: &Sidecar,
    source_lang: &str,
    target_lang: &str,
    translator: &T,
    force: bool,
) -> Result<(String, Summary, Sidecar), AppError> {
    let source_spans = walk_source(src, None)?;

    // Stable per-ID summary of source/target text for hashing and diffing.
    let source_text_per_id: BTreeMap<String, String> = group_spans_by_id(&source_spans);
    let target_text_per_id: BTreeMap<String, String> = match prev_target {
        Some(t) => {
            let target_spans = walk_source(t, None)?;
            group_spans_by_id(&target_spans)
        }
        None => BTreeMap::new(),
    };

    let verdicts: BTreeMap<String, Verdict> = if force {
        source_text_per_id
            .keys()
            .map(|k| (k.clone(), Verdict::New))
            .collect()
    } else {
        classify(&source_text_per_id, &target_text_per_id, prev_sidecar)
    };

    // Filter source spans to those belonging to messages we need to translate.
    let spans_to_translate: Vec<&TranslatableSpan> = source_spans
        .iter()
        .filter(|s| {
            matches!(
                verdicts.get(&s.entry_id),
                Some(Verdict::New) | Some(Verdict::Changed)
            )
        })
        .collect();

    let translations: Vec<String> = if spans_to_translate.is_empty() {
        Vec::new()
    } else {
        let texts: Vec<&str> = spans_to_translate.iter().map(|s| s.text.as_str()).collect();
        translator
            .translate_batch(&texts, source_lang, target_lang)
            .await?
    };

    // Build a (entry_id, span_index) → translation map for the spans we just translated.
    let mut translation_map: BTreeMap<(String, usize), String> = BTreeMap::new();
    for (span, tr) in spans_to_translate.iter().zip(translations.iter()) {
        translation_map.insert((span.entry_id.clone(), span.span_index), tr.clone());
    }

    // For Unchanged messages we copy translations from the previous target's spans.
    let prev_target_spans: Vec<TranslatableSpan> = match prev_target {
        Some(t) => walk_source(t, None)?,
        None => Vec::new(),
    };
    let mut prev_target_index: BTreeMap<(String, usize), String> = BTreeMap::new();
    for s in &prev_target_spans {
        prev_target_index.insert((s.entry_id.clone(), s.span_index), s.text.clone());
    }

    // Build the final ordered list of translations matching source_spans.
    let final_translations: Vec<String> = source_spans
        .iter()
        .map(|s| {
            let key = (s.entry_id.clone(), s.span_index);
            match verdicts.get(&s.entry_id) {
                Some(Verdict::New) | Some(Verdict::Changed) => translation_map
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| s.text.clone()),
                Some(Verdict::Unchanged) => prev_target_index
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| s.text.clone()),
                // Orphaned or absent — pass source text through unchanged.
                _ => s.text.clone(),
            }
        })
        .collect();

    let merged = splice_translations(src, &source_spans, &final_translations, None)?;

    // Compute summary counts per message (verdicts are per-message, not per-span).
    let mut summary = Summary::default();
    for v in verdicts.values() {
        match v {
            Verdict::New => summary.new += 1,
            Verdict::Changed => summary.changed += 1,
            Verdict::Unchanged => summary.unchanged += 1,
            Verdict::Orphaned => summary.orphaned += 1,
        }
    }

    // Build new sidecar: start with previous entries, update for everything
    // we just translated.
    let mut new_sidecar = clone_sidecar(prev_sidecar);
    let now = chrono::Utc::now().to_rfc3339();
    for (id, verdict) in &verdicts {
        if matches!(verdict, Verdict::New | Verdict::Changed) {
            if let Some(text) = source_text_per_id.get(id) {
                new_sidecar.entries.insert(
                    id.clone(),
                    SidecarEntry {
                        src_hash: hash_text(text),
                        translated_at: now.clone(),
                        backend: translator.name().to_string(),
                    },
                );
            }
        }
    }

    Ok((merged, summary, new_sidecar))
}

/// Concatenate every translatable span belonging to one entry_id into a
/// single representative string. Used to feed the diff classifier and to
/// produce a stable per-message hash for the side-car.
fn group_spans_by_id(spans: &[TranslatableSpan]) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for s in spans {
        let entry = out.entry(s.entry_id.clone()).or_default();
        if !entry.is_empty() {
            entry.push('\u{1f}'); // ASCII unit-separator between spans
        }
        if let Some(attr) = &s.attribute {
            entry.push_str(attr);
            entry.push(':');
        }
        entry.push_str(&s.text);
    }
    out
}

fn clone_sidecar(prev: &Sidecar) -> Sidecar {
    let mut new = Sidecar::default();
    for (k, v) in &prev.entries {
        new.entries.insert(
            k.clone(),
            SidecarEntry {
                src_hash: v.src_hash.clone(),
                translated_at: v.translated_at.clone(),
                backend: v.backend.clone(),
            },
        );
    }
    new
}
