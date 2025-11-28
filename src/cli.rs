use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gid",
    about = "Git Identity Manager - 管理多个 Git 身份的完整解决方案",
    version,
    author,
    after_help = "更多信息请访问: https://github.com/your-username/gid"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 切换到指定身份
    #[command(visible_alias = "sw")]
    Switch {
        /// 身份 ID
        identity: String,
        
        /// 全局切换（影响所有仓库）
        #[arg(short, long)]
        global: bool,
    },
    
    /// 列出所有身份
    #[command(visible_alias = "ls")]
    List,
    
    /// 显示当前身份
    #[command(visible_alias = "c")]
    Current,
    
    /// 添加新身份
    Add {
        /// 身份 ID（如：work, personal）
        #[arg(short, long)]
        id: Option<String>,
        
        /// 姓名
        #[arg(short, long)]
        name: Option<String>,
        
        /// 邮箱
        #[arg(short, long)]
        email: Option<String>,
        
        /// 描述
        #[arg(short, long)]
        description: Option<String>,
        
        /// SSH 私钥路径
        #[arg(long)]
        ssh_key: Option<PathBuf>,
        
        /// GPG 密钥 ID
        #[arg(long)]
        gpg_key: Option<String>,
    },
    
    /// 删除身份
    #[command(visible_alias = "rm")]
    Remove {
        /// 要删除的身份 ID
        identity: String,
    },
    
    /// 编辑配置文件
    Edit,
    
    /// 导出配置
    Export {
        /// 导出文件路径
        #[arg(default_value = "gid-config.toml")]
        file: PathBuf,
    },
    
    /// 导入配置
    Import {
        /// 要导入的文件路径
        file: PathBuf,
    },
    
    /// 管理规则
    Rule {
        #[command(subcommand)]
        action: RuleAction,
    },
    
    /// 检查当前目录的身份配置
    Doctor {
        /// 自动修复问题
        #[arg(short, long)]
        fix: bool,
    },
    
    /// 根据规则自动切换身份
    Auto,
    
    /// 管理 Git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
    
    /// 审计提交历史中的身份信息
    Audit {
        /// 要审计的路径（默认当前目录）
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// 尝试修复问题
        #[arg(short, long)]
        fix: bool,
    },
    
    /// 生成 Shell 补全脚本
    Completions {
        /// Shell 类型
        #[arg(value_enum)]
        shell: ShellType,
    },
}

#[derive(Subcommand, Clone)]
pub enum RuleAction {
    /// 添加规则
    Add {
        /// 规则类型
        #[arg(short, long, value_enum)]
        rule_type: RuleType,
        
        /// 匹配模式
        #[arg(short, long)]
        pattern: String,
        
        /// 匹配后使用的身份
        #[arg(short, long)]
        identity: String,
        
        /// 规则优先级（数字越小优先级越高）
        #[arg(long, default_value = "100")]
        priority: u32,
    },
    
    /// 列出所有规则
    List,
    
    /// 删除规则
    Remove {
        /// 规则索引
        index: usize,
    },
    
    /// 测试规则匹配
    Test {
        /// 测试路径
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// 测试 remote URL
        #[arg(short, long)]
        remote: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum HookAction {
    /// 安装 Git hook
    Install {
        /// 全局安装（使用 core.hooksPath）
        #[arg(short, long)]
        global: bool,
    },
    
    /// 卸载 Git hook
    Uninstall {
        /// 全局卸载
        #[arg(short, long)]
        global: bool,
    },
    
    /// 显示 hook 状态
    Status,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RuleType {
    /// 路径匹配规则
    Path,
    /// Remote URL 匹配规则
    Remote,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl From<ShellType> for clap_complete::Shell {
    fn from(shell: ShellType) -> Self {
        match shell {
            ShellType::Bash => clap_complete::Shell::Bash,
            ShellType::Zsh => clap_complete::Shell::Zsh,
            ShellType::Fish => clap_complete::Shell::Fish,
            ShellType::PowerShell => clap_complete::Shell::PowerShell,
        }
    }
}

