use ftl2lang::fsutil::atomic_write;

#[test]
fn writes_new_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("out.txt");
    atomic_write(&path, "hello").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
}

#[test]
fn overwrites_existing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("out.txt");
    std::fs::write(&path, "old contents").unwrap();
    atomic_write(&path, "new contents").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "new contents");
}

#[test]
fn creates_missing_parent_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nested").join("deep").join("out.txt");
    atomic_write(&path, "data").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "data");
}

#[test]
fn leaves_no_temp_file_behind_on_success() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("out.txt");
    atomic_write(&path, "x").unwrap();

    // The directory should contain exactly the target file — no .<pid>.tmp
    // litter from the atomic-write dance.
    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(entries, vec![std::ffi::OsString::from("out.txt")]);
}

#[test]
fn round_trips_unicode_and_newlines() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("out.txt");
    let content = "line one\nüñîçødé\n\ttabbed\n";
    atomic_write(&path, content).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
}
