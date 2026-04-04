use clap::Subcommand;

use echoax_core::config::model::AppConfig;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Print the configuration directory path
    Path,
}

pub async fn execute(cmd: ConfigCommands) -> echoax_core::Result<()> {
    let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
    let config_path = config_dir.join("config.toml");

    match cmd {
        ConfigCommands::Show => {
            let config = if config_path.exists() {
                AppConfig::load(&config_path)?
            } else {
                AppConfig::default()
            };
            let toml_str = toml::to_string_pretty(&config).map_err(|e| {
                echoax_core::EchoAccessError::Config(format!("Serialization error: {e}"))
            })?;
            println!("# EchoAccess Configuration");
            println!("# Path: {}", config_path.display());
            println!();
            println!("{toml_str}");
            Ok(())
        }
        ConfigCommands::Path => {
            println!("{}", config_dir.display());
            Ok(())
        }
    }
}
