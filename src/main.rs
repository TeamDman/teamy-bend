fn main() -> eyre::Result<()> {
    let code = teamy_bend::main()?;
    if code != 0 {
        // The worker, writers and logging guards have returned and dropped.
        // Windows preserves all status bits; Unix applies its process-status
        // convention. A Bend Halt is not a Rust error with an extra diagnostic.
        std::process::exit(i32::from_ne_bytes(code.to_ne_bytes()));
    }
    Ok(())
}
