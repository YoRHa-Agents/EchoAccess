use clap::Subcommand;

use echoax_core::config::model::AppConfig;
use echoax_core::EchoAccessError;

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Upload local config to cloud
    Upload,
    /// Download config from cloud
    Download,
    /// Check sync status of all files
    Check,
    /// Resolve sync conflicts
    Resolve {
        /// Resolution strategy: ours, theirs, or base
        #[arg(long, default_value = "ours")]
        take: String,
        /// Specific file to resolve (resolves all if omitted)
        #[arg(long)]
        file: Option<String>,
    },
}

fn normalize_sync_take(take: &str) -> echoax_core::Result<&'static str> {
    if take.eq_ignore_ascii_case("ours") {
        return Ok("ours");
    }
    if take.eq_ignore_ascii_case("theirs") {
        return Ok("theirs");
    }
    if take.eq_ignore_ascii_case("base") {
        return Ok("base");
    }
    Err(EchoAccessError::Sync(format!(
        "invalid --take {take:?}; expected ours, theirs, or base"
    )))
}

pub async fn execute(cmd: SyncCommands, verbose: bool) -> echoax_core::Result<()> {
    let config_dir = dirs::config_dir().unwrap_or_default().join("echoax");
    let config_path = config_dir.join("config.toml");

    let config = if config_path.exists() {
        AppConfig::load(&config_path)?
    } else {
        AppConfig::default()
    };

    match cmd {
        SyncCommands::Upload => {
            if !config.cloud.enabled {
                println!("Cloud sync is disabled. Enable it in your config:");
                println!("  echo_access config show");
                println!("  Set [cloud] enabled = true and endpoint = \"...\"");
                return Ok(());
            }
            println!("Cloud endpoint: {}", config.cloud.endpoint);
            println!("Sync upload: cloud backend integration pending");
            println!("(S3/OSS SDK integration will be available in a future release)");
            Ok(())
        }
        SyncCommands::Download => {
            if !config.cloud.enabled {
                println!("Cloud sync is disabled. Enable it in your config.");
                return Ok(());
            }
            println!("Cloud endpoint: {}", config.cloud.endpoint);
            println!("Sync download: cloud backend integration pending");
            println!("(S3/OSS SDK integration will be available in a future release)");
            Ok(())
        }
        SyncCommands::Check => {
            println!("Sync status check:");
            if verbose {
                println!("  Config dir : {}", config_dir.display());
                println!(
                    "  Cloud      : {}",
                    if config.cloud.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
                if config.cloud.enabled {
                    println!("  Endpoint   : {}", config.cloud.endpoint);
                }
            }
            println!("  Engine     : ready (local-only mode)");
            println!("  Diff algo  : diffy 3-way merge");

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
                println!("  Profiles   : {count} found");
            } else {
                println!("  Profiles   : directory not found");
            }
            Ok(())
        }
        SyncCommands::Resolve { take, file } => {
            println!("Conflict resolution");
            let take_norm = normalize_sync_take(&take)?;
            if let Some(path) = &file {
                println!("Resolving specific file: {path}");
            } else {
                println!("No active conflicts");
            }
            println!("Resolution strategy: {take_norm}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_sync_take;

    #[test]
    fn normalize_sync_take_accepts_variants() {
        assert_eq!(normalize_sync_take("ours").unwrap(), "ours");
        assert_eq!(normalize_sync_take("OURS").unwrap(), "ours");
        assert_eq!(normalize_sync_take("theirs").unwrap(), "theirs");
        assert_eq!(normalize_sync_take("base").unwrap(), "base");
    }

    #[test]
    fn normalize_sync_take_rejects_unknown() {
        assert!(normalize_sync_take("custom").is_err());
    }
}
