//! Managed local checks build and promote their sole runtime before starting CAD.
use anyhow::{ensure, Context, Result};

pub(crate) fn prepare(command: &str, mut arguments: Vec<String>) -> Result<Vec<String>> {
    if matches!(arguments.as_slice(), [argument] if matches!(argument.as_str(), "--help" | "-h"))
        || !matches!(
            command,
            "test-mcp" | "verify-package-mcp" | "run-script" | "cad-call"
        )
        || std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
            && std::env::var("RUNNER_ENVIRONMENT").as_deref() == Ok("github-hosted")
        || !crate::deploy_native::managed_install_exists()?
    {
        return Ok(arguments);
    }

    let worker_arguments = command != "test-mcp";
    let desktop_path = command != "verify-package-mcp";
    let mut server = None;
    let mut desktop = None;
    let mut headless = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--server" => {
                ensure!(server.is_none(), "Duplicate --server option");
                ensure!(index + 1 < arguments.len(), "Missing --server path");
                server = Some(index + 1);
                index += 2;
            }
            "--desktop" if desktop_path => {
                ensure!(desktop.is_none(), "Duplicate --desktop option");
                ensure!(index + 1 < arguments.len(), "Missing --desktop path");
                desktop = Some(index + 1);
                index += 2;
            }
            "--server-arg" => {
                let value = arguments
                    .get(index + 1)
                    .context("Missing --server-arg value")?;
                if worker_arguments {
                    ensure!(
                        value == "--headless" && !headless,
                        "Managed local workers accept exactly one --headless argument"
                    );
                    headless = true;
                }
                index += 2;
            }
            _ => index += 1,
        }
    }

    let source = crate::deploy_native::managed_source_checkout()?;
    let runtime = crate::deploy_native::prepare_runtime(
        &source,
        &crate::deploy_native::BuildOptions::default(),
    )
    .context("Build and promote the current managed CAD runtime before local checks")?;
    let runtime = runtime
        .to_str()
        .context("Managed CAD executable path must be Unicode")?
        .to_owned();
    if let Some(index) = server {
        arguments[index] = runtime.clone();
    } else {
        arguments.extend(["--server".into(), runtime.clone()]);
    }
    if let Some(index) = desktop {
        arguments[index] = runtime.clone();
    }
    if worker_arguments && !headless {
        arguments.extend(["--server-arg".into(), "--headless".into()]);
    }
    eprintln!("Managed local CAD subject: {runtime}");
    Ok(arguments)
}
