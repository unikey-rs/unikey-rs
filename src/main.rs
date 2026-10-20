use std::process::ExitCode;

fn main() -> ExitCode {
    match unikey_rs::cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
