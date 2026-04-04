use clap::Subcommand;

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Upload local config to cloud
    Upload,
    /// Download config from cloud
    Download,
    /// Check sync status of all files
    Check,
}

pub async fn execute(cmd: SyncCommands, _verbose: bool) -> echoax_core::Result<()> {
    match cmd {
        SyncCommands::Upload => {
            println!("Sync upload: not yet connected to cloud backend");
            Ok(())
        }
        SyncCommands::Download => {
            println!("Sync download: not yet connected");
            Ok(())
        }
        SyncCommands::Check => {
            println!("Sync check: all files up to date (stub)");
            Ok(())
        }
    }
}
