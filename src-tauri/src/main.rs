//! Limo CAD desktop entry point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod startup;

fn main() -> std::process::ExitCode {
    use startup::Startup;
    let startup = match startup::parse(std::env::args_os().skip(1)) {
        Ok(startup) => startup,
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::from(2);
        }
    };
    if startup == Startup::Help {
        println!("{}", startup::USAGE);
        return std::process::ExitCode::SUCCESS;
    }

    if let Startup::Recipe(recipe) = startup {
        if matches!(nbcad_mcp::open_recipe_in_running_desktop(recipe), Ok(true)) {
            return std::process::ExitCode::SUCCESS;
        }
    }

    if std::env::var_os("NBCAD_DESKTOP_BIN").is_none() {
        if let Ok(executable) = std::env::current_exe() {
            std::env::set_var("NBCAD_DESKTOP_BIN", executable);
        }
    }

    if startup == Startup::Headless {
        return match nbcad_mcp::run_stdio() {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Limo CAD MCP failed: {error}");
                std::process::ExitCode::FAILURE
            }
        };
    }

    if let Err(error) = nbcad_mcp::prepare_desktop_stdio() {
        eprintln!("Could not prepare local stdio MCP: {error}");
        return std::process::ExitCode::FAILURE;
    }

    if let Err(error) = std::thread::Builder::new()
        .name("cad-stdio".into())
        .spawn(|| {
            if let Err(error) = nbcad_mcp::run_desktop_stdio() {
                eprintln!("Limo CAD stdio MCP disconnected: {error}");
            }
        })
    {
        eprintln!("Could not start local stdio MCP: {error}");
    }
    let exit = nbcad_lib::native_viewport::winit_host::run_with_recipe(match startup {
        Startup::Recipe(recipe) => Some(recipe),
        _ => None,
    });
    let _ = nbcad_mcp::shutdown_desktop_stdio(std::time::Duration::from_secs(3));
    exit
}
