use ftl2lang::sidecar::{hash_text, Sidecar, SidecarEntry};

#[test]
fn hash_is_deterministic_and_prefixed() {
    let a = hash_text("hello");
    let b = hash_text("hello");
    assert_eq!(a, b);
    assert!(a.starts_with("sha256:"));
    assert_ne!(a, hash_text("Hello"));
}

#[test]
fn round_trips_through_json() {
    let mut s = Sidecar::default();
    s.entries.insert(
        "greeting".into(),
        SidecarEntry {
            src_hash: hash_text("hi"),
            translated_at: "2026-05-13T00:00:00Z".into(),
            backend: "mock".into(),
        },
    );
    let json = s.to_json().unwrap();
    let parsed = Sidecar::from_json(&json).unwrap();
    assert_eq!(parsed.entries.get("greeting").unwrap().backend, "mock");
}

#[test]
fn load_missing_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nope.json");
    let s = Sidecar::load(&path).unwrap();
    assert!(s.entries.is_empty());
}

#[test]
fn save_and_load_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("de.ftl.ftl2lang.json");
    let mut s = Sidecar::default();
    s.entries.insert(
        "k".into(),
        SidecarEntry {
            src_hash: hash_text("v"),
            translated_at: "now".into(),
            backend: "deepl".into(),
        },
    );
    s.save(&path).unwrap();
    let loaded = Sidecar::load(&path).unwrap();
    assert_eq!(loaded.entries.get("k").unwrap().src_hash, hash_text("v"));
}
