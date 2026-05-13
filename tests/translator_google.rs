use ftl2lang::translator::google::GoogleTranslator;
use ftl2lang::translator::Translator;

#[test]
fn supports_many_langs() {
    let t = GoogleTranslator::new("KEY".into(), "proj".into());
    assert!(t.supports("de"));
    assert!(t.supports("ta"));
    assert!(t.supports("hi"));
    assert!(t.supports("th"));
}

#[tokio::test]
#[ignore]
async fn live_translates_with_real_key() {
    let key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY required");
    let project = std::env::var("GOOGLE_PROJECT_ID").expect("GOOGLE_PROJECT_ID required");
    let t = GoogleTranslator::new(key, project);
    let out = t.translate_batch(&["Hello"], "en", "ta").await.unwrap();
    assert_eq!(out.len(), 1);
    assert!(!out[0].is_empty());
}
