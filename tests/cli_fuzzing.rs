//! CLI fuzzing tests using figue's arbitrary helper assertions.

use teamy_bend::cli::Cli;

#[test]
fn fuzz_cli_args_consistency() {
    if let Err(e) =
        figue::assert_to_args_consistency::<Cli>(figue::TestToArgsConsistencyConfig::default())
    {
        panic!("CLI argument consistency check failed:\n{e}")
    }
}

#[test]
fn fuzz_cli_args_roundtrip() {
    if let Err(e) = figue::assert_to_args_roundtrip::<Cli>(figue::TestToArgsRoundTrip::default()) {
        panic!("CLI argument roundtrip check failed:\n{e}")
    }
}
