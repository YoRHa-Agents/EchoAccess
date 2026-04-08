use std::io::Write;

use clap::Subcommand;
use echoax_core::updater::{
    archive_extension, binary_name, get_platform_target, parse_github_release, UpdateInfo,
};
use echoax_core::EchoAccessError;
use sha2::{Digest, Sha256};

const GITHUB_API_URL: &str = "https://api.github.com/repos/YoRHa-Agents/EchoAccess/releases/latest";

#[derive(Debug, Subcommand)]
pub enum UpdateCommands {
    /// Check if a newer version is available
    Check,
    /// Download and install the latest version
    Install,
}

pub async fn handle_update(cmd: &UpdateCommands) -> echoax_core::Result<()> {
    match cmd {
        UpdateCommands::Check => {
            let info = fetch_update_info().await?;
            if info.has_update {
                println!(
                    "Update available: {} → {}",
                    info.current_version, info.latest_version
                );
                if !info.download_url.is_empty() {
                    println!("Download: {}", info.download_url);
                }
                if !info.release_notes.is_empty() {
                    println!("\nRelease notes:\n{}", info.release_notes);
                }
            } else {
                println!(
                    "You are running the latest version ({})",
                    info.current_version
                );
            }
            Ok(())
        }
        UpdateCommands::Install => {
            let info = fetch_update_info().await?;
            if !info.has_update {
                println!("Already up to date ({})", info.current_version);
                return Ok(());
            }
            let msg = download_and_install(&info).await?;
            println!("{msg}");
            Ok(())
        }
    }
}

pub async fn fetch_update_info() -> echoax_core::Result<UpdateInfo> {
    let client = reqwest::Client::new();
    let resp = client
        .get(GITHUB_API_URL)
        .header(
            "User-Agent",
            format!("EchoAccess/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| EchoAccessError::Network(format!("Failed to check for updates: {e}")))?;

    if !resp.status().is_success() {
        return Err(EchoAccessError::Network(format!(
            "GitHub API returned status {}",
            resp.status()
        )));
    }

    let json_text = resp
        .text()
        .await
        .map_err(|e| EchoAccessError::Network(format!("Failed to read response: {e}")))?;

    parse_github_release(&json_text, env!("CARGO_PKG_VERSION"))
}

pub async fn download_and_install(info: &UpdateInfo) -> echoax_core::Result<String> {
    if info.download_url.is_empty() {
        let target = get_platform_target();
        return Err(EchoAccessError::Network(format!(
            "No compatible binary found for your platform ({target})"
        )));
    }

    let client = reqwest::Client::new();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message(format!("Downloading v{}...", info.latest_version));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let resp = client
        .get(&info.download_url)
        .header(
            "User-Agent",
            format!("EchoAccess/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| EchoAccessError::Network(format!("Failed to download update: {e}")))?;

    if !resp.status().is_success() {
        pb.finish_and_clear();
        return Err(EchoAccessError::Network(format!(
            "Download failed with status {}",
            resp.status()
        )));
    }

    let archive_bytes = resp
        .bytes()
        .await
        .map_err(|e| EchoAccessError::Network(format!("Failed to read download: {e}")))?;

    pb.finish_with_message("Download complete");

    if !info.checksum_url.is_empty() {
        verify_checksum(
            &client,
            &info.checksum_url,
            &info.download_url,
            &archive_bytes,
        )
        .await?;
    }

    let bin_name = binary_name();
    let ext = archive_extension();

    let new_binary = if ext == "zip" {
        extract_from_zip(&archive_bytes, bin_name)?
    } else {
        extract_from_tar_gz(&archive_bytes, bin_name)?
    };

    replace_binary(&new_binary)?;

    Ok(format!(
        "Successfully updated to v{}. Please restart EchoAccess.",
        info.latest_version
    ))
}

async fn verify_checksum(
    client: &reqwest::Client,
    checksum_url: &str,
    download_url: &str,
    archive_bytes: &[u8],
) -> echoax_core::Result<()> {
    let resp = client
        .get(checksum_url)
        .header(
            "User-Agent",
            format!("EchoAccess/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|e| EchoAccessError::Network(format!("Failed to download checksum: {e}")))?;

    if !resp.status().is_success() {
        return Err(EchoAccessError::Network(format!(
            "Checksum download failed with status {}",
            resp.status()
        )));
    }

    let checksum_text = resp
        .text()
        .await
        .map_err(|e| EchoAccessError::Network(format!("Failed to read checksum: {e}")))?;

    let asset_name = asset_name_from_download_url(download_url)?;
    let expected_hash = extract_expected_hash(&checksum_text, &asset_name)?;

    let mut hasher = Sha256::new();
    hasher.update(archive_bytes);
    let computed_hash = format!("{:x}", hasher.finalize());

    if expected_hash != computed_hash {
        return Err(EchoAccessError::Network(format!(
            "Checksum verification failed: expected {expected_hash}, got {computed_hash}"
        )));
    }

    println!("Checksum verified");
    Ok(())
}

fn asset_name_from_download_url(download_url: &str) -> echoax_core::Result<String> {
    download_url
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .ok_or_else(|| {
            EchoAccessError::Network(format!(
                "Failed to determine asset name from download URL: {download_url}"
            ))
        })
}

fn extract_expected_hash(checksum_text: &str, asset_name: &str) -> echoax_core::Result<String> {
    let lines: Vec<&str> = checksum_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    for line in &lines {
        let mut parts = line.split_whitespace();
        let hash = parts.next().unwrap_or("");
        let file_name = parts.next().unwrap_or("").trim_start_matches('*');
        if !hash.is_empty() && file_name == asset_name {
            return Ok(hash.to_lowercase());
        }
    }

    if lines.len() == 1 {
        let mut parts = lines[0].split_whitespace();
        let hash = parts.next().unwrap_or("").to_lowercase();
        let file_name = parts.next();
        if !hash.is_empty() && file_name.is_none() {
            return Ok(hash);
        }
    }

    Err(EchoAccessError::Network(format!(
        "No checksum entry found for asset '{asset_name}'"
    )))
}

fn extract_from_tar_gz(archive_bytes: &[u8], target_name: &str) -> echoax_core::Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let decoder = GzDecoder::new(archive_bytes);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| EchoAccessError::Network(format!("Failed to read archive: {e}")))?
    {
        let mut entry = entry
            .map_err(|e| EchoAccessError::Network(format!("Failed to read archive entry: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| EchoAccessError::Network(format!("Failed to read entry path: {e}")))?;

        if path.file_name().and_then(|n| n.to_str()) == Some(target_name) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| EchoAccessError::Network(format!("Failed to extract binary: {e}")))?;
            return Ok(buf);
        }
    }

    Err(EchoAccessError::Network(format!(
        "Binary '{target_name}' not found in archive"
    )))
}

fn extract_from_zip(archive_bytes: &[u8], target_name: &str) -> echoax_core::Result<Vec<u8>> {
    use std::io::{Cursor, Read};

    let reader = Cursor::new(archive_bytes);
    let mut zip = zip::ZipArchive::new(reader)
        .map_err(|e| EchoAccessError::Network(format!("Failed to open zip archive: {e}")))?;

    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| EchoAccessError::Network(format!("Failed to read zip entry: {e}")))?;

        let matches = std::path::Path::new(file.name())
            .file_name()
            .and_then(|n| n.to_str())
            == Some(target_name);

        if matches {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| EchoAccessError::Network(format!("Failed to extract binary: {e}")))?;
            return Ok(buf);
        }
    }

    Err(EchoAccessError::Network(format!(
        "Binary '{target_name}' not found in zip archive"
    )))
}

fn replace_binary(new_binary: &[u8]) -> echoax_core::Result<()> {
    let current_exe = std::env::current_exe().map_err(|e| {
        EchoAccessError::Network(format!("Failed to determine current executable path: {e}"))
    })?;

    let parent = current_exe.parent().ok_or_else(|| {
        EchoAccessError::Network("Failed to determine parent directory of executable".into())
    })?;

    let temp_path = parent.join(".echo_access_update_tmp");

    {
        let mut f = std::fs::File::create(&temp_path)
            .map_err(|e| EchoAccessError::Network(format!("Failed to create temp file: {e}")))?;
        f.write_all(new_binary)
            .map_err(|e| EchoAccessError::Network(format!("Failed to write temp file: {e}")))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| EchoAccessError::Network(format!("Failed to set permissions: {e}")))?;
        std::fs::rename(&temp_path, &current_exe)
            .map_err(|e| EchoAccessError::Network(format!("Failed to replace binary: {e}")))?;
    }

    #[cfg(windows)]
    {
        let backup_path = current_exe.with_extension("old");
        let _ = std::fs::remove_file(&backup_path);
        std::fs::rename(&current_exe, &backup_path).map_err(|e| {
            EchoAccessError::Network(format!("Failed to backup current binary: {e}"))
        })?;
        std::fs::rename(&temp_path, &current_exe).map_err(|e| {
            let _ = std::fs::rename(&backup_path, &current_exe);
            EchoAccessError::Network(format!("Failed to install new binary: {e}"))
        })?;
        println!("Please restart EchoAccess to use the new version.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_test_tar_gz(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let buf = Vec::new();
        let encoder = GzEncoder::new(buf, Compression::fast());
        let mut builder = tar::Builder::new(encoder);

        for &(name, data) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, name, data).unwrap();
        }

        builder.into_inner().unwrap().finish().unwrap()
    }

    fn make_test_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut zip = zip::ZipWriter::new(cursor);

        for &(name, data) in entries {
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file(name, options).unwrap();
            zip.write_all(data).unwrap();
        }

        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn extract_tar_gz_finds_binary() {
        let name = binary_name();
        let content = b"fake binary content";
        let archive = make_test_tar_gz(&[(name, content)]);
        let result = extract_from_tar_gz(&archive, name).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn extract_tar_gz_finds_nested_binary() {
        let name = binary_name();
        let path = format!("echoax-v0.2.0-x86_64/{name}");
        let content = b"nested binary";
        let archive = make_test_tar_gz(&[(&path, content)]);
        let result = extract_from_tar_gz(&archive, name).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn extract_tar_gz_missing_binary_errors() {
        let archive = make_test_tar_gz(&[("README.md", b"readme content")]);
        let err = extract_from_tar_gz(&archive, "echo_access").unwrap_err();
        assert!(matches!(err, EchoAccessError::Network(_)));
    }

    #[test]
    fn extract_zip_finds_binary() {
        let name = binary_name();
        let content = b"fake zip binary";
        let archive = make_test_zip(&[(name, content)]);
        let result = extract_from_zip(&archive, name).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn extract_zip_finds_nested_binary() {
        let name = binary_name();
        let path = format!("echoax-v0.2.0/{name}");
        let content = b"nested zip binary";
        let archive = make_test_zip(&[(&path, content)]);
        let result = extract_from_zip(&archive, name).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn extract_zip_missing_binary_errors() {
        let archive = make_test_zip(&[("README.md", b"readme")]);
        let err = extract_from_zip(&archive, "echo_access").unwrap_err();
        assert!(matches!(err, EchoAccessError::Network(_)));
    }

    #[test]
    fn extract_expected_hash_from_combined_manifest() {
        let checksum_text = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  echoax-v0.1.5-x86_64-unknown-linux-gnu.tar.gz\n\
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb  echoax-v0.1.5-x86_64-pc-windows-msvc.zip\n";
        let hash = extract_expected_hash(checksum_text, "echoax-v0.1.5-x86_64-pc-windows-msvc.zip")
            .unwrap();
        assert_eq!(
            hash,
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
    }

    #[test]
    fn extract_expected_hash_from_single_asset_checksum() {
        let checksum_text =
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc  echoax.zip\n";
        let hash = extract_expected_hash(checksum_text, "echoax.zip").unwrap();
        assert_eq!(
            hash,
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        );
    }

    #[test]
    fn extract_expected_hash_errors_when_asset_missing() {
        let checksum_text =
            "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd  other.zip\n";
        let err = extract_expected_hash(checksum_text, "echoax.zip").unwrap_err();
        assert!(format!("{err}").contains("No checksum entry found"));
    }

    #[test]
    fn asset_name_from_download_url_uses_final_path_segment() {
        let name = asset_name_from_download_url(
            "https://github.com/YoRHa-Agents/EchoAccess/releases/download/v0.1.5/echoax.zip",
        )
        .unwrap();
        assert_eq!(name, "echoax.zip");
    }
}
