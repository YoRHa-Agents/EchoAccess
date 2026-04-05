use clap::Subcommand;

use echoax_core::config::model::AppConfig;

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
        Commands::Tui => echoax_tui::run().await,
        Commands::Init => {
            let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
            let config_path = config_dir.join("config.toml");
            std::fs::create_dir_all(config_dir.join("profiles"))
                .map_err(echoax_core::EchoAccessError::Io)?;
            if !config_path.exists() {
                let default_config = AppConfig::default();
                let toml_str = toml::to_string_pretty(&default_config).map_err(|e| {
                    echoax_core::EchoAccessError::Config(format!("Serialization error: {e}"))
                })?;
                std::fs::write(&config_path, toml_str).map_err(echoax_core::EchoAccessError::Io)?;
                println!("Created default config at {}", config_path.display());
            }
            println!("Initialized EchoAccess at {}", config_dir.display());
            Ok(())
        }
        Commands::Status => {
            let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
            let config_path = config_dir.join("config.toml");
            let config = if config_path.exists() {
                AppConfig::load(&config_path).unwrap_or_default()
            } else {
                AppConfig::default()
            };

            println!("EchoAccess v{}", env!("CARGO_PKG_VERSION"));
            println!("Session  : Locked");
            println!(
                "Cloud    : {}",
                if config.cloud.enabled {
                    format!("Connected ({})", config.cloud.endpoint)
                } else {
                    "Disconnected".to_string()
                }
            );
            if verbose {
                println!("Config   : {}", config_dir.display());
                println!("Language : {}", config.general.language);
                println!("Log Level: {}", config.general.log_level);
                let profiles_dir = config_dir.join("profiles");
                if profiles_dir.exists() {
                    let count = std::fs::read_dir(&profiles_dir)
                        .map(|entries| {
                            entries
                                .filter_map(|e| e.ok())
                                .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
                                .count()
                        })
                        .unwrap_or(0);
                    println!("Profiles : {count}");
                }
            }
            Ok(())
        }
        Commands::Sync(sub) => sync::execute(sub, verbose).await,
        Commands::Profile(sub) => profile::execute(sub).await,
        Commands::Config(sub) => config_cmd::execute(sub).await,
    }
}
