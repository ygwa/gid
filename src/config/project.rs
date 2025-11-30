use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::rules::Rule;

/// Project configuration (.gid file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Default identity ID
    pub identity: String,

    /// Project specific rules (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

impl ProjectConfig {
    /// Load .gid config from specified directory
    pub fn load_from_dir(path: &Path) -> Result<Option<Self>> {
        let gid_path = path.join(".gid");

        if !gid_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&gid_path)
            .with_context(|| format!("Could not read .gid file: {}", gid_path.display()))?;

        Self::parse(&content)
    }

    /// Find .gid file in parents starting from current directory
    #[allow(dead_code)]
    pub fn find_in_parents(start: &Path) -> Result<Option<(Self, PathBuf)>> {
        let mut current = start.to_path_buf();

        loop {
            if let Some(config) = Self::load_from_dir(&current)? {
                return Ok(Some((config, current.join(".gid"))));
            }

            // Go to parent directory
            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    /// Parse .gid file content
    fn parse(content: &str) -> Result<Option<Self>> {
        let trimmed = content.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        // Try parsing as TOML
        if trimmed.contains('=') || trimmed.contains('[') {
            match toml::from_str::<ProjectConfig>(trimmed) {
                Ok(config) => return Ok(Some(config)),
                Err(e) => {
                    anyhow::bail!(".gid file format error: {e}");
                }
            }
        }

        // Simple format: single line identity ID
        let identity = trimmed.lines().next().unwrap_or("").trim().to_string();

        if identity.is_empty() {
            return Ok(None);
        }

        // Validate identity ID format
        if !identity
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            anyhow::bail!("Invalid identity ID format in .gid file: {identity}");
        }

        Ok(Some(ProjectConfig {
            identity,
            rules: Vec::new(),
        }))
    }

    /// Save to specified directory
    #[allow(dead_code)]
    pub fn save_to_dir(&self, path: &Path) -> Result<()> {
        let gid_path = path.join(".gid");

        // If no rules, use simple format
        if self.rules.is_empty() {
            fs::write(&gid_path, format!("{}\n", self.identity))
                .with_context(|| format!("Could not write .gid file: {}", gid_path.display()))?;
        } else {
            // Use TOML format
            let content = toml::to_string_pretty(self).context("Could not serialize config")?;
            fs::write(&gid_path, content)
                .with_context(|| format!("Could not write .gid file: {}", gid_path.display()))?;
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
