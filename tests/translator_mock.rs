use ftl2lang::translator::mock::MockTranslator;
use ftl2lang::translator::Translator;

#[tokio::test]
async fn mock_uppercases_inputs() {
    let m = MockTranslator::new();
    let out = m.translate_batch(&["hello", "world"], "en", "de").await.unwrap();
    assert_eq!(out, vec!["HELLO".to_string(), "WORLD".to_string()]);
}

#[tokio::test]
async fn mock_supports_any_target() {
    let m = MockTranslator::new();
    assert!(m.supports("de"));
    assert!(m.supports("ta"));
}

#[tokio::test]
async fn mock_name_is_mock() {
    assert_eq!(MockTranslator::new().name(), "mock");
}
