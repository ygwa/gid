# 贡献指南

感谢你有兴趣为 gid 做出贡献！🎉

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发环境设置](#开发环境设置)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)

## 行为准则

请保持友善和尊重。我们希望 gid 社区对所有人都是友好和包容的。

## 如何贡献

### 报告 Bug

1. 在 [Issues](https://github.com/your-username/gid/issues) 中搜索是否已存在相关问题
2. 如果没有，创建一个新的 Issue
3. 使用清晰的标题描述问题
4. 提供以下信息：
   - 操作系统版本
   - gid 版本 (`gid --version`)
   - 重现步骤
   - 期望行为
   - 实际行为
   - 相关日志或截图

### 功能建议

1. 在 Issues 中搜索是否已有类似建议
2. 创建新 Issue，标题以 `[Feature]` 开头
3. 清晰描述功能需求和使用场景

### 代码贡献

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 开发环境设置

### 前置要求

- Bash 4.0+
- Git
- [bats-core](https://github.com/bats-core/bats-core) (用于测试)

### 设置步骤

```bash
# 克隆你的 Fork
git clone https://github.com/YOUR_USERNAME/gid.git
cd gid

# 添加上游仓库
git remote add upstream https://github.com/your-username/gid.git

# 安装开发依赖（可选，用于测试）
# macOS
brew install bats-core

# Linux (Ubuntu/Debian)
sudo apt-get install bats
```

### 本地测试

```bash
# 运行测试
make test

# 或直接运行
bats tests/

# 测试脚本功能
./bin/gid --help
./bin/gid --version
```

## 代码规范

### Shell 脚本规范

1. **使用 `set -e`** - 遇到错误立即退出
2. **变量引用使用双引号** - `"$variable"` 而非 `$variable`
3. **函数命名使用下划线** - `my_function` 而非 `myFunction`
4. **添加注释** - 为复杂逻辑添加说明
5. **检查依赖** - 使用外部命令前检查是否存在

### 代码风格

```bash
# ✅ 好的风格
my_function() {
    local variable="$1"
    
    if [ -z "$variable" ]; then
        echo "Error: variable is empty" >&2
        return 1
    fi
    
    echo "$variable"
}

# ❌ 避免的风格
myFunction() {
    variable=$1
    if [ -z $variable ]; then
        echo Error: variable is empty
        return 1
    fi
    echo $variable
}
```

### 使用 ShellCheck

在提交前运行 ShellCheck 检查代码：

```bash
shellcheck bin/gid
```

## 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 格式

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### 类型 (Type)

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行）
- `refactor`: 重构（既不是新功能也不是修复）
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

### 示例

```bash
feat(identity): add export/import functionality

fix(switch): handle spaces in identity names

docs(readme): update installation instructions

chore(ci): add GitHub Actions workflow
```

## Pull Request 流程

1. **确保测试通过**
   ```bash
   make test
   ```

2. **更新文档**
   - 如果添加了新功能，更新 README.md
   - 更新 CHANGELOG.md

3. **PR 描述模板**
   ```markdown
   ## 描述
   简要描述这个 PR 做了什么
   
   ## 变更类型
   - [ ] Bug 修复
   - [ ] 新功能
   - [ ] 文档更新
   - [ ] 重构
   
   ## 测试
   描述如何测试这些变更
   
   ## 相关 Issue
   Fixes #123
   ```

4. **代码审查**
   - 等待维护者审查
   - 根据反馈进行修改
   - 保持 PR 小而专注

## 发布流程

发布由维护者执行：

1. 更新 CHANGELOG.md
2. 更新版本号 (`bin/gid` 中的 `VERSION`)
3. 创建 Git tag
4. 推送 tag 触发 CI 发布

## 获取帮助

如果你有任何问题：

- 查看 [README.md](README.md)
- 搜索 [Issues](https://github.com/your-username/gid/issues)
- 创建新 Issue 询问

---

再次感谢你的贡献！ 🙌

