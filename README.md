# gid - Git Identity Manager

<p align="center">
  <strong>🔄 Git 身份管理的完整解决方案</strong>
</p>

<p align="center">
  <a href="#特性">特性</a> •
  <a href="#安装">安装</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#使用方法">使用方法</a> •
  <a href="#配置">配置</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg" alt="Platform">
</p>

---

## ✨ 特性

- 🚀 **一键切换** - 快速在多个 Git 身份之间切换
- 📋 **智能规则** - 基于路径或 Remote URL 自动匹配身份
- 🔑 **SSH 联动** - 自动配置 SSH 密钥
- 🔏 **GPG 签名** - 支持提交签名密钥管理
- 🪝 **Git Hooks** - 提交前自动检查身份
- 📊 **审计功能** - 检查历史提交中的身份问题
- 🌍 **跨平台** - 原生支持 Linux、macOS 和 Windows
- ⚡ **高性能** - Rust 编写，启动速度极快

## 📦 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/your-username/gid.git
cd gid

# 编译安装
cargo install --path .

# 或者使用 release 模式
cargo build --release
sudo cp target/release/gid /usr/local/bin/
```

### Homebrew (即将支持)

```bash
brew install your-username/tap/gid
```

### 下载预编译二进制

从 [Releases](https://github.com/your-username/gid/releases) 页面下载对应平台的二进制文件。

## 🚀 快速开始

### 1. 添加身份

```bash
# 交互式添加
gid add

# 或直接指定参数
gid add --id work --name "张三" --email "zhangsan@company.com"
```

### 2. 切换身份

```bash
# 切换当前项目的身份
gid switch work

# 切换全局身份
gid switch -g personal
```

### 3. 设置规则（自动切换）

```bash
# 添加路径规则
gid rule add -t path -p "~/work/**" -i work

# 添加 Remote URL 规则
gid rule add -t remote -p "github.com/my-company/*" -i work

# 自动应用规则
gid auto
```

### 4. 安装 Git Hook

```bash
# 安装到当前仓库
gid hook install

# 或全局安装
gid hook install -g
```

## 📖 使用方法

```
gid - Git Identity Manager

用法: gid <命令>

命令:
  switch       切换到指定身份
  list         列出所有身份
  current      显示当前身份
  add          添加新身份
  remove       删除身份
  edit         编辑配置文件
  export       导出配置
  import       导入配置
  rule         管理规则
  doctor       检查身份配置问题
  auto         根据规则自动切换
  hook         管理 Git hooks
  audit        审计提交历史
  completions  生成 Shell 补全脚本
  help         显示帮助

选项:
  -h, --help     显示帮助
  -V, --version  显示版本
```

### 身份管理

```bash
# 列出所有身份
gid list

# 查看当前身份
gid current

# 添加身份（交互式）
gid add

# 添加身份（带 SSH 和 GPG）
gid add --id work \
  --name "张三" \
  --email "zhangsan@company.com" \
  --ssh-key ~/.ssh/id_work \
  --gpg-key ABCD1234

# 删除身份
gid remove work
```

### 规则管理

```bash
# 添加路径规则
gid rule add -t path -p "~/work/**" -i work

# 添加 Remote URL 规则
gid rule add -t remote -p "github.com/company/*" -i work

# 列出所有规则
gid rule list

# 测试规则匹配
gid rule test

# 删除规则
gid rule remove 0
```

### 检查与自动切换

```bash
# 检查当前目录的身份配置
gid doctor

# 自动修复
gid doctor --fix

# 根据规则自动切换
gid auto
```

### Git Hooks

```bash
# 安装 pre-commit hook（当前仓库）
gid hook install

# 安装全局 hook
gid hook install -g

# 查看 hook 状态
gid hook status

# 卸载 hook
gid hook uninstall
```

### 审计

```bash
# 审计当前仓库
gid audit

# 审计指定目录
gid audit --path ~/projects
```

## ⚙️ 配置

### 配置文件位置

- Linux/macOS: `~/.config/gid/config.toml`
- Windows: `%APPDATA%\gid\config\config.toml`

可通过 `GID_CONFIG_DIR` 环境变量自定义。

### 配置文件格式

```toml
# 身份列表
[[identities]]
id = "work"
name = "张三"
email = "zhangsan@company.com"
description = "工作身份"
ssh_key = "~/.ssh/id_work"
gpg_key = "ABCD1234"
gpg_sign = true

[[identities]]
id = "personal"
name = "张三"
email = "zhangsan@gmail.com"
description = "个人身份"

# 规则列表
[[rules]]
type = "path"
pattern = "~/work/**"
identity = "work"
priority = 100

[[rules]]
type = "remote"
pattern = "github.com/my-company/*"
identity = "work"
priority = 50

# 设置
[settings]
verbose = true
color = true
auto_switch = false
pre_commit_check = true
strict_mode = false
```

### 项目配置 (.gid)

在项目根目录创建 `.gid` 文件指定默认身份：

```
work
```

## 🐚 Shell 补全

```bash
# Bash
gid completions bash > /etc/bash_completion.d/gid

# Zsh
gid completions zsh > /usr/local/share/zsh/site-functions/_gid

# Fish
gid completions fish > ~/.config/fish/completions/gid.fish

# PowerShell
gid completions powershell > gid.ps1
```

## 🔧 开发

### 构建

```bash
# Debug 模式
cargo build

# Release 模式
cargo build --release

# 运行测试
cargo test
```

### 目录结构

```
src/
├── main.rs           # 入口
├── cli.rs            # CLI 定义
├── commands/         # 命令实现
├── config/           # 配置管理
├── rules/            # 规则引擎
├── git/              # Git 操作
├── ssh/              # SSH 管理
├── gpg/              # GPG 管理
└── audit/            # 审计功能
```

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 📄 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件。

---

<p align="center">
  如果这个工具对你有帮助，请给一个 ⭐️
</p>
