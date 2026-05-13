use ftl2lang::translator::gtranslate::GtranslateTranslator;
use ftl2lang::translator::Translator;

#[test]
fn name_and_supports_all_langs() {
    let t = GtranslateTranslator::new();
    assert_eq!(t.name(), "gtranslate");
    assert!(t.supports("de"));
    assert!(t.supports("ta"));
}

// Live API call — gated behind --ignored
#[tokio::test]
#[ignore]
async fn live_translates_simple_string() {
    let t = GtranslateTranslator::new();
    let out = t.translate_batch(&["Hello"], "en", "de", None).await.unwrap();
    assert_eq!(out.len(), 1);
    let lower = out[0].to_lowercase();
    assert!(lower.contains("hallo") || lower.contains("guten"), "got: {}", out[0]);
}
