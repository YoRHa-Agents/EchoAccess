use clap::Parser;

mod commands;
mod web;

#[derive(Parser)]
#[command(
    name = "echo_access",
    about = "EchoAccess — cross-platform config sync",
    long_about = "EchoAccess synchronizes configuration files across devices with encryption.\n\nRun without arguments to start the Web UI dashboard."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<commands::Commands>,

    #[arg(long, global = true)]
    verbose: bool,

    #[arg(long, global = true)]
    quiet: bool,

    #[arg(long, global = true, value_name = "PATH")]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None => {
            let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
            let config_path = config_dir.join("config.toml");
            let port = echoax_core::config::model::AppConfig::load(&config_path)
                .map(|c| c.general.port)
                .unwrap_or(9876);
            web::start_server(port, false).await
        }
        Some(cmd) => commands::execute(cmd, cli.verbose).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
