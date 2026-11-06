// SPDX-License-Identifier: MPL-2.0
// Research probe: compare the checked normalization and data API boundaries.
use std::error::Error;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::{self};

fn measure(mut value: &TermRef) -> Result<usize, &'static str> {
    let mut count = 0;
    loop {
        match value.as_ref() {
            Term::Ctr { name, args } if name == "Succ" && args.len() == 1 => {
                count += 1;
                value = &args[0];
            }
            Term::Ctr { name, args } if name == "Zero" && args.is_empty() => return Ok(count),
            _ => return Err("expected a Nat constructor tree"),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = std::env::args().nth(1).ok_or("expected one source file")?;
    let loaded = teamy_bend::syntax::load(file)?;
    let checked = kernel::check_book(&loaded)?;
    for (lane, result) in [
        ("normalize_definition", checked.normalize_definition("main")),
        ("evaluate", checked.evaluate("main", &[])),
        ("evaluate_data", checked.evaluate_data("main", &[])),
    ] {
        match result {
            Ok(value) => println!("{lane}:ok:{}", measure(&value)?),
            Err(error) => println!("{lane}:error:{}", error.message),
        }
    }
    Ok(())
}
