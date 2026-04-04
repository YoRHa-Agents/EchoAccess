use std::path::Path;

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
    let profiles_dir = dirs::config_dir()
        .unwrap_or_default()
        .join("echoax")
        .join("profiles");

    match cmd {
        ProfileCommands::List => {
            if !profiles_dir.exists() {
                println!("No profiles directory found. Run 'echo_access init' first.");
                return Ok(());
            }

            let mut found = false;
            if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "toml") {
                        let name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        match echoax_core::profile::load_profile(&path) {
                            Ok(profile) => {
                                println!(
                                    "  {name:<20} {:<20} ({} rules)",
                                    profile.device.hostname,
                                    profile.sync_rules.len()
                                );
                                found = true;
                            }
                            Err(e) => {
                                eprintln!("  {name:<20} ERROR: {e}");
                                found = true;
                            }
                        }
                    }
                }
            }

            if !found {
                println!("No profiles found in {}", profiles_dir.display());
            }
            Ok(())
        }
        ProfileCommands::Show { name } => {
            let profile_path = profiles_dir.join(format!("{name}.toml"));
            if !profile_path.exists() {
                return Err(echoax_core::EchoAccessError::Profile(format!(
                    "Profile '{name}' not found at {}",
                    profile_path.display()
                )));
            }
            let profile = echoax_core::profile::load_profile(&profile_path)?;
            println!("Profile: {name}");
            println!("  Hostname : {}", profile.device.hostname);
            println!("  OS       : {}", profile.device.os);
            println!("  Role     : {}", profile.device.role);
            println!("  Rules    : {}", profile.sync_rules.len());
            for (i, rule) in profile.sync_rules.iter().enumerate() {
                println!("    [{i}] {} -> {}", rule.source, rule.target);
            }
            Ok(())
        }
        ProfileCommands::Validate { path } => {
            let profile = echoax_core::profile::load_profile(Path::new(&path))?;
            echoax_core::profile::validate_profile(&profile)?;
            println!(
                "Profile '{}' is valid ({} sync rules)",
                profile.device.hostname,
                profile.sync_rules.len()
            );
            Ok(())
        }
    }
}
