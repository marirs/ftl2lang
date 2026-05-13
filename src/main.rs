use ftl2lang::error::{exit_code, AppError};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(exit_code(&e));
    }
}

fn run() -> Result<(), AppError> {
    Ok(())
}
