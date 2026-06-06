use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use rustpanel_core::{
    AppSpec, PanelPaths, render_env_file, render_nginx_server, render_systemd_service,
};

#[derive(Debug, Parser)]
#[command(name = "rustpanel")]
#[command(about = "A small systemd-first server panel")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Render sample config artifacts for the first vertical slice.
    RenderSample {
        #[arg(default_value = "all")]
        artifact: Artifact,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum Artifact {
    All,
    Systemd,
    Env,
    Nginx,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::RenderSample { artifact } => render_sample(artifact),
    }
}

fn render_sample(artifact: Artifact) -> Result<()> {
    let app = AppSpec::sample();
    let paths = PanelPaths::default();

    match artifact {
        Artifact::All => {
            print_systemd(&app, &paths)?;
            print_env(&app)?;
            print_nginx(&app, &paths)?;
        }
        Artifact::Systemd => print_systemd(&app, &paths)?,
        Artifact::Env => print_env(&app)?,
        Artifact::Nginx => print_nginx(&app, &paths)?,
    }

    Ok(())
}

fn print_systemd(app: &AppSpec, paths: &PanelPaths) -> Result<()> {
    let unit = render_systemd_service(app, paths)?;
    println!(
        "--- {} ---",
        paths.systemd_unit_for(&unit.unit_name).display()
    );
    print!("{}", unit.contents);
    println!();
    Ok(())
}

fn print_env(app: &AppSpec) -> Result<()> {
    println!("--- /etc/rustpanel/apps/{}.env ---", app.id);
    print!("{}", render_env_file(app)?);
    println!();
    Ok(())
}

fn print_nginx(app: &AppSpec, paths: &PanelPaths) -> Result<()> {
    for domain in &app.domains {
        let config = render_nginx_server(app, domain, paths)?;
        println!(
            "--- {} ---",
            paths.nginx_config_for(&app.id, &domain.name).display()
        );
        print!("{}", config.contents);
        println!();
    }

    Ok(())
}
