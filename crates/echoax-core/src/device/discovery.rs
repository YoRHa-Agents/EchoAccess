use std::path::Path;

use crate::error::{EchoAccessError, Result};

#[derive(Debug, Clone)]
pub struct DiscoveredHost {
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: u16,
    pub identity_files: Vec<String>,
}

pub fn discover_ssh_hosts(ssh_config_path: &Path) -> Result<Vec<DiscoveredHost>> {
    let content = std::fs::read_to_string(ssh_config_path)
        .map_err(|e| EchoAccessError::Network(format!("Cannot read SSH config: {e}")))?;
    parse_ssh_config(&content)
}

fn parse_ssh_config(content: &str) -> Result<Vec<DiscoveredHost>> {
    let mut hosts = Vec::new();
    let mut current: Option<DiscoveredHost> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }
        let (key, value) = (parts[0].to_lowercase(), parts[1].trim());

        match key.as_str() {
            "host" => {
                if let Some(h) = current.take() {
                    if !h.alias.contains('*') && !h.alias.contains('?') {
                        hosts.push(h);
                    }
                }
                current = Some(DiscoveredHost {
                    alias: value.to_string(),
                    hostname: None,
                    user: None,
                    port: 22,
                    identity_files: Vec::new(),
                });
            }
            "hostname" => {
                if let Some(ref mut h) = current {
                    h.hostname = Some(value.to_string());
                }
            }
            "user" => {
                if let Some(ref mut h) = current {
                    h.user = Some(value.to_string());
                }
            }
            "port" => {
                if let Some(ref mut h) = current {
                    h.port = value.parse().unwrap_or(22);
                }
            }
            "identityfile" => {
                if let Some(ref mut h) = current {
                    h.identity_files.push(value.to_string());
                }
            }
            _ => {}
        }
    }
    if let Some(h) = current {
        if !h.alias.contains('*') && !h.alias.contains('?') {
            hosts.push(h);
        }
    }
    Ok(hosts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_config() {
        let config = "\
Host server1
    HostName 192.168.1.1
    User admin
    Port 2222
    IdentityFile ~/.ssh/id_rsa

Host server2
    HostName example.com
    User deploy
";
        let hosts = parse_ssh_config(config).unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].alias, "server1");
        assert_eq!(hosts[0].hostname.as_deref(), Some("192.168.1.1"));
        assert_eq!(hosts[0].port, 2222);
        assert_eq!(hosts[1].user.as_deref(), Some("deploy"));
    }

    #[test]
    fn wildcard_hosts_skipped() {
        let config = "Host *\n    ServerAliveInterval 60\n\nHost dev-*\n    User dev\n";
        let hosts = parse_ssh_config(config).unwrap();
        assert!(hosts.is_empty());
    }

    #[test]
    fn empty_config() {
        let hosts = parse_ssh_config("").unwrap();
        assert!(hosts.is_empty());
    }
}
