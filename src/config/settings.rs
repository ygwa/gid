use serde::{Deserialize, Serialize};

/// 全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// 是否在切换时显示详细信息
    #[serde(default = "default_true")]
    pub verbose: bool,

    /// 是否启用颜色输出
    #[serde(default = "default_true")]
    pub color: bool,

    /// 自动模式：进入目录时是否自动切换身份
    #[serde(default)]
    pub auto_switch: bool,

    /// 是否在提交前检查身份
    #[serde(default = "default_true")]
    pub pre_commit_check: bool,

    /// 身份不匹配时是否阻止提交
    #[serde(default)]
    pub strict_mode: bool,

    /// 默认使用的编辑器
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,

    /// 全局 hooks 目录
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks_path: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            verbose: true,
            color: true,
            auto_switch: false,
            pre_commit_check: true,
            strict_mode: false,
            editor: None,
            hooks_path: None,
        }
    }
}

fn default_true() -> bool {
    true
}
