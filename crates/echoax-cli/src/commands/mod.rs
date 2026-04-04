use clap::Subcommand;

pub mod config_cmd;
pub mod profile;
pub mod sync;

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Web UI dashboard (default when no command given)
    Web {
        /// Port to listen on
        #[arg(long, default_value = "9876")]
        port: u16,
        /// Don't auto-open the browser
        #[arg(long)]
        no_open: bool,
    },
    /// Launch the TUI terminal dashboard
    Tui,
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
        Commands::Web { port, no_open } => crate::web::start_server(port, no_open).await,
        Commands::Tui => {
            println!("EchoAccess TUI (NieR: Automata style)");
            println!("TUI interactive mode coming soon.");
            Ok(())
        }
        Commands::Init => {
            let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
            std::fs::create_dir_all(config_dir.join("profiles"))
                .map_err(echoax_core::EchoAccessError::Io)?;
            println!("Initialized EchoAccess at {}", config_dir.display());
            Ok(())
        }
        Commands::Status => {
            println!("EchoAccess v{}", env!("CARGO_PKG_VERSION"));
            println!("Session  : Locked");
            println!("Cloud    : Disconnected");
            if verbose {
                println!(
                    "Config   : {}",
                    dirs::config_dir()
                        .unwrap_or_default()
                        .join("echoax")
                        .display()
                );
            }
            Ok(())
        }
        Commands::Sync(sub) => sync::execute(sub, verbose).await,
        Commands::Profile(sub) => profile::execute(sub).await,
        Commands::Config(sub) => config_cmd::execute(sub).await,
    }
}
