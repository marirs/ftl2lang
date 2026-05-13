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
    );
    assert_eq!(result, std::path::PathBuf::from("/work/de/auth/login.ftl"));
}
