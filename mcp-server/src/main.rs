fn main() -> std::process::ExitCode {
    match nbcad_mcp::run_stdio() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Limo CAD MCP failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
