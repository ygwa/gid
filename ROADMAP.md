# gid - Git Identity Manager Roadmap

## 🎯 项目愿景

将 `gid` 打造成 **Git 身份管理的完整解决方案**，解决开发者在多账号、多身份场景下的所有痛点，成为开发者必备的命令行工具。

---

## 🔥 核心痛点分析

### 痛点 1：SSH 密钥切换繁琐 ⭐⭐⭐⭐⭐
- 多账号 SSH 配置复杂（~/.ssh/config）
- 克隆时要记得用对应的 Host
- push/pull 失败时难以排查
- 新项目每次都要手动配置

### 痛点 2：忘记切换导致提交错误 ⭐⭐⭐⭐⭐
- 提交后才发现用错身份
- 修改历史提交很麻烦
- 公司合规要求用公司邮箱
- 开源项目不想暴露真实邮箱

### 痛点 3：无法基于远程仓库自动判断 ⭐⭐⭐⭐
- Git 的 `includeIf` 只能基于本地路径
- 无法根据 remote URL 自动切换身份
- 从不同组织克隆的项目可能在任意目录

### 痛点 4：GPG 签名密钥联动缺失 ⭐⭐⭐⭐
- GitHub/GitLab 越来越多要求签名提交
- 不同身份对应不同 GPG 密钥
- 配置繁琐，容易忘记切换

### 痛点 5：提交历史审计困难 ⭐⭐⭐
- 不知道哪些提交用了错误的身份
- 难以找出所有用错身份的提交

### 痛点 6：新设备配置耗时 ⭐⭐⭐
- 换电脑要重新配置所有身份和规则
- 新团队成员配置成本高

---

## 📦 版本规划

### v1.0.0 - 基础版本 ✅ 已完成
**目标：完善核心功能，建立工程基础**

- [x] 基本的身份切换功能（项目级/全局级）
- [x] 身份列表管理（增删查）
- [x] 配置导入/导出
- [x] 彩色终端输出
- [x] 版本号管理 (`--version`)
- [x] Shell 补全脚本 (bash/zsh/fish)
- [x] 安装/卸载脚本
- [x] CI/CD 自动化
- [x] 完善的项目文档

---

### v1.1.0 - 智能规则引擎 🎯 下一版本
**目标：解决"忘记切换"这个最痛的问题**

**核心功能：**
- [ ] 基于路径的规则匹配
  ```bash
  gid rule add --path "~/work/**" --identity work
  gid rule add --path "~/personal/**" --identity personal
  ```
- [ ] 基于 remote URL 的规则匹配
  ```bash
  gid rule add --remote "github.com/my-company/*" --identity work
  gid rule add --remote "gitlab.company.com/*" --identity work
  ```
- [ ] 自动检测并提示
  ```bash
  gid doctor              # 检查当前目录应该用什么身份
  gid auto                # 根据规则自动设置身份
  ```
- [ ] 项目级配置文件 `.gid`
  ```yaml
  # .gid (项目根目录)
  identity: work
  ```

**辅助功能：**
- [ ] 交互式身份选择器（fzf 风格）
- [ ] 规则优先级管理
- [ ] 规则测试/调试命令

---

### v1.2.0 - Git Hooks 集成
**目标：提交前检查，防患于未然**

**核心功能：**
- [ ] pre-commit hook 自动检查身份
  ```bash
  gid hook install        # 安装 hook 到当前仓库
  gid hook install -g     # 安装全局 hook
  ```
- [ ] 身份不匹配时阻止提交
- [ ] 支持 bypass 选项
  ```bash
  git commit --no-verify  # 跳过检查
  GID_SKIP=1 git commit   # 环境变量跳过
  ```
- [ ] 与 husky/pre-commit 框架集成

**提交修复工具：**
- [ ] 修改最近一次提交的身份
  ```bash
  gid fix-commit          # 修改 HEAD 的 author
  gid fix-commit HEAD~3   # 修改指定提交
  ```

---

### v1.3.0 - SSH 密钥联动
**目标：真正的一站式身份管理**

**核心功能：**
- [ ] 身份配置增加 SSH 密钥
  ```bash
  gid add work \
    --name "张三" \
    --email "zs@company.com" \
    --ssh-key ~/.ssh/id_work
  ```
- [ ] 切换身份时自动配置 SSH
- [ ] 自动管理 ~/.ssh/config
- [ ] SSH 密钥生成向导
  ```bash
  gid ssh-keygen work     # 为 work 身份生成新密钥
  ```

**高级功能：**
- [ ] SSH agent 管理
- [ ] 智能克隆命令
  ```bash
  gid clone git@github.com:company/repo.git
  # 自动检测应使用的身份和 SSH key
  ```

---

### v1.4.0 - GPG 签名联动
**目标：支持 verified commits**

**核心功能：**
- [ ] 身份配置增加 GPG 密钥
  ```bash
  gid add work \
    --name "张三" \
    --email "zs@company.com" \
    --gpg-key ABCD1234
  ```
- [ ] 切换身份时自动设置签名密钥
- [ ] 自动启用/禁用签名
- [ ] GPG 密钥生成向导

---

### v1.5.0 - 审计与报告
**目标：发现并修复身份问题**

**核心功能：**
- [ ] 仓库提交身份审计
  ```bash
  gid audit               # 审计当前仓库
  gid audit --path ~/work # 审计多个仓库
  gid audit --fix         # 交互式修复
  ```
- [ ] 生成审计报告
- [ ] 与 CI 集成（检查 PR 中的提交身份）

---

### v2.0.0 - Rust 重写版本 🦀
**目标：性能提升，跨平台原生支持**

**为什么考虑 Rust：**
- 原生二进制，无需 Bash 依赖
- Windows 原生支持（不需要 WSL/Git Bash）
- 更好的性能（大量规则匹配场景）
- 类型安全，更易维护
- 更丰富的生态（CLI 框架、配置解析等）

**重写范围：**
- [ ] 核心功能完全兼容
- [ ] 配置文件格式升级（YAML/TOML）
- [ ] 插件系统
- [ ] TUI 界面（可选）

详细技术评估见下方 [Rust 重写评估](#rust-重写评估)

---

### v2.1.0+ - 未来展望

**团队协作：**
- [ ] 配置模板市场
- [ ] 团队统一配置同步
- [ ] 组织级策略管理

**平台集成：**
- [ ] GitHub CLI 集成
- [ ] VS Code 扩展
- [ ] JetBrains 插件

**高级功能：**
- [ ] 身份使用统计
- [ ] 时间/日期规则（工作时间用工作身份）
- [ ] 网络位置规则（公司网络用工作身份）

---

## 🦀 Rust 重写评估

### 技术对比

| 方面 | Bash | Rust |
|------|------|------|
| **启动速度** | ~50ms | ~5ms |
| **跨平台** | 需要 Bash 环境 | 原生支持所有平台 |
| **Windows 支持** | 需要 WSL/Git Bash | ✅ 原生 |
| **依赖** | Git, Bash 4.0+ | 无（静态编译） |
| **安装大小** | ~20KB | ~2-5MB |
| **开发复杂度** | 低 | 中等 |
| **可维护性** | 中等 | 高（类型系统） |
| **生态系统** | 有限 | 丰富（clap, serde 等） |
| **错误处理** | 弱 | 强 |
| **并发能力** | 弱 | 强 |

### Rust 版本技术栈推荐

```toml
# Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }  # CLI 框架
serde = { version = "1", features = ["derive"] } # 序列化
toml = "0.8"                                      # 配置文件
directories = "5"                                 # 跨平台路径
colored = "2"                                     # 终端颜色
dialoguer = "0.11"                               # 交互式提示
indicatif = "0.17"                               # 进度条
git2 = "0.18"                                    # Git 操作
glob = "0.3"                                     # 路径匹配
thiserror = "1"                                  # 错误处理
```

### 迁移建议

| 阶段 | 版本 | 语言 | 说明 |
|------|------|------|------|
| **验证期** | v1.0 - v1.2 | Bash | 快速迭代，验证功能需求 |
| **增长期** | v1.3 - v1.5 | Bash | 完善功能，积累用户 |
| **重写期** | v2.0 | Rust | 用户量达到一定规模后重写 |

### 重写时机建议

**适合重写的信号：**
- ✅ GitHub Stars > 500
- ✅ 功能需求已稳定
- ✅ Windows 用户需求强烈
- ✅ 性能成为瓶颈（大量规则匹配）
- ✅ 需要更复杂的功能（插件系统等）

**不建议过早重写：**
- ❌ 功能还在快速变化
- ❌ 用户量很少，需求不明确
- ❌ 维护成本会显著增加

---

## 📁 目标工程结构

### v1.x (Bash)
```
gid/
├── bin/gid                    # 主脚本
├── lib/                       # 模块化脚本
│   ├── core.sh               # 核心函数
│   ├── rules.sh              # 规则引擎
│   ├── ssh.sh                # SSH 管理
│   └── hooks.sh              # Git hooks
├── completions/              # Shell 补全
├── hooks/                    # Git hook 模板
├── scripts/                  # 安装脚本
└── tests/                    # 测试
```

### v2.x (Rust)
```
gid/
├── src/
│   ├── main.rs
│   ├── cli/                  # CLI 定义
│   ├── config/               # 配置管理
│   ├── identity/             # 身份管理
│   ├── rules/                # 规则引擎
│   ├── ssh/                  # SSH 管理
│   ├── gpg/                  # GPG 管理
│   ├── hooks/                # Git hooks
│   └── audit/                # 审计功能
├── completions/              # Shell 补全（构建时生成）
├── Cargo.toml
└── tests/
```

---

## 🚀 发布渠道规划

### 阶段一：基础发布 (v1.0)
- [x] GitHub Releases
- [x] 一键安装脚本

### 阶段二：包管理器 (v1.1+)
- [ ] Homebrew (macOS/Linux)
- [ ] AUR (Arch Linux)
- [ ] Scoop (Windows) - Rust 版本后

### 阶段三：扩展发布 (v2.0+)
- [ ] crates.io (Rust)
- [ ] apt/deb (Debian/Ubuntu)
- [ ] rpm (Fedora/RHEL)
- [ ] Docker 镜像
- [ ] Nix 包

---

## 📋 近期任务清单 (v1.1.0)

### 核心功能
- [ ] 设计规则配置文件格式
- [ ] 实现路径匹配规则
- [ ] 实现 remote URL 匹配规则
- [ ] 实现 `gid doctor` 命令
- [ ] 实现 `gid auto` 命令
- [ ] 实现 `.gid` 项目配置文件

### 工程优化
- [ ] 脚本模块化拆分
- [ ] 增加更多测试用例
- [ ] 性能基准测试

### 文档
- [ ] 规则配置文档
- [ ] 最佳实践指南
- [ ] 迁移指南

---

## 🎯 成功指标

| 里程碑 | 目标 | 指标 |
|--------|------|------|
| v1.0 发布 | 项目上线 | 完成基础功能 |
| 早期采用 | 获得反馈 | 50 Stars, 10 Issues |
| 产品验证 | 功能验证 | 200 Stars, 功能稳定 |
| 规模增长 | 用户增长 | 500 Stars, Homebrew 收录 |
| Rust 重写 | 平台扩展 | 1000 Stars, Windows 支持 |

---

## 📅 时间规划

| 版本 | 预计时间 | 状态 |
|------|----------|------|
| v1.0.0 | Week 1 | ✅ 已完成 |
| v1.1.0 | Week 3-4 | 🎯 进行中 |
| v1.2.0 | Month 2 | ⏳ 待开始 |
| v1.3.0 | Month 3 | ⏳ 待开始 |
| v1.4.0 | Month 4 | ⏳ 待开始 |
| v1.5.0 | Month 5 | ⏳ 待开始 |
| v2.0.0 | Month 6-8 | ⏳ 待评估 |
