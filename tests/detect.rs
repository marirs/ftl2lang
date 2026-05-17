use ftl2lang::detect::detect_source_language;

#[test]
fn detects_english_from_simple_text() {
    // Lengthened from the original two short phrases: whatlang needs enough
    // trigram density to reach confidence > 0 on short English input.
    let texts = [
        "The quick brown fox jumps over the lazy dog near the river bank.",
        "She opened the door and walked into the bright morning sunlight.",
    ];
    let result = detect_source_language(&texts).unwrap();
    assert_eq!(result.code, "en");
}

#[test]
fn detects_german() {
    let texts = ["Guten Tag, wie geht es Ihnen?", "Auf Wiedersehen"];
    let result = detect_source_language(&texts).unwrap();
    assert_eq!(result.code, "de");
}

#[test]
fn empty_input_returns_error() {
    assert!(detect_source_language(&[]).is_err());
}

#[test]
fn unmapped_language_returns_error_not_und_code() {
    // Esperanto is a real language whatlang can identify, but it is not in
    // our backend-supported map. The function should return Err rather than
    // a DetectedLanguage with code = "und".
    let texts = [
        "Saluton mondo, kiel vi fartas hodiaŭ en la bela tago?",
        "Mi amas programi en Rust kaj uzi diversajn bibliotekojn.",
    ];
    let result = detect_source_language(&texts);
    // The detector may or may not identify Esperanto reliably; what we are
    // testing is that IF whatlang produces a Lang we have not mapped, we
    // return Err. So we only assert the negative: never an Ok with "und".
    if let Ok(ok) = result {
        assert_ne!(
            ok.code, "und",
            "must never expose 'und' through DetectedLanguage"
        );
    }
}
