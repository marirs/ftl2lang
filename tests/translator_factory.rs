use ftl2lang::config::{Config, DeeplConfig};
use ftl2lang::translator::factory::build_translator;

#[test]
fn builds_gtranslate_with_no_config() {
    let cfg = Config::default();
    let t = build_translator(Some("gtranslate"), &cfg).unwrap();
    assert_eq!(t.name(), "gtranslate");
}

#[test]
fn deepl_without_section_errors() {
    // No [deepl] section at all → the error names the section.
    let cfg = Config::default();
    // unwrap_err() requires T: Debug; use .err().unwrap() to avoid adding
    // Debug to Box<dyn Translator>.
    let err = build_translator(Some("deepl"), &cfg).err().unwrap();
    let msg = format!("{}", err);
    assert!(msg.contains("deepl"), "got: {}", msg);
    assert!(msg.contains("[deepl]"), "got: {}", msg);
}

#[test]
fn deepl_with_section_but_no_key_errors() {
    // [deepl] section present but api_key missing → the error names api_key.
    let cfg = Config {
        deepl: Some(DeeplConfig {
            api_key: None,
            api_url: None,
        }),
        ..Default::default()
    };
    let err = build_translator(Some("deepl"), &cfg).err().unwrap();
    let msg = format!("{}", err);
    assert!(msg.contains("api_key"), "got: {}", msg);
}

#[test]
fn deepl_with_key_builds() {
    let cfg = Config {
        deepl: Some(DeeplConfig {
            api_key: Some("X".into()),
            api_url: None,
        }),
        ..Default::default()
    };
    let t = build_translator(Some("deepl"), &cfg).unwrap();
    assert_eq!(t.name(), "deepl");
}

#[test]
fn default_to_gtranslate_when_no_flag_no_config() {
    let cfg = Config::default();
    let t = build_translator(None, &cfg).unwrap();
    assert_eq!(t.name(), "gtranslate");
}

#[test]
fn unknown_translator_name_errors() {
    let cfg = Config::default();
    // unwrap_err() requires T: Debug; use .err().unwrap() to avoid adding
    // Debug to Box<dyn Translator>.
    let err = build_translator(Some("nopes"), &cfg).err().unwrap();
    assert!(format!("{}", err).contains("nopes"));
}
