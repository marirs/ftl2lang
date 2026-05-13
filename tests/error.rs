use ftl2lang::error::{exit_code, AppError};

#[test]
fn missing_api_key_has_exit_code_2() {
    let e = AppError::MissingApiKey {
        backend: "deepl".into(),
        field: "[deepl].api_key".into(),
    };
    assert_eq!(exit_code(&e), 2);
}

#[test]
fn parse_error_has_exit_code_3() {
    let e = AppError::FtlParse {
        path: "test.ftl".into(),
        message: "unexpected token".into(),
    };
    assert_eq!(exit_code(&e), 3);
}
