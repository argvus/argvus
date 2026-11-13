---
title: 安装主题
description: 使用 pacman 安装官方 ARGVUS 主题包。
---
官方主题作为 Arch Linux 软件包发布。每个主题都是名为 `argvus-theme-<THEME_NAME>` 的独立软件包。

## 获取主题的位置

每个官方主题的源代码都存在于 ARGVUS 项目组织中：

- 每个主题都是一个名为 `argvus-theme-<THEME_NAME>` 的仓库（例如 `argvus-theme-dracula`），并以相同名称构建为软件包。

主题软件包依赖于 `argvus-appearance`。请先按照 [安装](/docs/getting-started/installation/) 的说明安装 ARGVUS，以确保外观组件存在。

## 安装单个主题

要一次性安装所有官方主题，请安装 `argvus-themes` 元软件包：

```bash
sudo pacman -S argvus-themes
```
这取决于所有已发布的 `argvus-theme-*` 软件包。ARGVUS Dark 和 ARGVUS Light 已由 `argvus-appearance` 发货。

使用其软件包名称安装一个主题：

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
例如：

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
该软件包会安装主题清单 `/usr/share/argvus/themes/<theme-id>/theme.toml`（其名称、类别、壁纸和调色板，可选 `overrides/`），以及主题的壁纸到 `/usr/share/backgrounds/argvus/`。当主题应用时，每个组件的文件都会根据调色板呈现。它不会写入你的主目录，也不会更改你当前的主题。之后可在控制中心中选择它，参见 [官方主题](/docs/components/argvus-themes/official-themes/)。

## 安装多个主题

没有一个元软件包可以安装所有主题。使用一条命令安装你想要的主题：

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## 可用软件包

以下软件包是今天发布的官方主题包。

**深色主题**

| 软件包 | 主题 |
| --- | --- |
| `argvus-theme-one-dark` |一暗 |
| `argvus-theme-dracula` |德古拉 |
| `argvus-theme-silver-dark` |银色深色|
| `argvus-theme-slate-dark` |板岩色 |
| `argvus-theme-universe` |宇宙|
| `argvus-theme-gruvbox-high-dark` | Gruvbox 高暗 |
| `argvus-theme-gruvbox-dark` | Gruvbox 深色 |
| `argvus-theme-rose-pine` |玫瑰松 |
| `argvus-theme-tokyo-night` |东京之夜 |
| `argvus-theme-solitude` |孤独|
| `argvus-theme-sunset` |日落|
| `argvus-theme-matrix` |矩阵|
| `argvus-theme-monokai-dark` |莫诺凯黑暗|

**浅色主题**

| 软件包 | 主题 |
| --- | --- |
| `argvus-theme-github-light` | GitHub 灯 |
| `argvus-theme-solarized-light` |日光灯|
| `argvus-theme-nord-light` |北欧之光 |
| `argvus-theme-one-light` |一盏灯|
| `argvus-theme-everforest-light` |永林之光 |
| `argvus-theme-catppuccin-latte` |卡布奇诺拿铁|
| `argvus-theme-gruvbox-light` | Gruvbox 灯 |
| `argvus-theme-watercolor` |水彩|

内置的 **ARGVUS Dark** 和 **ARGVUS Light** 主题不需要单独的软件包；它们随 `argvus-appearance` 一起提供。

## 检查已安装内容

使用以下命令列出已安装的主题软件包：

```bash
pacman -Qs argvus-theme
```
打开控制中心，进入 **外观 → 主题 → 官方主题** 查看当前选择器中可用的主题。已安装但在该列表中缺失的主题是打包问题；请参阅 [主题指南](/docs/components/argvus-themes/) 的故障排除部分。

## 添加自己的主题

主题是一个清单文件，所以你自己的主题不需要打包。创建 `~/.local/share/argvus/themes/<id>/theme.toml`，其中目录名称为主题 ID（小写字母、数字和单个连字符）:

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
只有 `bg`、`fg` 和 `accent` 是必需的：每种其他颜色都是从它们派生的，清单中只定义了一个以便更改它。检查它，然后像使用其他主题一样应用它：

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list` 会在官方主题旁显示你的主题（具有相同 ID 的官方主题优先）。你的主题会出现在主题菜单中（`SUPER + Shift + T`）；登录前的欢迎界面无法读取你的主目录，因此在你的主题处于激活状态时，它会保留你最后应用的官方主题。

旧版本 ARGVUS 在 `~/.config/argvus/data/<app>/themes/` 下写入的每个主题副本将不再被读取。更新后的第一次主题切换会将它们移动到 `~/.config/argvus/data/backups/theme-copies-<time>/`；
如果想保留你编辑的文件，请将其放入你自己的主题中的 `overrides/`，并在清单的 `[overrides]` 表中命名。

## 删除主题

使用以下方法删除单个主题包：

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
移除主题包不会删除您导入的配置文件。如果被移除的主题是当前应用的主题，请在移除之前从控制中心选择另一个主题。

## 从 Git 安装主题

通过 HTTPS 或 SSH 安装基于 skeleton 的仓库，使用已有 Git 凭据。主题按用户安装，不会自动应用。`--ref` 选择分支或标签；`--replace` 替换已有自定义主题。已安装的官方 ID 受到保护。

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

主题显示在 **Custom themes** 中；按 `r` 刷新页面。通过 CLI 或 Control Center 删除当前主题前，先应用其他主题。替换后需重新应用。

仓库必须在 `src/usr/share/argvus/themes/<id>/` 中包含且仅包含一个 schema 2 主题，ID 与目录一致。图片必须提交到主题目录或 `src/usr/share/` 中。不执行脚本，不支持子模块或 Git LFS。需要 `git`，SSH 还需要 `openssh`。目标目录为 `$XDG_DATA_HOME/argvus/themes/<id>`。官方软件包仍由 pacman 管理。
