
use glob::Pattern;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RuleType {
    /// 路径匹配规则
    Path { pattern: String },
    /// Remote URL 匹配规则
    Remote { pattern: String },
}

/// 匹配规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// 规则类型和模式
    #[serde(flatten)]
    pub rule_type: RuleType,

    /// 匹配后使用的身份 ID
    pub identity: String,

    /// 规则优先级（数字越小优先级越高）
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// 规则描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_priority() -> u32 {
    100
}

fn default_true() -> bool {
    true
}

impl Rule {
    /// 创建路径规则
    pub fn path(pattern: String, identity: String) -> Self {
        Self {
            rule_type: RuleType::Path { pattern },
            identity,
            priority: default_priority(),
            description: None,
            enabled: true,
        }
    }

    /// 创建 remote URL 规则
    pub fn remote(pattern: String, identity: String) -> Self {
        Self {
            rule_type: RuleType::Remote { pattern },
            identity,
            priority: default_priority(),
            description: None,
            enabled: true,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// 检查是否匹配路径
    pub fn matches_path(&self, path: &Path) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.rule_type {
            RuleType::Path { pattern } => {
                let path_str = path.to_string_lossy();

                // 展开 ~ 符号
                let expanded_pattern = if let Some(stripped) = pattern.strip_prefix("~/") {
                    if let Some(home) = home::home_dir() {
                        format!("{}/{stripped}", home.display())
                    } else {
                        pattern.clone()
                    }
                } else {
                    pattern.clone()
                };

                // 使用 glob 模式匹配
                if let Ok(glob) = Pattern::new(&expanded_pattern) {
                    if glob.matches(&path_str) {
                        return true;
                    }
                }

                // 检查路径是否在模式目录下
                let trimmed = expanded_pattern
                    .trim_end_matches("**")
                    .trim_end_matches('/');
                let pattern_path = Path::new(trimmed);
                if path.starts_with(pattern_path) {
                    return true;
                }

                false
            }
            RuleType::Remote { .. } => false,
        }
    }

    /// 检查是否匹配 remote URL
    pub fn matches_remote(&self, remote_url: &str) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.rule_type {
            RuleType::Remote { pattern } => {
                // 首先尝试精确匹配
                if remote_url.contains(pattern) {
                    return true;
                }

                // 尝试正则匹配
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(remote_url) {
                        return true;
                    }
                }

                // 尝试 glob 模式匹配
                if let Ok(glob) = Pattern::new(pattern) {
                    // 标准化 URL 进行匹配
                    let normalized = normalize_git_url(remote_url);
                    if glob.matches(&normalized) {
                        return true;
                    }
                }

                false
            }
            RuleType::Path { .. } => false,
        }
    }

    /// 获取规则类型名称
    pub fn type_name(&self) -> &'static str {
        match &self.rule_type {
            RuleType::Path { .. } => "path",
            RuleType::Remote { .. } => "remote",
        }
    }

    /// 获取匹配模式
    pub fn pattern(&self) -> &str {
        match &self.rule_type {
            RuleType::Path { pattern } => pattern,
            RuleType::Remote { pattern } => pattern,
        }
    }
}

/// 规则引擎
pub struct RuleEngine<'a> {
    rules: &'a [Rule],
}

impl<'a> RuleEngine<'a> {
    pub fn new(rules: &'a [Rule]) -> Self {
        Self { rules }
    }

    /// 根据上下文匹配规则
    pub fn match_context(&self, context: &MatchContext) -> Option<&'a Rule> {
        // 规则已按优先级排序
        for rule in self.rules {
            if !rule.enabled {
                continue;
            }

            // 优先匹配 remote URL
            if let Some(ref remote) = context.remote_url {
                if rule.matches_remote(remote) {
                    return Some(rule);
                }
            }

            // 匹配路径
            if let Some(ref path) = context.path {
                if rule.matches_path(path) {
                    return Some(rule);
                }
            }
        }

        None
    }

    /// 获取所有匹配的规则
    pub fn match_all(&self, context: &MatchContext) -> Vec<&'a Rule> {
        self.rules
            .iter()
            .filter(|rule| {
                if !rule.enabled {
                    return false;
                }

                if let Some(ref remote) = context.remote_url {
                    if rule.matches_remote(remote) {
                        return true;
                    }
                }

                if let Some(ref path) = context.path {
                    if rule.matches_path(path) {
                        return true;
                    }
                }

                false
            })
            .collect()
    }
}

/// 匹配上下文
#[derive(Debug, Default)]
pub struct MatchContext {
    pub path: Option<std::path::PathBuf>,
    pub remote_url: Option<String>,
}

impl MatchContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: std::path::PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_remote(mut self, remote: String) -> Self {
        self.remote_url = Some(remote);
        self
    }
}

/// 标准化 Git URL
fn normalize_git_url(url: &str) -> String {
    let url = url.trim();

    // git@github.com:user/repo.git -> github.com/user/repo
    if url.starts_with("git@") {
        let url = url.trim_start_matches("git@");
        let url = url.replace(':', "/");
        let url = url.trim_end_matches(".git");
        return url.to_string();
    }

    // https://github.com/user/repo.git -> github.com/user/repo
    if url.starts_with("https://") || url.starts_with("http://") {
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let url = url.trim_end_matches(".git");
        return url.to_string();
    }

    url.to_string()
}



impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} -> {}",
            self.type_name(),
            self.pattern(),
            self.identity
        )
    }
}
