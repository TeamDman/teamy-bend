// SPDX-License-Identifier: MPL-2.0
use std::process::ExitCode;
fn main() -> ExitCode {
    let path = std::env::args_os().nth(1).expect("fixture path");
    let executable = std::env::args().nth(2).as_deref() == Some("--executable");
    let checked = if executable {
        let Ok(book) = teamy_bend::syntax::load_executable(path) else {
            println!("load");
            return ExitCode::FAILURE;
        };
        teamy_bend::kernel::check_executable(&book).is_ok()
    } else {
        let Ok(book) = teamy_bend::syntax::load(path) else {
            println!("load");
            return ExitCode::FAILURE;
        };
        teamy_bend::kernel::check_book(&book).is_ok()
    };
    if checked {
        println!("accepted");
        ExitCode::SUCCESS
    } else {
        println!("check");
        ExitCode::FAILURE
    }
}
