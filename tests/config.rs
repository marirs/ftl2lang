use ftl2lang::config::Config;
use std::io::Write;

#[test]
fn loads_full_config() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        f,
        r#"
default_translator = "deepl"
default_source = "en"

[deepl]
api_key = "DEEPL-KEY"
api_url = "https://api-free.deepl.com/v2"

[google]
api_key = "GOOGLE-KEY"
project_id = "my-project"

[gtranslate]
"#
    )
    .unwrap();

    let cfg = Config::load_from_path(f.path()).unwrap();
    assert_eq!(cfg.default_translator.as_deref(), Some("deepl"));
    assert_eq!(cfg.default_source.as_deref(), Some("en"));
    assert_eq!(
        cfg.deepl.as_ref().unwrap().api_key.as_deref(),
        Some("DEEPL-KEY")
    );
    assert_eq!(
        cfg.google.as_ref().unwrap().project_id.as_deref(),
        Some("my-project")
    );
}

#[test]
fn missing_config_file_returns_default() {
    let cfg =
        Config::load_from_path(std::path::Path::new("/nonexistent/path/config.toml")).unwrap();
    assert!(cfg.default_translator.is_none());
    assert!(cfg.deepl.is_none());
}

#[test]
fn invalid_toml_returns_error() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "this is not = valid = toml ===").unwrap();
    let result = Config::load_from_path(f.path());
    assert!(result.is_err());
}
