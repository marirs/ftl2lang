use ftl2lang::detect::detect_source_language;

#[test]
fn detects_english_from_simple_text() {
    // Lengthened from the original two short phrases: whatlang needs enough
    // trigram density to reach confidence > 0 on short English input.
    let texts = vec![
        "The quick brown fox jumps over the lazy dog near the river bank.".to_string(),
        "She opened the door and walked into the bright morning sunlight.".to_string(),
    ];
    let result = detect_source_language(&texts).unwrap();
    assert_eq!(result.code, "en");
}

#[test]
fn detects_german() {
    let texts = vec!["Guten Tag, wie geht es Ihnen?".to_string(), "Auf Wiedersehen".to_string()];
    let result = detect_source_language(&texts).unwrap();
    assert_eq!(result.code, "de");
}

#[test]
fn empty_input_returns_error() {
    assert!(detect_source_language(&[]).is_err());
}
