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

#[test]
fn clear_missing_cache_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nope.json");
    assert_eq!(TranslationCache::clear(&path).unwrap(), None);
}

#[test]
fn clear_reports_entry_count_and_deletes_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.json");
    {
        let mut cache = TranslationCache::load(&path).unwrap();
        cache.put("Hello", "en", "de", "deepl", "Hallo");
        cache.put("Bye", "en", "de", "deepl", "Tschüss");
        cache.save(&path).unwrap();
    }
    assert_eq!(TranslationCache::clear(&path).unwrap(), Some(2));
    assert!(!path.exists(), "cache file should be deleted after clear");
}

#[test]
fn clear_is_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("cache.json");
    {
        let mut cache = TranslationCache::load(&path).unwrap();
        cache.put("Hi", "en", "de", "deepl", "Hallo");
        cache.save(&path).unwrap();
    }
    assert_eq!(TranslationCache::clear(&path).unwrap(), Some(1));
    // Second clear: file is already gone, so None — not an error.
    assert_eq!(TranslationCache::clear(&path).unwrap(), None);
}
