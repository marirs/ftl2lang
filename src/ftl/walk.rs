use crate::error::AppError;
use crate::ftl::split_whitespace;
use fluent_syntax::ast;
use fluent_syntax::parser;
use std::path::{Path, PathBuf};

/// A single contiguous run of translatable text extracted from an FTL pattern.
///
/// When a pattern contains placeables (variable references, term references, or
/// select expressions), the surrounding text is split across multiple spans.
/// Whitespace at the edges of each raw `TextElement` is stripped and recorded
/// separately so that a round-trip can reconstruct the original spacing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatableSpan {
    /// The identifier of the message or term that owns this span.
    pub entry_id: String,
    /// `None` for the message/term value itself; `Some(name)` for an attribute.
    pub attribute: Option<String>,
    /// Sequential index of this span within the entry. The counter is shared
    /// across the entry's value and all of its attributes (in declaration
    /// order), and is NOT reset when entering attributes. The pair
    /// `(entry_id, span_index)` uniquely identifies a span — `attribute` is
    /// informational and need not be part of any splice-back key.
    pub span_index: usize,
    /// Trimmed translatable text — never empty, never whitespace-only.
    pub text: String,
    /// Whitespace stripped from the front of the original `TextElement`.
    pub leading_ws: String,
    /// Whitespace stripped from the back of the original `TextElement`.
    pub trailing_ws: String,
}

/// Parse `src` as FTL and return every translatable text span.
///
/// A span is created for each non-empty, non-whitespace-only `TextElement`
/// found in message/term values and their attributes. Select-expression
/// variants are walked recursively so their literal text arms are included.
/// Pure placeables (variable/term references, function calls) produce no spans.
///
/// `path` is reported in parse errors so users can see which file failed.
/// Pass `None` when the source did not come from a file on disk.
pub fn walk_source(src: &str, path: Option<&Path>) -> Result<Vec<TranslatableSpan>, AppError> {
    // On parse failure fluent-syntax returns a partially-recovered Resource
    // alongside the error list. We deliberately discard that partial AST and
    // refuse to walk anything: a translation tool must not silently translate
    // half of a malformed file.
    let resource = parser::parse(src).map_err(|(_, errs)| AppError::FtlParse {
        path: path.map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("<input>")),
        message: format!("{} error(s); first: {:?}", errs.len(), errs.first()),
    })?;

    let mut spans = Vec::new();
    for entry in &resource.body {
        match entry {
            ast::Entry::Message(msg) => {
                let id = msg.id.name.to_string();
                let mut idx = 0_usize;
                if let Some(pattern) = &msg.value {
                    walk_pattern(&id, None, pattern, &mut idx, &mut spans);
                }
                for attr in &msg.attributes {
                    walk_pattern(
                        &id,
                        Some(attr.id.name.to_string()),
                        &attr.value,
                        &mut idx,
                        &mut spans,
                    );
                }
            }
            ast::Entry::Term(term) => {
                // v3 differs: terms are also walked so term values can be
                // translated. In Fluent, terms are not user-visible on their
                // own, but they carry brand names and other translatable
                // strings that downstream translation engines need.
                let id = format!("-{}", term.id.name);
                let mut idx = 0_usize;
                walk_pattern(&id, None, &term.value, &mut idx, &mut spans);
                for attr in &term.attributes {
                    walk_pattern(
                        &id,
                        Some(attr.id.name.to_string()),
                        &attr.value,
                        &mut idx,
                        &mut spans,
                    );
                }
            }
            // Comments and junk carry no translatable text.
            _ => {}
        }
    }
    Ok(spans)
}

/// Walk all `PatternElement`s in `pattern`, appending spans to `out`.
///
/// `idx` is a shared counter so span indices are sequential across both the
/// entry value and its attributes within a single entry.
fn walk_pattern(
    entry_id: &str,
    attribute: Option<String>,
    pattern: &ast::Pattern<&str>,
    idx: &mut usize,
    out: &mut Vec<TranslatableSpan>,
) {
    for element in &pattern.elements {
        match element {
            ast::PatternElement::TextElement { value } => {
                let (leading, core, trailing) = split_whitespace(value);
                // Whitespace-only text elements (e.g. the space between two
                // adjacent placeables) carry no translatable content.
                if core.is_empty() {
                    continue;
                }
                out.push(TranslatableSpan {
                    entry_id: entry_id.to_string(),
                    attribute: attribute.clone(),
                    span_index: *idx,
                    text: core.to_string(),
                    leading_ws: leading.to_string(),
                    trailing_ws: trailing.to_string(),
                });
                *idx += 1;
            }
            ast::PatternElement::Placeable { expression } => {
                walk_expression(entry_id, attribute.clone(), expression, idx, out);
            }
        }
    }
}

/// Recurse into select expressions to extract text from each variant arm.
///
/// Non-select expressions (variable references, term references, function
/// calls, literals) are opaque placeables and produce no translatable spans.
fn walk_expression(
    entry_id: &str,
    attribute: Option<String>,
    expression: &ast::Expression<&str>,
    idx: &mut usize,
    out: &mut Vec<TranslatableSpan>,
) {
    if let ast::Expression::Select { variants, .. } = expression {
        for variant in variants {
            walk_pattern(entry_id, attribute.clone(), &variant.value, idx, out);
        }
    }
}

