use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// SSH 配置管理器
pub struct SshManager {
    ssh_dir: PathBuf,
    config_path: PathBuf,
}

impl SshManager {
    /// 创建新的 SSH 管理器
    pub fn new() -> Result<Self> {
        let home = home::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        let ssh_dir = home.join(".ssh");
        let config_path = ssh_dir.join("config");
        
        Ok(Self { ssh_dir, config_path })
    }
    
    /// 确保 SSH 目录存在
    pub fn ensure_ssh_dir(&self) -> Result<()> {
        if !self.ssh_dir.exists() {
            fs::create_dir_all(&self.ssh_dir)
                .context("无法创建 .ssh 目录")?;
            
            // 设置正确的权限 (700)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&self.ssh_dir, fs::Permissions::from_mode(0o700))?;
            }
        }
        Ok(())
    }
    
    /// 检查密钥文件是否存在
    pub fn key_exists(&self, key_path: &Path) -> bool {
        let expanded = self.expand_path(key_path);
        expanded.exists()
    }
    
    /// 获取密钥对应的公钥路径
    pub fn get_public_key_path(&self, private_key: &Path) -> PathBuf {
        let mut pub_path = private_key.to_path_buf();
        let file_name = pub_path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| format!("{}.pub", n))
            .unwrap_or_else(|| "id_rsa.pub".to_string());
        pub_path.set_file_name(file_name);
        pub_path
    }
    
    /// 读取公钥内容
    pub fn read_public_key(&self, private_key: &Path) -> Result<String> {
        let pub_path = self.get_public_key_path(private_key);
        let expanded = self.expand_path(&pub_path);
        
        fs::read_to_string(&expanded)
            .with_context(|| format!("无法读取公钥文件: {}", expanded.display()))
    }
    
    /// 添加 SSH 配置条目
    pub fn add_host_config(&self, host_alias: &str, hostname: &str, identity_file: &Path, user: &str) -> Result<()> {
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
        
        // 读取现有配置
        let mut existing = if self.config_path.exists() {
            fs::read_to_string(&self.config_path)
                .context("无法读取 SSH 配置文件")?
        } else {
            String::new()
        };
        
        // 检查是否已存在该 Host 配置
        if existing.contains(&format!("Host {}", host_alias)) {
            // 移除旧配置
            existing = self.remove_host_from_config(&existing, host_alias);
        }
        
        // 添加新配置
        existing.push_str(&config_entry);
        
        fs::write(&self.config_path, existing)
            .context("无法写入 SSH 配置文件")?;
        
        // 设置正确的权限 (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.config_path, fs::Permissions::from_mode(0o600))?;
        }
        
        Ok(())
    }
    
    /// 从配置中移除指定 Host
    fn remove_host_from_config(&self, config: &str, host_alias: &str) -> String {
        let mut result = String::new();
        let mut skip = false;
        let mut skip_comment = false;
        
        for line in config.lines() {
            // 检查是否是 gid 管理的注释
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
    
    /// 列出所有 gid 管理的 SSH 配置
    pub fn list_managed_hosts(&self) -> Result<Vec<SshHostConfig>> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.config_path)?;
        let mut hosts = Vec::new();
        let mut current_host: Option<SshHostConfig> = None;
        let mut is_managed = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.contains("# gid managed") {
                is_managed = true;
                continue;
            }
            
            if line.starts_with("Host ") {
                if let Some(host) = current_host.take() {
                    hosts.push(host);
                }
                
                let alias = line.trim_start_matches("Host ").trim();
                current_host = Some(SshHostConfig {
                    alias: alias.to_string(),
                    hostname: String::new(),
                    user: String::new(),
                    identity_file: PathBuf::new(),
                    managed: is_managed,
                });
                is_managed = false;
            } else if let Some(ref mut host) = current_host {
                if line.starts_with("HostName ") {
                    host.hostname = line.trim_start_matches("HostName ").trim().to_string();
                } else if line.starts_with("User ") {
                    host.user = line.trim_start_matches("User ").trim().to_string();
                } else if line.starts_with("IdentityFile ") {
                    host.identity_file = PathBuf::from(
                        line.trim_start_matches("IdentityFile ").trim()
                    );
                }
            }
        }
        
        if let Some(host) = current_host {
            hosts.push(host);
        }
        
        // 只返回 gid 管理的配置
        Ok(hosts.into_iter().filter(|h| h.managed).collect())
    }
    
    /// 生成新的 SSH 密钥对
    pub fn generate_key(&self, name: &str, email: &str) -> Result<PathBuf> {
        self.ensure_ssh_dir()?;
        
        let key_name = format!("id_ed25519_gid_{}", name);
        let key_path = self.ssh_dir.join(&key_name);
        
        if key_path.exists() {
            anyhow::bail!("密钥文件已存在: {}", key_path.display());
        }
        
        // 使用 ssh-keygen 生成密钥
        let output = std::process::Command::new("ssh-keygen")
            .args([
                "-t", "ed25519",
                "-C", email,
                "-f", key_path.to_str().unwrap(),
                "-N", "", // 空密码
            ])
            .output()
            .context("无法执行 ssh-keygen")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ssh-keygen 失败: {}", stderr);
        }
        
        Ok(key_path)
    }
    
    /// 展开路径中的 ~ 符号
    fn expand_path(&self, path: &Path) -> PathBuf {
        if let Some(path_str) = path.to_str() {
            if path_str.starts_with("~/") {
                if let Some(home) = home::home_dir() {
                    return home.join(&path_str[2..]);
                }
            }
        }
        path.to_path_buf()
    }
    
    /// 为身份配置 SSH
    pub fn configure_for_identity(&self, identity_id: &str, hostname: &str, key_path: &Path) -> Result<String> {
        let host_alias = format!("{}-{}", hostname.replace('.', "-"), identity_id);
        self.add_host_config(&host_alias, hostname, key_path, "git")?;
        Ok(host_alias)
    }
}

/// SSH Host 配置
#[derive(Debug)]
pub struct SshHostConfig {
    pub alias: String,
    pub hostname: String,
    pub user: String,
    pub identity_file: PathBuf,
    pub managed: bool,
}

