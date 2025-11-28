use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Git 身份配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// 身份 ID（唯一标识）
    pub id: String,

    /// Git 用户名
    pub name: String,

    /// Git 邮箱
    pub email: String,

    /// 描述信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// SSH 私钥路径
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<PathBuf>,

    /// GPG 签名密钥 ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpg_key: Option<String>,

    /// 是否启用 GPG 签名
    #[serde(default)]
    pub gpg_sign: bool,
}

impl Identity {
    /// 创建新身份
    pub fn new(id: String, name: String, email: String) -> Self {
        Self {
            id,
            name,
            email,
            description: None,
            ssh_key: None,
            gpg_key: None,
            gpg_sign: false,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// 设置 SSH 密钥
    pub fn with_ssh_key(mut self, ssh_key: Option<PathBuf>) -> Self {
        self.ssh_key = ssh_key;
        self
    }

    /// 设置 GPG 密钥
    pub fn with_gpg_key(mut self, gpg_key: Option<String>) -> Self {
        if gpg_key.is_some() {
            self.gpg_sign = true;
        }
        self.gpg_key = gpg_key;
        self
    }

    /// 验证身份配置
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("身份 ID 不能为空".to_string());
        }

        if !self
            .id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err("身份 ID 只能包含字母、数字、下划线和连字符".to_string());
        }

        if self.name.is_empty() {
            return Err("姓名不能为空".to_string());
        }

        if self.email.is_empty() {
            return Err("邮箱不能为空".to_string());
        }

        // 简单的邮箱格式验证
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err("邮箱格式不正确".to_string());
        }

        // 验证 SSH 密钥文件是否存在
        if let Some(ref ssh_key) = self.ssh_key {
            let expanded = expand_path(ssh_key);
            if !expanded.exists() {
                return Err(format!("SSH 密钥文件不存在: {}", expanded.display()));
            }
        }

        Ok(())
    }
}

/// 展开路径中的 ~ 符号
fn expand_path(path: &Path) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if let Some(stripped) = path_str.strip_prefix("~/") {
            if let Some(home) = home::home_dir() {
                return home.join(stripped);
            }
        }
    }
    path.to_path_buf()
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} <{}>", self.id, self.name, self.email)
    }
}
