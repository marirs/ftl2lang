use ftl2lang::ftl::walk::walk_source;

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{}", name)).unwrap()
}

#[test]
fn simple_file_extracts_each_message_value() {
    let src = read_fixture("simple.ftl");
    let spans = walk_source(&src, None).unwrap();

    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].entry_id, "hello");
    assert_eq!(spans[0].attribute, None);
    assert_eq!(spans[0].text, "Hello, world!");
    assert_eq!(spans[1].entry_id, "goodbye");
    assert_eq!(spans[1].text, "Goodbye.");
}

#[test]
fn placeables_are_excluded_from_spans() {
    let src = read_fixture("with_placeables.ftl");
    let spans = walk_source(&src, None).unwrap();

    // Trimmed cores: "Hello," / "! You have new mail." / "Welcome to" / "."
    // (the raw TextElements include a trailing space before each placeable,
    // which the walker strips into `trailing_ws`).
    assert_eq!(spans.len(), 4);
    assert_eq!(spans[0].text, "Hello,");
    assert_eq!(spans[0].leading_ws, "");
    assert_eq!(spans[0].trailing_ws, " ");
    assert_eq!(spans[1].text, "! You have new mail.");
    assert_eq!(spans[2].text, "Welcome to");
    assert_eq!(spans[2].trailing_ws, " ");
    assert_eq!(spans[3].text, ".");
}

#[test]
fn selector_variants_each_contribute_spans() {
    let src = read_fixture("with_selectors.ftl");
    let spans = walk_source(&src, None).unwrap();

    let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
    // The two variant arms and the outer text before/after the selector.
    assert!(texts.contains(&"You have"), "spans: {:?}", texts);
    assert!(texts.contains(&"one new message"), "spans: {:?}", texts);
    assert!(texts.contains(&"new messages"), "spans: {:?}", texts);
    assert!(texts.contains(&"."), "spans: {:?}", texts);
}

#[test]
fn attributes_produce_spans_with_attribute_set() {
    let src = read_fixture("with_attributes.ftl");
    let spans = walk_source(&src, None).unwrap();

    assert!(spans.iter().any(|s| s.entry_id == "login-button" && s.attribute.is_none() && s.text == "Log in"));
    assert!(spans.iter().any(|s| s.attribute.as_deref() == Some("title") && s.text == "Click to log in"));
    assert!(spans.iter().any(|s| s.attribute.as_deref() == Some("aria-label") && s.text == "Log in to your account"));
}

#[test]
fn whitespace_only_text_elements_are_skipped() {
    let src = "key =\n    { $a } { $b }\n";
    let spans = walk_source(src, None).unwrap();
    // The " " between placeables is whitespace-only, so 0 translatable spans.
    assert_eq!(spans.len(), 0);
}
