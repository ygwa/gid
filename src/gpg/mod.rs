use anyhow::{Context, Result};
use std::process::Command;

/// GPG 管理器
pub struct GpgManager;

impl GpgManager {
    /// 创建新的 GPG 管理器
    pub fn new() -> Self {
        Self
    }
    
    /// 检查 GPG 是否可用
    pub fn is_available(&self) -> bool {
        Command::new("gpg")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    /// 列出所有密钥
    pub fn list_keys(&self) -> Result<Vec<GpgKey>> {
        let output = Command::new("gpg")
            .args(["--list-secret-keys", "--keyid-format", "long"])
            .output()
            .context("无法执行 gpg 命令")?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let keys = self.parse_gpg_output(&stdout);
        
        Ok(keys)
    }
    
    /// 解析 GPG 输出
    fn parse_gpg_output(&self, output: &str) -> Vec<GpgKey> {
        let mut keys = Vec::new();
        let mut current_key: Option<GpgKey> = None;
        
        for line in output.lines() {
            if line.starts_with("sec") {
                // sec   rsa4096/ABCD1234EF567890 2023-01-01 [SC]
                if let Some(key_id) = self.extract_key_id(line) {
                    current_key = Some(GpgKey {
                        key_id,
                        uid: String::new(),
                        email: None,
                    });
                }
            } else if line.starts_with("uid") && current_key.is_some() {
                // uid           [ultimate] Name <email@example.com>
                let uid = line.trim_start_matches("uid").trim();
                // 移除信任级别标记
                let uid = uid.trim_start_matches(|c: char| c == '[' || c.is_alphabetic() || c == ']' || c.is_whitespace());
                
                if let Some(ref mut key) = current_key {
                    key.uid = uid.to_string();
                    key.email = self.extract_email(uid);
                }
            } else if line.is_empty() && current_key.is_some() {
                if let Some(key) = current_key.take() {
                    keys.push(key);
                }
            }
        }
        
        if let Some(key) = current_key {
            keys.push(key);
        }
        
        keys
    }
    
    /// 从输出中提取密钥 ID
    fn extract_key_id(&self, line: &str) -> Option<String> {
        // sec   rsa4096/ABCD1234EF567890 2023-01-01 [SC]
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key_part = parts[1];
            if let Some(pos) = key_part.find('/') {
                return Some(key_part[pos + 1..].to_string());
            }
        }
        None
    }
    
    /// 从 UID 中提取邮箱
    fn extract_email(&self, uid: &str) -> Option<String> {
        // Name <email@example.com>
        if let Some(start) = uid.find('<') {
            if let Some(end) = uid.find('>') {
                return Some(uid[start + 1..end].to_string());
            }
        }
        None
    }
    
    /// 根据邮箱查找密钥
    pub fn find_key_by_email(&self, email: &str) -> Result<Option<GpgKey>> {
        let keys = self.list_keys()?;
        Ok(keys.into_iter().find(|k| {
            k.email.as_ref().map(|e| e == email).unwrap_or(false)
        }))
    }
    
    /// 验证密钥 ID 是否有效
    pub fn verify_key(&self, key_id: &str) -> Result<bool> {
        let output = Command::new("gpg")
            .args(["--list-secret-keys", key_id])
            .output()
            .context("无法执行 gpg 命令")?;
        
        Ok(output.status.success())
    }
    
    /// 生成新的 GPG 密钥
    pub fn generate_key(&self, name: &str, email: &str) -> Result<String> {
        // 创建批处理输入
        let batch_input = format!(
            r#"%no-protection
Key-Type: eddsa
Key-Curve: ed25519
Key-Usage: sign
Subkey-Type: ecdh
Subkey-Curve: cv25519
Subkey-Usage: encrypt
Name-Real: {}
Name-Email: {}
Expire-Date: 0
%commit
"#,
            name, email
        );
        
        let mut child = Command::new("gpg")
            .args(["--batch", "--gen-key"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("无法启动 gpg")?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(batch_input.as_bytes())?;
        }
        
        let output = child.wait_with_output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("GPG 密钥生成失败: {}", stderr);
        }
        
        // 获取新生成的密钥 ID
        if let Some(key) = self.find_key_by_email(email)? {
            Ok(key.key_id)
        } else {
            anyhow::bail!("无法找到新生成的密钥")
        }
    }
}

/// GPG 密钥信息
#[derive(Debug)]
pub struct GpgKey {
    pub key_id: String,
    pub uid: String,
    pub email: Option<String>,
}

impl std::fmt::Display for GpgKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.key_id, self.uid)
    }
}

