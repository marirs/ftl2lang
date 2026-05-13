use ftl2lang::pipeline::{translate_file_content, translate_file_incremental};
use ftl2lang::sidecar::{hash_text, Sidecar, SidecarEntry};
use ftl2lang::translator::mock::MockTranslator;

#[tokio::test]
async fn snapshot_simple() {
    let src = std::fs::read_to_string("tests/fixtures/simple.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator, None).await.unwrap();
    insta::assert_snapshot!("simple", out);
}

#[tokio::test]
async fn snapshot_with_placeables() {
    let src = std::fs::read_to_string("tests/fixtures/with_placeables.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator, None).await.unwrap();
    insta::assert_snapshot!("with_placeables", out);
}

#[tokio::test]
async fn snapshot_with_selectors() {
    let src = std::fs::read_to_string("tests/fixtures/with_selectors.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator, None).await.unwrap();
    insta::assert_snapshot!("with_selectors", out);
}

#[tokio::test]
async fn snapshot_with_attributes() {
    let src = std::fs::read_to_string("tests/fixtures/with_attributes.ftl").unwrap();
    let translator = MockTranslator::new();
    let out = translate_file_content(&src, "en", "de", &translator, None).await.unwrap();
    insta::assert_snapshot!("with_attributes", out);
}

#[tokio::test]
async fn first_run_translates_all_messages() {
    let src = "a = Hello\nb = World\n";
    let translator = MockTranslator::new();
    let (out, summary, sidecar) =
        translate_file_incremental(src, None, &Sidecar::default(), "en", "de", &translator, false)
            .await
            .unwrap();
    assert!(out.contains("a = HELLO"));
    assert!(out.contains("b = WORLD"));
    assert_eq!(summary.new, 2);
    assert_eq!(summary.unchanged, 0);
    assert_eq!(summary.changed, 0);
    assert!(sidecar.entries.contains_key("a"));
    assert!(sidecar.entries.contains_key("b"));
}

#[tokio::test]
async fn unchanged_messages_in_target_are_preserved() {
    let src = "a = Hello\nb = World\n";
    let prev_target = "a = Hallo\nb = Welt\n";
    let mut prev_sc = Sidecar::default();
    prev_sc.entries.insert(
        "a".into(),
        SidecarEntry {
            src_hash: hash_text("Hello"),
            translated_at: "x".into(),
            backend: "mock".into(),
        },
    );
    prev_sc.entries.insert(
        "b".into(),
        SidecarEntry {
            src_hash: hash_text("World"),
            translated_at: "x".into(),
            backend: "mock".into(),
        },
    );
    let translator = MockTranslator::new();
    let (out, summary, _sc) = translate_file_incremental(
        src,
        Some(prev_target),
        &prev_sc,
        "en",
        "de",
        &translator,
        false,
    )
    .await
    .unwrap();
    assert!(out.contains("Hallo"));
    assert!(out.contains("Welt"));
    assert_eq!(summary.unchanged, 2);
    assert_eq!(summary.new, 0);
}

#[tokio::test]
async fn changed_source_triggers_retranslation() {
    let src = "a = Hi\n";
    let prev_target = "a = Hallo\n";
    let mut prev_sc = Sidecar::default();
    prev_sc.entries.insert(
        "a".into(),
        SidecarEntry {
            src_hash: hash_text("Hello"),
            translated_at: "x".into(),
            backend: "mock".into(),
        },
    );
    let translator = MockTranslator::new();
    let (out, summary, _sc) =
        translate_file_incremental(src, Some(prev_target), &prev_sc, "en", "de", &translator, false)
            .await
            .unwrap();
    assert!(out.contains("a = HI"));
    assert_eq!(summary.changed, 1);
    assert_eq!(summary.unchanged, 0);
}

#[tokio::test]
async fn force_retranslates_everything() {
    let src = "a = Hello\n";
    let prev_target = "a = Hallo\n";
    let mut prev_sc = Sidecar::default();
    prev_sc.entries.insert(
        "a".into(),
        SidecarEntry {
            src_hash: hash_text("Hello"),
            translated_at: "x".into(),
            backend: "mock".into(),
        },
    );
    let translator = MockTranslator::new();
    let (out, summary, _sc) =
        translate_file_incremental(src, Some(prev_target), &prev_sc, "en", "de", &translator, true)
            .await
            .unwrap();
    assert!(out.contains("a = HELLO"));
    assert_eq!(summary.new + summary.changed, 1);
}
