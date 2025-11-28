# gid - Git Identity Manager
# Makefile

.PHONY: all build release install uninstall test lint fmt clean help

# 变量
BINARY_NAME := gid
CARGO := cargo
PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin

# 获取版本号
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

all: build

## help: 显示帮助
help:
	@echo "gid - Git Identity Manager v$(VERSION)"
	@echo ""
	@echo "用法: make <target>"
	@echo ""
	@echo "Targets:"
	@echo "  build     编译 debug 版本"
	@echo "  release   编译 release 版本"
	@echo "  install   安装到系统"
	@echo "  uninstall 卸载"
	@echo "  test      运行测试"
	@echo "  lint      代码检查"
	@echo "  fmt       格式化代码"
	@echo "  clean     清理构建文件"
	@echo ""
	@echo "变量:"
	@echo "  PREFIX=$(PREFIX)"
	@echo "  BINDIR=$(BINDIR)"

## build: 编译 debug 版本
build:
	$(CARGO) build

## release: 编译 release 版本
release:
	$(CARGO) build --release

## install: 安装到系统
install: release
	@echo "安装 $(BINARY_NAME) 到 $(BINDIR)..."
	@mkdir -p $(BINDIR)
	@cp target/release/$(BINARY_NAME) $(BINDIR)/$(BINARY_NAME)
	@chmod +x $(BINDIR)/$(BINARY_NAME)
	@echo "✓ 安装完成!"
	@echo ""
	@echo "运行 '$(BINARY_NAME) --help' 开始使用"

## uninstall: 卸载
uninstall:
	@echo "卸载 $(BINARY_NAME)..."
	@rm -f $(BINDIR)/$(BINARY_NAME)
	@echo "✓ 卸载完成"
	@echo ""
	@echo "配置文件保留在 ~/.config/gid/"
	@echo "如需删除，请手动执行: rm -rf ~/.config/gid"

## test: 运行测试
test:
	$(CARGO) test

## lint: 代码检查
lint:
	$(CARGO) clippy -- -D warnings

## fmt: 格式化代码
fmt:
	$(CARGO) fmt

## fmt-check: 检查格式
fmt-check:
	$(CARGO) fmt -- --check

## clean: 清理
clean:
	$(CARGO) clean
	@echo "✓ 清理完成"

## completions: 生成补全脚本
completions: release
	@mkdir -p completions
	@./target/release/$(BINARY_NAME) completions bash > completions/$(BINARY_NAME).bash
	@./target/release/$(BINARY_NAME) completions zsh > completions/_$(BINARY_NAME)
	@./target/release/$(BINARY_NAME) completions fish > completions/$(BINARY_NAME).fish
	@echo "✓ 补全脚本已生成到 completions/"
