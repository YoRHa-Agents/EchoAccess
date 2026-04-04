use clap::Subcommand;

pub mod config_cmd;
pub mod profile;
pub mod sync;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize EchoAccess on this device
    Init,
    /// Show current sync status
    Status,
    /// Sync operations (upload, download, check)
    #[command(subcommand)]
    Sync(sync::SyncCommands),
    /// Profile management (list, show, validate)
    #[command(subcommand)]
    Profile(profile::ProfileCommands),
    /// Configuration management (show, path)
    #[command(subcommand)]
    Config(config_cmd::ConfigCommands),
}

pub async fn execute(cmd: Commands, verbose: bool) -> echoax_core::Result<()> {
    match cmd {
        Commands::Init => {
            println!("Initializing EchoAccess...");
            Ok(())
        }
        Commands::Status => {
            println!("EchoAccess status: ready");
            Ok(())
        }
        Commands::Sync(sub) => sync::execute(sub, verbose).await,
        Commands::Profile(sub) => profile::execute(sub).await,
        Commands::Config(sub) => config_cmd::execute(sub).await,
    }
}
