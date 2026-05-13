use clap::Parser;
use ftl2lang::cli::Args;

#[test]
fn parses_minimal_invocation() {
    let args = Args::try_parse_from(["ftl2lang", "--to", "de", "en.ftl"]).unwrap();
    assert_eq!(args.to, "de");
    assert_eq!(args.input.to_str(), Some("en.ftl"));
    assert!(args.from.is_none());
    assert!(!args.force);
    assert!(!args.cache);
}

#[test]
fn parses_all_flags() {
    let args = Args::try_parse_from([
        "ftl2lang",
        "--to", "de",
        "--from", "en",
        "--translator", "deepl",
        "--out", "out.ftl",
        "--force",
        "--prune",
        "--cache",
        "--yes",
        "--dry-run",
        "-v",
        "en.ftl",
    ]).unwrap();
    assert_eq!(args.to, "de");
    assert_eq!(args.from.as_deref(), Some("en"));
    assert_eq!(args.translator.as_deref(), Some("deepl"));
    assert_eq!(args.out.as_deref().and_then(|p| p.to_str()), Some("out.ftl"));
    assert!(args.force);
    assert!(args.prune);
    assert!(args.cache);
    assert!(args.yes);
    assert!(args.dry_run);
    assert!(args.verbose);
}

#[test]
fn missing_to_errors() {
    assert!(Args::try_parse_from(["ftl2lang", "en.ftl"]).is_err());
}

#[test]
fn missing_input_errors() {
    assert!(Args::try_parse_from(["ftl2lang", "--to", "de"]).is_err());
}
