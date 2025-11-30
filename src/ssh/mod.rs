use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// SSH Configuration Manager
pub struct SshManager {
    ssh_dir: PathBuf,
    config_path: PathBuf,
}

impl SshManager {
    /// Create new SSH manager
    pub fn new() -> Result<Self> {
        let home =
            home::home_dir().ok_or_else(|| anyhow::anyhow!("Could not get user home directory"))?;
        let ssh_dir = home.join(".ssh");
        let config_path = ssh_dir.join("config");

        Ok(Self {
            ssh_dir,
            config_path,
        })
    }

    /// Ensure SSH directory exists
    pub fn ensure_ssh_dir(&self) -> Result<()> {
        if !self.ssh_dir.exists() {
            fs::create_dir_all(&self.ssh_dir).context("Could not create .ssh directory")?;

            // Set correct permissions (700)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&self.ssh_dir, fs::Permissions::from_mode(0o700))?;
            }
        }
        Ok(())
    }

    /// Check if key file exists
    pub fn key_exists(&self, key_path: &Path) -> bool {
        let expanded = self.expand_path(key_path);
        expanded.exists()
    }

    /// Get public key path for private key
    pub fn get_public_key_path(&self, private_key: &Path) -> PathBuf {
        let mut pub_path = private_key.to_path_buf();
        let file_name = pub_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| format!("{n}.pub"))
            .unwrap_or_else(|| "id_rsa.pub".to_string());
        pub_path.set_file_name(file_name);
        pub_path
    }

    /// Read public key content
    pub fn read_public_key(&self, private_key: &Path) -> Result<String> {
        let pub_path = self.get_public_key_path(private_key);
        let expanded = self.expand_path(&pub_path);

        fs::read_to_string(&expanded)
            .with_context(|| format!("Could not read public key file: {}", expanded.display()))
    }

    /// Add SSH configuration entry
    pub fn add_host_config(
        &self,
        host_alias: &str,
        hostname: &str,
        identity_file: &Path,
        user: &str,
    ) -> Result<()> {
        self.ensure_ssh_dir()?;

        let config_entry = format!(
            r#"
# gid managed - {}
Host {}
    HostName {}
    User {}
    IdentityFile {}
    IdentitiesOnly yes
"#,
            host_alias,
            host_alias,
            hostname,
            user,
            identity_file.display()
        );

        // Read existing configuration
        let mut existing = if self.config_path.exists() {
            fs::read_to_string(&self.config_path).context("Could not read SSH config file")?
        } else {
            String::new()
        };

        // Check if Host config already exists
        if existing.contains(&format!("Host {host_alias}")) {
            // Remove old configuration
            existing = self.remove_host_from_config(&existing, host_alias);
        }

        // Add new configuration
        existing.push_str(&config_entry);

        fs::write(&self.config_path, existing).context("Could not write SSH config file")?;

        // Set correct permissions (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.config_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Remove specified Host from configuration
    fn remove_host_from_config(&self, config: &str, host_alias: &str) -> String {
        let mut result = String::new();
        let mut skip = false;
        let mut skip_comment = false;

        for line in config.lines() {
            // Check if it is a gid managed comment
            if line.contains("# gid managed") && line.contains(host_alias) {
                skip_comment = true;
                continue;
            }

            if skip_comment && line.starts_with("Host ") {
                if line.contains(host_alias) {
                    skip = true;
                    skip_comment = false;
                    continue;
                }
                skip_comment = false;
            }

            if skip {
                if line.starts_with("Host ") {
                    skip = false;
                    result.push_str(line);
                    result.push('\n');
                }
                continue;
            }

            if !skip_comment {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Generate new SSH key pair
    pub fn generate_key(&self, name: &str, email: &str) -> Result<PathBuf> {
        self.ensure_ssh_dir()?;

        let key_name = format!("id_ed25519_gid_{name}");
        let key_path = self.ssh_dir.join(&key_name);

        if key_path.exists() {
            anyhow::bail!("Key file already exists: {}", key_path.display());
        }

        // Generate key using ssh-keygen
        let output = std::process::Command::new("ssh-keygen")
            .args([
                "-t",
                "ed25519",
                "-C",
                email,
                "-f",
                key_path.to_str().unwrap(),
                "-N",
                "", // Empty passphrase
            ])
            .output()
            .context("Could not execute ssh-keygen")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ssh-keygen failed: {stderr}");
        }

        Ok(key_path)
    }

    /// Expand ~ symbol in path
    fn expand_path(&self, path: &Path) -> PathBuf {
        if let Some(path_str) = path.to_str() {
            if let Some(stripped) = path_str.strip_prefix("~/") {
                if let Some(home) = home::home_dir() {
                    return home.join(stripped);
                }
            }
        }
        path.to_path_buf()
    }

    /// Configure SSH for identity
    pub fn configure_for_identity(
        &self,
        identity_id: &str,
        hostname: &str,
        key_path: &Path,
    ) -> Result<String> {
        let host_alias = format!("{}-{}", hostname.replace('.', "-"), identity_id);
        self.add_host_config(&host_alias, hostname, key_path, "git")?;
        Ok(host_alias)
    }

    /// Check if ssh-agent is running
    pub fn is_agent_running(&self) -> bool {
        std::process::Command::new("ssh-add")
            .arg("-l")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Add key to ssh-agent
    pub fn add_to_agent(&self, key_path: &Path) -> Result<()> {
        let expanded = self.expand_path(key_path);

        if !expanded.exists() {
            anyhow::bail!("SSH key file does not exist: {}", expanded.display());
        }

        let output = std::process::Command::new("ssh-add")
            .arg(expanded.to_str().unwrap())
            .output()
            .context("无法执行 ssh-add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to add key to ssh-agent: {stderr}");
        }

        Ok(())
    }

    /// Remove key from ssh-agent
    #[allow(dead_code)]
    pub fn remove_from_agent(&self, key_path: &Path) -> Result<()> {
        let expanded = self.expand_path(key_path);

        let output = std::process::Command::new("ssh-add")
            .arg("-d")
            .arg(expanded.to_str().unwrap())
            .output()
            .context("无法执行 ssh-add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore key not found error
            if !stderr.contains("not found") {
                anyhow::bail!("Failed to remove key from ssh-agent: {stderr}");
            }
        }

        Ok(())
    }

    /// List keys in ssh-agent
    #[allow(dead_code)]
    pub fn list_agent_keys(&self) -> Result<Vec<String>> {
        let output = std::process::Command::new("ssh-add")
            .arg("-l")
            .output()
            .context("无法执行 ssh-add")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let keys = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        Ok(keys)
    }

    /// Clear all keys from ssh-agent
    #[allow(dead_code)]
    pub fn clear_agent(&self) -> Result<()> {
        let output = std::process::Command::new("ssh-add")
            .arg("-D")
            .output()
            .context("无法执行 ssh-add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to clear ssh-agent: {stderr}");
        }

        Ok(())
    }
}
