use clap::Parser;
use ftl2lang::cli::Args;

#[test]
fn parses_minimal_invocation() {
    let args = Args::try_parse_from(["ftl2lang", "--to", "de", "en.ftl"]).unwrap();
    assert_eq!(args.to.as_deref(), Some("de"));
    assert_eq!(
        args.input.as_deref().and_then(|p| p.to_str()),
        Some("en.ftl")
    );
    assert!(args.from.is_none());
    assert!(!args.force);
    assert!(!args.cache);
    assert!(!args.list_langs);
    assert!(!args.list_translators);
    args.validate().unwrap();
}

#[test]
fn parses_all_flags() {
    let args = Args::try_parse_from([
        "ftl2lang",
        "--to",
        "de",
        "--from",
        "en",
        "--translator",
        "deepl",
        "--out",
        "out.ftl",
        "--force",
        "--prune",
        "--cache",
        "--yes",
        "--dry-run",
        "-v",
        "en.ftl",
    ])
    .unwrap();
    assert_eq!(args.to.as_deref(), Some("de"));
    assert_eq!(args.from.as_deref(), Some("en"));
    assert_eq!(args.translator.as_deref(), Some("deepl"));
    assert_eq!(
        args.out.as_deref().and_then(|p| p.to_str()),
        Some("out.ftl")
    );
    assert!(args.force);
    assert!(args.prune);
    assert!(args.cache);
    assert!(args.yes);
    assert!(args.dry_run);
    assert!(args.verbose);
    args.validate().unwrap();
}

#[test]
fn missing_to_fails_validation() {
    // Now that --to and INPUT are optional at the clap layer, the error
    // surfaces at validate() time instead of at parse time.
    let args = Args::try_parse_from(["ftl2lang", "en.ftl"]).unwrap();
    let err = args.validate().unwrap_err();
    assert!(err.contains("--to"), "got: {}", err);
}

#[test]
fn missing_input_fails_validation() {
    let args = Args::try_parse_from(["ftl2lang", "--to", "de"]).unwrap();
    let err = args.validate().unwrap_err();
    assert!(err.contains("INPUT"), "got: {}", err);
}

#[test]
fn list_langs_does_not_require_to_or_input() {
    let args = Args::try_parse_from(["ftl2lang", "--list-langs"]).unwrap();
    assert!(args.list_langs);
    assert!(args.input.is_none());
    assert!(args.to.is_none());
    args.validate().unwrap();
}

#[test]
fn list_translators_does_not_require_to_or_input() {
    let args = Args::try_parse_from(["ftl2lang", "--list-translators"]).unwrap();
    assert!(args.list_translators);
    args.validate().unwrap();
}

#[test]
fn clear_cache_does_not_require_to_or_input() {
    let args = Args::try_parse_from(["ftl2lang", "--clear-cache"]).unwrap();
    assert!(args.clear_cache);
    assert!(args.input.is_none());
    assert!(args.to.is_none());
    args.validate().unwrap();
}
