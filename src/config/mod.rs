pub mod identity;
pub mod settings;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub use identity::Identity;
pub use settings::Settings;

use crate::rules::Rule;

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// 身份列表
    #[serde(default)]
    pub identities: Vec<Identity>,
    
    /// 规则列表
    #[serde(default)]
    pub rules: Vec<Rule>,
    
    /// 设置
    #[serde(default)]
    pub settings: Settings,
}

impl Config {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = if let Ok(path) = std::env::var("GID_CONFIG_DIR") {
            PathBuf::from(path)
        } else if let Some(config_dir) = directories::ProjectDirs::from("com", "gid", "gid") {
            config_dir.config_dir().to_path_buf()
        } else {
            let home = home::home_dir().context("无法获取用户主目录")?;
            home.join(".config").join("gid")
        };
        
        Ok(config_dir.join("config.toml"))
    }
    
    /// 加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取配置文件: {}", config_path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| "配置文件格式错误")?;
        
        Ok(config)
    }
    
    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("无法序列化配置")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("无法写入配置文件: {}", config_path.display()))?;
        
        Ok(())
    }
    
    /// 查找身份
    pub fn find_identity(&self, id: &str) -> Option<&Identity> {
        self.identities.iter().find(|i| i.id == id)
    }
    
    /// 查找身份（可变引用）
    pub fn find_identity_mut(&mut self, id: &str) -> Option<&mut Identity> {
        self.identities.iter_mut().find(|i| i.id == id)
    }
    
    /// 添加身份
    pub fn add_identity(&mut self, identity: Identity) -> Result<()> {
        if self.find_identity(&identity.id).is_some() {
            anyhow::bail!("身份 '{}' 已存在", identity.id);
        }
        self.identities.push(identity);
        Ok(())
    }
    
    /// 删除身份
    pub fn remove_identity(&mut self, id: &str) -> Result<Identity> {
        let index = self.identities.iter()
            .position(|i| i.id == id)
            .ok_or_else(|| anyhow::anyhow!("找不到身份 '{}'", id))?;
        
        Ok(self.identities.remove(index))
    }
    
    /// 添加规则
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        // 按优先级排序
        self.rules.sort_by_key(|r| r.priority);
    }
    
    /// 删除规则
    pub fn remove_rule(&mut self, index: usize) -> Result<Rule> {
        if index >= self.rules.len() {
            anyhow::bail!("规则索引 {} 超出范围", index);
        }
        Ok(self.rules.remove(index))
    }
}

