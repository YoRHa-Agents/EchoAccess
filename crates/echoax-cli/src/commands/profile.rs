use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// List all configured profiles
    List,
    /// Show details for a named profile
    Show {
        /// Profile name to display
        name: String,
    },
    /// Validate a TOML profile file
    Validate {
        /// Path to the profile TOML file
        path: String,
    },
}

pub async fn execute(cmd: ProfileCommands) -> echoax_core::Result<()> {
    match cmd {
        ProfileCommands::List => {
            println!("No profiles configured yet");
            Ok(())
        }
        ProfileCommands::Show { name } => {
            println!("Profile '{name}': not found");
            Ok(())
        }
        ProfileCommands::Validate { path } => {
            let profile =
                echoax_core::profile::load_profile(std::path::Path::new(&path))?;
            println!(
                "Profile '{}' is valid ({} sync rules)",
                profile.device.hostname,
                profile.sync_rules.len()
            );
            Ok(())
        }
    }
}
