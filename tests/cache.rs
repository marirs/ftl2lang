use ftl2lang::cache::TranslationCache;

#[test]
fn miss_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.json");
    let cache = TranslationCache::load(&path).unwrap();
    assert!(cache.get("Hello", "en", "de", "deepl").is_none());
}

#[test]
fn put_then_get_hits() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.json");
    let mut cache = TranslationCache::load(&path).unwrap();
    cache.put("Hello", "en", "de", "deepl", "Hallo");
    assert_eq!(cache.get("Hello", "en", "de", "deepl").as_deref(), Some("Hallo"));
}

#[test]
fn keys_differ_by_backend() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.json");
    let mut cache = TranslationCache::load(&path).unwrap();
    cache.put("Hello", "en", "de", "deepl", "Hallo (deepl)");
    cache.put("Hello", "en", "de", "google", "Hallo (google)");
    assert_eq!(cache.get("Hello", "en", "de", "deepl").as_deref(), Some("Hallo (deepl)"));
    assert_eq!(cache.get("Hello", "en", "de", "google").as_deref(), Some("Hallo (google)"));
}

#[test]
fn save_and_load_persists() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.json");
    {
        let mut cache = TranslationCache::load(&path).unwrap();
        cache.put("Hello", "en", "de", "deepl", "Hallo");
        cache.save(&path).unwrap();
    }
    let cache = TranslationCache::load(&path).unwrap();
    assert_eq!(cache.get("Hello", "en", "de", "deepl").as_deref(), Some("Hallo"));
}
