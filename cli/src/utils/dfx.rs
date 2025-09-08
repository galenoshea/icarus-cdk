use anyhow::Result;
use which::which;

use super::run_command;

pub struct DfxInfo {
    pub is_installed: bool,
    pub is_running: bool,
}

pub async fn check_dfx() -> Result<DfxInfo> {
    let is_installed = which("dfx").is_ok();

    if !is_installed {
        return Ok(DfxInfo {
            is_installed: false,
            is_running: false,
        });
    }

    let is_running = run_command("dfx", &["ping"], None).await.is_ok();

    Ok(DfxInfo {
        is_installed,
        is_running,
    })
}

pub async fn ensure_dfx_running() -> Result<()> {
    let info = check_dfx().await?;

    if !info.is_installed {
        anyhow::bail!(
            "dfx is not installed. Please install it from https://internetcomputer.org/docs/current/developer-docs/setup/install/"
        );
    }

    if !info.is_running {
        super::print_info("Starting local dfx network...");
        run_command("dfx", &["start", "--clean", "--background"], None).await?;

        // Wait for dfx to start
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    Ok(())
}

pub async fn get_canister_id(canister_name: &str, network: &str) -> Result<String> {
    let output = run_command(
        "dfx",
        &["canister", "id", canister_name, "--network", network],
        None,
    )
    .await?;

    Ok(output.trim().to_string())
}

pub async fn deploy_canister(
    canister_name: &str,
    network: &str,
    with_cycles: Option<u64>,
) -> Result<String> {
    // Get the current principal to use as the owner
    let principal_output = tokio::process::Command::new("dfx")
        .args(&["identity", "get-principal"])
        .output()
        .await?;

    let principal = String::from_utf8_lossy(&principal_output.stdout)
        .trim()
        .to_string();

    // Build the init argument in Candid format
    let init_arg = format!("(principal \"{}\")", principal);

    let mut args = vec![
        "deploy",
        canister_name,
        "--network",
        network,
        "--argument",
        &init_arg,
    ];

    let cycles_str;
    if let Some(cycles) = with_cycles {
        cycles_str = cycles.to_string();
        args.push("--with-cycles");
        args.push(&cycles_str);
    }

    run_command("dfx", &args, None).await?;

    // Small delay to ensure dfx has synced the canister registry
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    get_canister_id(canister_name, network).await
}

pub async fn install_canister(
    canister_name: &str,
    mode: &str, // "install" or "upgrade"
    network: &str,
) -> Result<()> {
    // For local deployments, always pass the principal argument
    if network == "local" {
        // Get the current principal to use as the owner
        let principal_output = tokio::process::Command::new("dfx")
            .args(&["identity", "get-principal"])
            .output()
            .await?;

        let principal = String::from_utf8_lossy(&principal_output.stdout)
            .trim()
            .to_string();
        let init_arg = format!("(principal \"{}\")", principal);

        run_command(
            "dfx",
            &[
                "canister",
                "install",
                canister_name,
                "--mode",
                mode,
                "--network",
                network,
                "--argument",
                &init_arg,
            ],
            None,
        )
        .await?;
    } else {
        // For non-local networks, only pass init argument for install mode
        if mode == "install" {
            // Get the current principal to use as the owner
            let principal_output = tokio::process::Command::new("dfx")
                .args(&["identity", "get-principal"])
                .output()
                .await?;

            let principal = String::from_utf8_lossy(&principal_output.stdout)
                .trim()
                .to_string();
            let init_arg = format!("(principal \"{}\")", principal);

            run_command(
                "dfx",
                &[
                    "canister",
                    "install",
                    canister_name,
                    "--mode",
                    mode,
                    "--network",
                    network,
                    "--argument",
                    &init_arg,
                ],
                None,
            )
            .await?;
        } else {
            // For upgrade mode on non-local networks, don't pass init argument
            run_command(
                "dfx",
                &[
                    "canister",
                    "install",
                    canister_name,
                    "--mode",
                    mode,
                    "--network",
                    network,
                ],
                None,
            )
            .await?;
        }
    }
    Ok(())
}
