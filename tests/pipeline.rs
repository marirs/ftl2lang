use ftl2lang::pipeline::translate_file_content;
use ftl2lang::translator::mock::MockTranslator;

#[tokio::test]
async fn snapshot_simple() {
    let src = std::fs::read_to_string("tests/fixtures/simple.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator).await.unwrap();
    insta::assert_snapshot!("simple", out);
}

#[tokio::test]
async fn snapshot_with_placeables() {
    let src = std::fs::read_to_string("tests/fixtures/with_placeables.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator).await.unwrap();
    insta::assert_snapshot!("with_placeables", out);
}

#[tokio::test]
async fn snapshot_with_selectors() {
    let src = std::fs::read_to_string("tests/fixtures/with_selectors.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator).await.unwrap();
    insta::assert_snapshot!("with_selectors", out);
}

#[tokio::test]
async fn snapshot_with_attributes() {
    let src = std::fs::read_to_string("tests/fixtures/with_attributes.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator).await.unwrap();
    insta::assert_snapshot!("with_attributes", out);
}
