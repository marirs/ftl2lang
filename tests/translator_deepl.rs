use ftl2lang::translator::deepl::{DeeplTranslator, DEEPL_SUPPORTED_LANGS};
use ftl2lang::translator::Translator;

#[test]
fn supports_european_langs() {
    let t = DeeplTranslator::new("KEY".into(), None);
    assert!(t.supports("de"));
    assert!(t.supports("fr"));
    assert!(t.supports("ja"));
}

#[test]
fn does_not_support_tamil() {
    let t = DeeplTranslator::new("KEY".into(), None);
    assert!(!t.supports("ta"));
    assert!(!t.supports("hi"));
}

#[test]
fn supported_list_includes_known_langs() {
    assert!(DEEPL_SUPPORTED_LANGS.contains(&"de"));
    assert!(DEEPL_SUPPORTED_LANGS.contains(&"ja"));
    assert!(!DEEPL_SUPPORTED_LANGS.contains(&"ta"));
}

#[tokio::test]
#[ignore]
async fn live_translates_with_real_key() {
    let key = std::env::var("DEEPL_API_KEY").expect("DEEPL_API_KEY required");
    let t = DeeplTranslator::new(key, None);
    let out = t.translate_batch(&["Hello", "Goodbye"], "en", "de", None).await.unwrap();
    assert_eq!(out.len(), 2);
}
