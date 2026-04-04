use clap::Parser;

mod commands;

#[derive(Parser)]
#[command(name = "echoax", about = "EchoAccess — cross-platform config sync")]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,

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
    if let Err(e) = commands::execute(cli.command, cli.verbose).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
