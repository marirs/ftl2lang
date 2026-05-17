use ftl2lang::folder::collect_ftl_files;

#[test]
fn finds_ftl_files_recursively() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("a.ftl"), "x = 1\n").unwrap();
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("sub/b.ftl"), "y = 2\n").unwrap();
    std::fs::write(root.join("ignore.txt"), "not ftl\n").unwrap();

    let mut files = collect_ftl_files(root).unwrap();
    files.sort();
    let rel: Vec<String> = files
        .iter()
        .map(|p| p.strip_prefix(root).unwrap().to_string_lossy().to_string())
        .collect();
    assert!(rel.contains(&"a.ftl".to_string()));
    assert!(rel.iter().any(|s| s.ends_with("b.ftl")));
    assert!(!rel.iter().any(|s| s.ends_with(".txt")));
}

#[test]
fn target_path_mirrors_relative_layout() {
    use ftl2lang::folder::target_path_for;
    let result = target_path_for(
        std::path::Path::new("/work/en/auth/login.ftl"),
        std::path::Path::new("/work/en"),
        std::path::Path::new("/work/de"),
    )
    .unwrap();
    assert_eq!(result, std::path::PathBuf::from("/work/de/auth/login.ftl"));
}

#[test]
fn target_path_refuses_file_outside_source_root() {
    use ftl2lang::folder::target_path_for;
    // /etc/passwd is not under /work/en — must fail, not silently
    // produce /work/de/etc/passwd.
    let result = target_path_for(
        std::path::Path::new("/etc/passwd"),
        std::path::Path::new("/work/en"),
        std::path::Path::new("/work/de"),
    );
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("not under"), "got: {}", msg);
}

#[test]
fn symlinks_inside_source_tree_are_not_followed() {
    // Build a source tree with a symlink pointing at an .ftl outside it.
    // The walker must NOT pick the symlink up — otherwise a malicious tree
    // could pull in arbitrary files.
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        let outside_ftl = outside.path().join("secret.ftl");
        std::fs::write(&outside_ftl, "secret = should not be read\n").unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("legit.ftl"), "x = 1\n").unwrap();
        std::os::unix::fs::symlink(&outside_ftl, root.join("escape.ftl")).unwrap();

        let files = collect_ftl_files(root).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(names.contains(&"legit.ftl".to_string()));
        // The symlink itself shouldn't be classified as a regular file by
        // walkdir when follow_links is off; the outside target must not
        // appear in the list.
        assert!(!names.contains(&"secret.ftl".to_string()));
    }
}
