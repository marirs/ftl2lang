use ftl2lang::ftl::splice::splice_translations;
use ftl2lang::ftl::walk::walk_source;

#[test]
fn round_trip_with_identity_translation_is_byte_equivalent() {
    let src = std::fs::read_to_string("tests/fixtures/with_placeables.ftl").unwrap();
    let spans = walk_source(&src, None).unwrap();
    let translations: Vec<String> = spans.iter().map(|s| s.text.clone()).collect();
    let out = splice_translations(&src, &spans, &translations).unwrap();
    assert_eq!(out.trim_end(), src.trim_end());
}

#[test]
fn translated_text_appears_in_output_with_placeables_preserved() {
    let src = std::fs::read_to_string("tests/fixtures/with_placeables.ftl").unwrap();
    let spans = walk_source(&src, None).unwrap();
    let translations: Vec<String> = spans.iter().map(|s| s.text.to_uppercase()).collect();
    let out = splice_translations(&src, &spans, &translations).unwrap();

    assert!(out.contains("HELLO,"));
    assert!(out.contains("{ $name }"), "placeable preserved");
    assert!(out.contains("WELCOME TO"));
    assert!(out.contains("{ -brand-name }"), "term reference preserved");
}

#[test]
fn selectors_have_each_variant_translated() {
    let src = std::fs::read_to_string("tests/fixtures/with_selectors.ftl").unwrap();
    let spans = walk_source(&src, None).unwrap();
    let translations: Vec<String> = spans.iter().map(|s| s.text.to_uppercase()).collect();
    let out = splice_translations(&src, &spans, &translations).unwrap();

    assert!(out.contains("ONE NEW MESSAGE"));
    assert!(out.contains("NEW MESSAGES"));
    assert!(out.contains("[one]"));
    assert!(out.contains("*[other]"));
}

#[test]
fn attributes_translate_independently() {
    let src = std::fs::read_to_string("tests/fixtures/with_attributes.ftl").unwrap();
    let spans = walk_source(&src, None).unwrap();
    let translations: Vec<String> = spans.iter().map(|s| s.text.to_uppercase()).collect();
    let out = splice_translations(&src, &spans, &translations).unwrap();

    assert!(out.contains("login-button = LOG IN"));
    assert!(out.contains(".title = CLICK TO LOG IN"));
    assert!(out.contains(".aria-label = LOG IN TO YOUR ACCOUNT"));
}

#[test]
fn count_mismatch_returns_error() {
    let src = "hello = Hello\n";
    let spans = walk_source(src, None).unwrap();
    let result = splice_translations(src, &spans, &[]);
    assert!(result.is_err());
}
