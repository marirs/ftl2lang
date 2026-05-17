use ftl2lang::diff::{classify, Verdict};
use ftl2lang::sidecar::{hash_text, Sidecar, SidecarEntry};
use std::collections::BTreeMap;

fn ids(items: &[(&str, Verdict)]) -> BTreeMap<String, Verdict> {
    items
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[test]
fn new_id_in_source_only_is_new() {
    let source: BTreeMap<String, String> = [("greeting".to_string(), "Hello".to_string())]
        .into_iter()
        .collect();
    let target: BTreeMap<String, String> = BTreeMap::new();
    let sidecar = Sidecar::default();
    let verdicts = classify(&source, &target, &sidecar);
    assert_eq!(verdicts, ids(&[("greeting", Verdict::New)]));
}

#[test]
fn id_in_both_with_matching_sidecar_is_unchanged() {
    let source: BTreeMap<String, String> =
        [("greeting".into(), "Hello".into())].into_iter().collect();
    let target: BTreeMap<String, String> =
        [("greeting".into(), "Hallo".into())].into_iter().collect();
    let mut sc = Sidecar::default();
    sc.entries.insert(
        "greeting".into(),
        SidecarEntry {
            src_hash: hash_text("Hello"),
            translated_at: "x".into(),
            backend: "mock".into(),
        },
    );
    let verdicts = classify(&source, &target, &sc);
    assert_eq!(verdicts.get("greeting"), Some(&Verdict::Unchanged));
}

#[test]
fn id_in_both_with_changed_source_is_changed() {
    let source: BTreeMap<String, String> = [("greeting".into(), "Hi there".into())]
        .into_iter()
        .collect();
    let target: BTreeMap<String, String> =
        [("greeting".into(), "Hallo".into())].into_iter().collect();
    let mut sc = Sidecar::default();
    sc.entries.insert(
        "greeting".into(),
        SidecarEntry {
            src_hash: hash_text("Hello"),
            translated_at: "x".into(),
            backend: "mock".into(),
        },
    );
    let verdicts = classify(&source, &target, &sc);
    assert_eq!(verdicts.get("greeting"), Some(&Verdict::Changed));
}

#[test]
fn id_in_both_no_sidecar_is_unchanged() {
    let source: BTreeMap<String, String> =
        [("greeting".into(), "Hello".into())].into_iter().collect();
    let target: BTreeMap<String, String> =
        [("greeting".into(), "Hallo".into())].into_iter().collect();
    let sc = Sidecar::default();
    let verdicts = classify(&source, &target, &sc);
    assert_eq!(verdicts.get("greeting"), Some(&Verdict::Unchanged));
}

#[test]
fn id_only_in_target_is_orphaned() {
    let source: BTreeMap<String, String> = BTreeMap::new();
    let target: BTreeMap<String, String> = [("old".into(), "Alt".into())].into_iter().collect();
    let sc = Sidecar::default();
    let verdicts = classify(&source, &target, &sc);
    assert_eq!(verdicts.get("old"), Some(&Verdict::Orphaned));
}
