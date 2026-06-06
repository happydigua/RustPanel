use anyhow::Result;
use clap::{Parser, Subcommand};
use rustpanel_core::{AppSpec, PanelPaths, render_nginx_server, render_systemd_service};

#[derive(Debug, Parser)]
#[command(name = "rustpanel-helper")]
#[command(about = "Privileged helper for RustPanel")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the operations this helper will eventually expose over a Unix socket.
    Contract,

    /// Validate that core config rendering is available to the helper.
    ValidateSample,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Contract => print_contract(),
        Command::ValidateSample => validate_sample(),
    }
}

fn print_contract() -> Result<()> {
    println!("rustpanel-helper accepts structured privileged operations only:");
    println!("  WriteSystemdUnit {{ app_id }}");
    println!("  DaemonReload");
    println!("  EnableService {{ service }}");
    println!("  RestartService {{ service }}");
    println!("  WriteNginxConfig {{ app_id, domain }}");
    println!("  TestNginxConfig");
    println!("  ReloadNginx");
    println!("  IssueCertificate {{ domain }}");
    Ok(())
}

fn validate_sample() -> Result<()> {
    let app = AppSpec::sample();
    let paths = PanelPaths::default();

    let _unit = render_systemd_service(&app, &paths)?;
    for domain in &app.domains {
        let _nginx = render_nginx_server(&app, domain, &paths)?;
    }

    println!("sample helper inputs are valid");
    Ok(())
}
