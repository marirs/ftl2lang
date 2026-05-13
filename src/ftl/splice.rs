use crate::error::AppError;
use crate::ftl::walk::TranslatableSpan;
use fluent_syntax::ast;
use fluent_syntax::parser;
use fluent_syntax::serializer;
use std::path::PathBuf;

/// Replace each `TranslatableSpan`'s text in the parsed AST with the corresponding
/// entry in `translations`, then serialize. Whitespace from `leading_ws` and
/// `trailing_ws` is restored around each translated segment.
pub fn splice_translations(
    src: &str,
    spans: &[TranslatableSpan],
    translations: &[String],
) -> Result<String, AppError> {
    if spans.len() != translations.len() {
        return Err(AppError::Other(format!(
            "splice: span/translation count mismatch ({} vs {})",
            spans.len(),
            translations.len()
        )));
    }

    let mut resource = parser::parse(src).map_err(|(_, errs)| AppError::FtlParse {
        path: PathBuf::from("<input>"),
        message: format!("{} error(s); first: {:?}", errs.len(), errs.first()),
    })?;

    // Walk the AST in the SAME order as walk_source. The shared `idx` counter
    // is per-entry: it increments through value spans, then attribute spans,
    // and matches the span_index field on each TranslatableSpan exactly.
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
                let matches = iter
                    .peek()
                    .map(|(s, _)| {
                        s.entry_id == entry_id
                            && s.attribute.as_deref() == attribute
                            && s.span_index == *idx
                    })
                    .unwrap_or(false);
                if matches {
                    let (_, translation) = iter.next().unwrap();
                    let replaced = format!("{}{}{}", leading, translation, trailing);
                    // SAFETY: fluent-syntax holds &'a str borrowed from `src`.
                    // To insert a replacement we must produce a &'a str with the
                    // same lifetime. Box::leak is the simplest correct approach
                    // for a CLI that runs once and exits; memory is reclaimed
                    // on process exit. This is a deliberate, bounded use.
                    let leaked: &'a str = Box::leak(replaced.into_boxed_str());
                    *value = leaked;
                    *idx += 1;
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

fn split_whitespace(s: &str) -> (&str, &str, &str) {
    let trimmed_start = s.trim_start();
    let leading_len = s.len() - trimmed_start.len();
    let leading = &s[..leading_len];
    let trimmed = trimmed_start.trim_end();
    let trailing = &trimmed_start[trimmed.len()..];
    (leading, trimmed, trailing)
}
