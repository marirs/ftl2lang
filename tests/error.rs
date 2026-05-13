use ftl2lang::error::{exit_code, AppError};

#[test]
fn missing_api_key_has_config_exit_code() {
    let e = AppError::MissingApiKey {
        backend: "deepl".into(),
        field: "[deepl].api_key".into(),
    };
    // MissingApiKey shares exit code 6 with Config (it's a configuration problem).
    // Exit code 2 is reserved for clap's argument-parsing errors.
    assert_eq!(exit_code(&e), 6);
}

#[test]
fn parse_error_has_exit_code_3() {
    let e = AppError::FtlParse {
        path: "test.ftl".into(),
        message: "unexpected token".into(),
    };
    assert_eq!(exit_code(&e), 3);
}
