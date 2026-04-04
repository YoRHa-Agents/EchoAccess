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
        None => web::start_server(9876, false).await,
        Some(cmd) => commands::execute(cmd, cli.verbose).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
