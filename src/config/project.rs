use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::rules::Rule;

/// 项目级配置（.gid 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 默认身份 ID
    pub identity: String,

    /// 项目特定规则（可选）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

impl ProjectConfig {
    /// 从指定目录加载 .gid 配置
    pub fn load_from_dir(path: &Path) -> Result<Option<Self>> {
        let gid_path = path.join(".gid");

        if !gid_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&gid_path)
            .with_context(|| format!("无法读取 .gid 文件: {}", gid_path.display()))?;

        Self::parse(&content)
    }

    /// 从当前目录向上查找 .gid 文件
    pub fn find_in_parents(start: &Path) -> Result<Option<(Self, PathBuf)>> {
        let mut current = start.to_path_buf();

        loop {
            if let Some(config) = Self::load_from_dir(&current)? {
                return Ok(Some((config, current.join(".gid"))));
            }

            // 向上一级目录
            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    /// 解析 .gid 文件内容
    fn parse(content: &str) -> Result<Option<Self>> {
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        // 尝试作为 TOML 解析
        if trimmed.contains('=') || trimmed.contains('[') {
            match toml::from_str::<ProjectConfig>(trimmed) {
                Ok(config) => return Ok(Some(config)),
                Err(e) => {
                    anyhow::bail!(".gid 文件格式错误: {}", e);
                }
            }
        }

        // 简单格式：单行身份 ID
        let identity = trimmed.lines().next().unwrap_or("").trim().to_string();

        if identity.is_empty() {
            return Ok(None);
        }

        // 验证身份 ID 格式
        if !identity
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            anyhow::bail!(".gid 文件中的身份 ID 格式不正确: {}", identity);
        }

        Ok(Some(ProjectConfig {
            identity,
            rules: Vec::new(),
        }))
    }

    /// 保存到指定目录
    pub fn save_to_dir(&self, path: &Path) -> Result<()> {
        let gid_path = path.join(".gid");

        // 如果没有规则，使用简单格式
        if self.rules.is_empty() {
            fs::write(&gid_path, format!("{}\n", self.identity))
                .with_context(|| format!("无法写入 .gid 文件: {}", gid_path.display()))?;
        } else {
            // 使用 TOML 格式
            let content = toml::to_string_pretty(self).context("无法序列化配置")?;
            fs::write(&gid_path, content)
                .with_context(|| format!("无法写入 .gid 文件: {}", gid_path.display()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_format() {
        let content = "work\n";
        let config = ProjectConfig::parse(content).unwrap().unwrap();
        assert_eq!(config.identity, "work");
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_parse_toml_format() {
        let content = r#"
identity = "work"

[[rules]]
type = "path"
pattern = "src/**"
identity = "work"
priority = 100
"#;
        let config = ProjectConfig::parse(content).unwrap().unwrap();
        assert_eq!(config.identity, "work");
        assert_eq!(config.rules.len(), 1);
    }

    #[test]
    fn test_parse_empty() {
        let content = "";
        let config = ProjectConfig::parse(content).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_parse_invalid_id() {
        let content = "invalid id with spaces";
        assert!(ProjectConfig::parse(content).is_err());
    }
}
