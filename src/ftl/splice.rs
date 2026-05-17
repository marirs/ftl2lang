use crate::error::AppError;
use crate::ftl::split_whitespace;
use crate::ftl::walk::TranslatableSpan;
use fluent_syntax::ast;
use fluent_syntax::parser;
use fluent_syntax::serializer;
use std::path::{Path, PathBuf};

/// Replace each `TranslatableSpan`'s text in the parsed AST with the corresponding
/// entry in `translations`, then serialize. Whitespace from `leading_ws` and
/// `trailing_ws` is restored around each translated segment.
///
/// `path` is reported in parse errors so users can see which file failed; pass
/// `None` when the source did not come from a file on disk.
///
/// Note: this function leaks memory proportional to the number of translated
/// spans (one `Box::leak` per spliced `TextElement`). It is designed for
/// single-invocation CLI use, not for a long-running library context.
pub fn splice_translations(
    src: &str,
    spans: &[TranslatableSpan],
    translations: &[String],
    path: Option<&Path>,
) -> Result<String, AppError> {
    if spans.len() != translations.len() {
        return Err(AppError::Other(format!(
            "splice: span/translation count mismatch ({} vs {})",
            spans.len(),
            translations.len()
        )));
    }

    let mut resource = parser::parse(src).map_err(|(_, errs)| AppError::FtlParse {
        path: path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("<input>")),
        message: format!("{} error(s); first: {:?}", errs.len(), errs.first()),
    })?;

    // Walk the AST in the SAME order as walk_source. The shared `idx` counter
    // is per-entry: it increments through value spans, then attribute spans,
    // and matches the span_index field on each TranslatableSpan exactly.
    //
    // Per the walker's contract, (entry_id, span_index) is the unique match
    // key. `attribute` is informational; we still pass it through so the
    // debug-mode consistency check can catch drift.
    let mut iter = spans.iter().zip(translations.iter()).peekable();

    for entry in resource.body.iter_mut() {
        match entry {
            ast::Entry::Message(msg) => {
                let id = msg.id.name;
                let mut idx = 0usize;
                if let Some(pattern) = msg.value.as_mut() {
                    splice_pattern(id, None, pattern, &mut idx, &mut iter);
                }
                for attr in msg.attributes.iter_mut() {
                    let attr_name = attr.id.name;
                    splice_pattern(id, Some(attr_name), &mut attr.value, &mut idx, &mut iter);
                }
            }
            ast::Entry::Term(term) => {
                let id_owned = format!("-{}", term.id.name);
                let id: &str = &id_owned;
                let mut idx = 0usize;
                splice_pattern(id, None, &mut term.value, &mut idx, &mut iter);
                for attr in term.attributes.iter_mut() {
                    let attr_name = attr.id.name;
                    splice_pattern(id, Some(attr_name), &mut attr.value, &mut idx, &mut iter);
                }
            }
            _ => {}
        }
    }

    Ok(serializer::serialize(&resource))
}

type SpliceIter<'a> = std::iter::Peekable<
    std::iter::Zip<std::slice::Iter<'a, TranslatableSpan>, std::slice::Iter<'a, String>>,
>;

fn splice_pattern<'a>(
    entry_id: &str,
    attribute: Option<&str>,
    pattern: &mut ast::Pattern<&'a str>,
    idx: &mut usize,
    iter: &mut SpliceIter<'a>,
) {
    for element in pattern.elements.iter_mut() {
        match element {
            ast::PatternElement::TextElement { value } => {
                let (leading, core, trailing) = split_whitespace(value);
                if core.is_empty() {
                    continue;
                }
                // The walker guarantees that (entry_id, span_index) uniquely
                // identifies a span, so we only match on those two fields.
                // The attribute is checked separately as a debug_assert below
                // so a desync between walker and splicer fails loudly in tests
                // instead of silently producing partial output.
                let matches = iter
                    .peek()
                    .map(|(s, _)| s.entry_id == entry_id && s.span_index == *idx)
                    .unwrap_or(false);
                if matches {
                    let (span, translation) = iter.next().unwrap();
                    debug_assert_eq!(
                        span.attribute.as_deref(),
                        attribute,
                        "splice: attribute disagrees with walker for entry_id={:?} idx={}",
                        entry_id,
                        idx,
                    );
                    let replaced = format!("{}{}{}", leading, translation, trailing);
                    // Deliberate leak: fluent-syntax holds &'a str borrowed
                    // from `src`. To insert a replacement we need a &'a str
                    // with the same lifetime. Box::leak is the simplest
                    // correct approach for a one-shot CLI; memory is
                    // reclaimed on process exit. See the function-level
                    // doc-comment for the broader caveat.
                    let leaked: &'a str = Box::leak(replaced.into_boxed_str());
                    *value = leaked;
                    *idx += 1;
                } else {
                    // The walker emitted a span for this TextElement (core is
                    // non-empty) but the iterator does not have a matching
                    // entry. This means walker and splicer have diverged —
                    // panic in debug, silently leave the original text in
                    // release rather than splicing the wrong translation.
                    debug_assert!(
                        false,
                        "splice: no matching span for entry_id={:?} attribute={:?} idx={} \
                         — walker/splicer out of sync",
                        entry_id, attribute, idx
                    );
                }
            }
            ast::PatternElement::Placeable { expression } => {
                splice_expression(entry_id, attribute, expression, idx, iter);
            }
        }
    }
}

fn splice_expression<'a>(
    entry_id: &str,
    attribute: Option<&str>,
    expression: &mut ast::Expression<&'a str>,
    idx: &mut usize,
    iter: &mut SpliceIter<'a>,
) {
    if let ast::Expression::Select { variants, .. } = expression {
        for variant in variants.iter_mut() {
            splice_pattern(entry_id, attribute, &mut variant.value, idx, iter);
        }
    }
}
