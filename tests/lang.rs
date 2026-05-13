use ftl2lang::lang::{normalize, display_name};

#[test]
fn normalize_lowercases_simple_code() {
    assert_eq!(normalize("DE"), "de");
}

#[test]
fn normalize_keeps_region_uppercase() {
    assert_eq!(normalize("pt-br"), "pt-BR");
    assert_eq!(normalize("PT-br"), "pt-BR");
}

#[test]
fn normalize_trims_whitespace() {
    assert_eq!(normalize("  fr  "), "fr");
}

#[test]
fn display_name_known_codes() {
    assert_eq!(display_name("en"), "English");
    assert_eq!(display_name("de"), "German");
    assert_eq!(display_name("ta"), "Tamil");
}

#[test]
fn display_name_unknown_falls_back_to_code() {
    assert_eq!(display_name("xx"), "xx");
}
