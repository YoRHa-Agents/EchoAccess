use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Print the configuration directory path
    Path,
}

pub async fn execute(cmd: ConfigCommands) -> echoax_core::Result<()> {
    match cmd {
        ConfigCommands::Show => {
            println!("Config: (default settings)");
            Ok(())
        }
        ConfigCommands::Path => {
            let dir = dirs::config_dir().unwrap_or_default().join("echoax");
            println!("{}", dir.display());
            Ok(())
        }
    }
}
