// SPDX-License-Identifier: MPL-2.0
use std::process::ExitCode;
fn main() -> ExitCode {
    let path = std::env::args_os().nth(1).expect("fixture path");
    if std::env::args().nth(2).as_deref() == Some("--executable") {
        let Ok(source) = teamy_bend::syntax::load_executable(path) else {
            println!("load");
            return ExitCode::FAILURE;
        };
        if teamy_bend::kernel::check_executable(&source).is_err() {
            println!("check");
            return ExitCode::FAILURE;
        }
        println!("accepted");
        return ExitCode::SUCCESS;
    }
    let Ok(book) = teamy_bend::syntax::load(path) else {
        println!("load");
        return ExitCode::FAILURE;
    };
    if teamy_bend::kernel::check_book(&book).is_err() {
        println!("check");
        return ExitCode::FAILURE;
    }
    println!("accepted");
    ExitCode::SUCCESS
}
