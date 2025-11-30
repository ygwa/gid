# gid - Git Identity Manager

<p align="center">
  <strong>🔄 A complete solution for managing multiple Git identities</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#usage">Usage</a> •
  <a href="#configuration">Configuration</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg" alt="Platform">
</p>

---

## ✨ Features

- 🚀 **One-click Switch** - Quickly switch between multiple Git identities
- 📋 **Smart Rules** - Automatically match identities based on path or remote URL
- 🔑 **SSH Integration** - Automatically configure SSH keys
- 🔏 **GPG Signing** - Support commit signing key management
- 🪝 **Git Hooks** - Automatically check identity before commit
- 📊 **Audit** - Check for identity issues in commit history
- 🌍 **Cross-platform** - Native support for Linux, macOS, and Windows
- ⚡ **High Performance** - Written in Rust, extremely fast startup

## 📦 Installation

### Build from Source

```bash
# Clone repository
git clone https://github.com/ygwa/gid.git
cd gid

# Install
cargo install --path .

# Or build release
cargo build --release
sudo cp target/release/gid /usr/local/bin/
```

### Homebrew (Coming Soon)

```bash
brew install ygwa/tap/gid
```

### Download Binary

Download binaries for your platform from the [Releases](https://github.com/ygwa/gid/releases) page.

## 🚀 Quick Start

### 1. Add Identity

```bash
# Interactive add
gid add

# Or specify arguments
gid add --id work --name "John Doe" --email "john@company.com"
```

### 2. Switch Identity

```bash
# Switch identity for current project
gid switch work

# Switch global identity
gid switch -g personal
```

### 3. Set Rules (Auto Switch)

```bash
# Add path rule
gid rule add -t path -p "~/work/**" -i work

# Add remote URL rule
gid rule add -t remote -p "github.com/my-company/*" -i work

# Apply rules automatically
gid auto
```

### 4. Install Git Hook

```bash
# Install to current repository
gid hook install

# Or install globally
gid hook install -g
```

## 📖 Usage

```
gid - Git Identity Manager

Usage: gid <COMMAND>

Commands:
  switch       Switch to specified identity
  list         List all identities
  current      Show current identity
  add          Add a new identity
  remove       Remove an identity
  edit         Edit configuration file
  export       Export configuration
  import       Import configuration
  rule         Manage rules
  doctor       Check identity configuration issues
  auto         Automatically switch identity based on rules
  hook         Manage Git hooks
  audit        Audit commit history
  completions  Generate shell completion scripts
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Identity Management

```bash
# List all identities
gid list

# Show current identity
gid current

# Add identity (interactive)
gid add

# Add identity (with SSH and GPG)
gid add --id work \
  --name "John Doe" \
  --email "john@company.com" \
  --ssh-key ~/.ssh/id_work \
  --gpg-key ABCD1234

# Remove identity
gid remove work
```

### Rule Management

```bash
# Add path rule
gid rule add -t path -p "~/work/**" -i work

# Add remote URL rule
gid rule add -t remote -p "github.com/company/*" -i work

# List all rules
gid rule list

# Test rule matching
gid rule test

# Remove rule
gid rule remove 0
```

### Check and Auto Switch

```bash
# Check identity configuration in current directory
gid doctor

# Auto fix
gid doctor --fix

# Auto switch based on rules
gid auto
```

### Git Hooks

```bash
# Install pre-commit hook (current repo)
gid hook install

# Install global hook
gid hook install -g

# Check hook status
gid hook status

# Uninstall hook
gid hook uninstall
```

### Audit

```bash
# Audit current repository
gid audit

# Audit specified directory
gid audit --path ~/projects
```

## ⚙️ Configuration

### Configuration File Location

- Linux/macOS: `~/.config/gid/config.toml`
- Windows: `%APPDATA%\gid\config\config.toml`

Can be customized via `GID_CONFIG_DIR` environment variable.

### Configuration Format

```toml
# Identity List
[[identities]]
id = "work"
name = "John Doe"
email = "john@company.com"
description = "Work Identity"
ssh_key = "~/.ssh/id_work"
gpg_key = "ABCD1234"
gpg_sign = true

[[identities]]
id = "personal"
name = "John Doe"
email = "john@gmail.com"
description = "Personal Identity"

# Rule List
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

# Settings
[settings]
verbose = true
color = true
auto_switch = false
pre_commit_check = true
strict_mode = false
```

### Project Config (.gid)

Create a `.gid` file in the project root to specify the default identity:

```
work
```

## 🐚 Shell Completion

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

## 🔧 Development

### Build

```bash
# Debug mode
cargo build

# Release mode
cargo build --release

# Run tests
cargo test
```

### Directory Structure

```
src/
├── main.rs           # Entry point
├── cli.rs            # CLI definition
├── commands/         # Command implementations
├── config/           # Configuration management
├── rules/            # Rule engine
├── git/              # Git operations
├── ssh/              # SSH management
├── gpg/              # GPG management
└── audit/            # Audit functionality
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

---

<p align="center">
  If this tool helps you, please give it a ⭐️
</p>
