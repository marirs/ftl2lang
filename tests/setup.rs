use ftl2lang::config::{Config, DeeplConfig, GoogleConfig, GtranslateConfig};
use ftl2lang::setup::render_config_toml;

#[test]
fn empty_config_produces_header_only() {
    let cfg = Config::default();
    let out = render_config_toml(&cfg);
    assert!(out.starts_with("# ftl2lang config"));
    assert!(!out.contains("[deepl]"));
    assert!(!out.contains("[google]"));
    assert!(!out.contains("[gtranslate]"));
}

#[test]
fn deepl_section_includes_key_and_url() {
    let cfg = Config {
        deepl: Some(DeeplConfig {
            api_key: Some("DEEPL-KEY".into()),
            api_url: Some("https://api.deepl.com/v2".into()),
        }),
        ..Default::default()
    };
    let out = render_config_toml(&cfg);
    assert!(out.contains("[deepl]"));
    assert!(out.contains("api_key = \"DEEPL-KEY\""));
    assert!(out.contains("api_url = \"https://api.deepl.com/v2\""));
}

#[test]
fn google_section_includes_project_id() {
    let cfg = Config {
        google: Some(GoogleConfig {
            api_key: Some("GKEY".into()),
            project_id: Some("my-proj".into()),
        }),
        ..Default::default()
    };
    let out = render_config_toml(&cfg);
    assert!(out.contains("[google]"));
    assert!(out.contains("api_key = \"GKEY\""));
    assert!(out.contains("project_id = \"my-proj\""));
}

#[test]
fn gtranslate_is_rendered_when_enabled() {
    let cfg = Config {
        gtranslate: Some(GtranslateConfig::default()),
        ..Default::default()
    };
    let out = render_config_toml(&cfg);
    assert!(out.contains("[gtranslate]"));
}

#[test]
fn defaults_render_at_top_level() {
    let cfg = Config {
        default_translator: Some("deepl".into()),
        default_source: Some("en".into()),
        ..Default::default()
    };
    let out = render_config_toml(&cfg);
    assert!(out.contains("default_translator = \"deepl\""));
    assert!(out.contains("default_source = \"en\""));
}

#[test]
fn rendered_output_round_trips_through_config_loader() {
    // The whole point: what we write must be readable by Config::load.
    let cfg = Config {
        default_translator: Some("google".into()),
        default_source: Some("en".into()),
        deepl: Some(DeeplConfig {
            api_key: Some("DEEPL-KEY".into()),
            api_url: None,
        }),
        google: Some(GoogleConfig {
            api_key: Some("GKEY".into()),
            project_id: Some("my-proj".into()),
        }),
        gtranslate: Some(GtranslateConfig::default()),
    };
    let body = render_config_toml(&cfg);

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), body).unwrap();
    let loaded = Config::load_from_path(tmp.path()).unwrap();

    assert_eq!(loaded.default_translator.as_deref(), Some("google"));
    assert_eq!(loaded.default_source.as_deref(), Some("en"));
    assert_eq!(
        loaded.deepl.as_ref().unwrap().api_key.as_deref(),
        Some("DEEPL-KEY")
    );
    assert!(loaded.deepl.as_ref().unwrap().api_url.is_none());
    assert_eq!(
        loaded.google.as_ref().unwrap().project_id.as_deref(),
        Some("my-proj")
    );
    assert!(loaded.gtranslate.is_some());
}

#[test]
fn special_characters_in_values_are_escaped() {
    let cfg = Config {
        deepl: Some(DeeplConfig {
            // Real keys never contain quotes or backslashes, but make sure
            // the escaper handles them so a future weird value doesn't
            // break the TOML.
            api_key: Some(r#"weird"key\with\stuff"#.into()),
            api_url: None,
        }),
        ..Default::default()
    };
    let body = render_config_toml(&cfg);
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), body).unwrap();
    let loaded = Config::load_from_path(tmp.path()).unwrap();
    assert_eq!(
        loaded.deepl.as_ref().unwrap().api_key.as_deref(),
        Some(r#"weird"key\with\stuff"#)
    );
}
