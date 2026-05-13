pub mod splice;
pub mod walk;

/// Split a raw `TextElement` value into `(leading_whitespace, core, trailing_whitespace)`.
///
/// `core` is the trimmed text and may be empty if the input was entirely
/// whitespace (e.g. a single space between two adjacent placeables).
///
/// This helper is shared by [`walk`] and [`splice`] so that the two stages
/// strip and restore whitespace symmetrically — round-trip fidelity depends
/// on them agreeing exactly.
pub(crate) fn split_whitespace(s: &str) -> (&str, &str, &str) {
    let trimmed_start = s.trim_start();
    let leading_len = s.len() - trimmed_start.len();
    let leading = &s[..leading_len];
    let trimmed = trimmed_start.trim_end();
    let trailing = &trimmed_start[trimmed.len()..];
    (leading, trimmed, trailing)
}
