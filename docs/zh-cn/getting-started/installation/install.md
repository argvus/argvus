---
title: 安装 ARGVUS
description: 安装、更新、删除和配置 ARGVUS。
---
ARGVUS 作为 Arch Linux 软件包通过官方 ARGVUS 软件包仓库分发。

## 安装前

在安装 ARGVUS 之前，请启用官方 Arch Linux `multilib` 仓库。编辑 `/etc/pacman.conf`，并取消注释 `[multilib]` 部分及其 `Include` 行：

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
然后同步软件包数据库并执行完整的系统升级：

```sh
sudo pacman -Syu
```
快速安装程序会配置 ARGVUS 仓库，但不会为你启用 `multilib`。

## 快速安装

在全新的 Arch Linux 安装上，你可以直接从 TTY 安装 ARGVUS，而无需先安装其他桌面环境。

运行：

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
安装程序：

1. 导入并在本地签署 ARGVUS 仓库签名密钥。
2. 配置 ARGVUS 软件包仓库。
3. 更新软件包数据库。
4. 安装 `argvus` 软件包及其依赖项。

安装完成后，注销并从显示管理器中选择 **ARGVUS**。

> **注意:** 快速安装程序需要 `curl`、`sudo`，以及标准的 Arch Linux 软件包管理工具。

## 仅配置仓库

若要仅配置 ARGVUS 软件包仓库而不安装桌面环境，请运行：

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
之后手动安装 ARGVUS：

```sh
sudo pacman -Syu argvus
```
## 手动安装

如果您更喜欢逐步配置存储库，请使用以下命令。

### 1. 下载存储库签名密钥

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. 导入签名密钥

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. 本地签署密钥

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. 配置 ARGVUS 仓库

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
将仓库配置添加到 `/etc/pacman.conf`：

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
仓库配置启用了 ARGVUS 仓库，包括 `argvus`、`argvus-extras` 和 `SigLevel = Required TrustedOnly`。Pacman 仅在包的签名有效且被本地密钥环信任时才接受它们。

### 5. 安装 ARGVUS

```sh
sudo pacman -Syu argvus
```
安装完成后，注销并从你的显示管理器中选择 **ARGVUS**。

## 更新

将 ARGVUS 与其他 Arch Linux 一起更新：

```sh
sudo pacman -Syu
```
仅更新 ARGVUS 软件包：

```sh
sudo pacman -S argvus
```

> **推荐：** Arch Linux 通常建议使用 `sudo pacman -Syu` 进行完整系统升级，而不是更新单个软件包。

## 删除 ARGVUS

删除 ARGVUS，同时保留其他已安装软件包仍然需要的依赖项：

```sh
sudo pacman -R argvus
```
删除 ARGVUS 及不再被任何已安装软件包需要的依赖项：

```sh
sudo pacman -Rns argvus
```
### 删除 ARGVUS 仓库

如果你也想要删除软件包仓库，请删除其配置：

```sh
sudo rm /etc/pacman.d/argvus.conf
```
然后从 `/etc/pacman.conf` 中删除相应的行：

```text
Include = /etc/pacman.d/argvus.conf
```
你可以使用以下方式编辑文件：

```sh
sudo nano /etc/pacman.conf
```
### 移除签名密钥

要从 pacman 密钥环中移除 ARGVUS 仓库的签名密钥，首先列出密钥：

```sh
sudo pacman-key --list
```
然后使用其密钥 ID 删除 ARGVUS 密钥：

```sh
sudo pacman-key --delete KEY_ID
```
将 `KEY_ID` 替换为 ARGVUS 签名密钥 ID。

> **警告:** 仅在你同时删除 ARGVUS 仓库时才删除此密钥。不要删除其他仓库仍需要的密钥。

## 用户配置

软件包安装不会将配置写入 `$HOME`。ARGVUS 从 `/usr/share/argvus` 中读取打包的默认配置，并且无需将其复制到你的主目录即可工作。

运行时配置遵循以下一般优先顺序:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config` 是唯一写入 `~/.config/argvus/data/generated/` 的组件，并且该投影是活动层：托管或本地副本仅在相应的生成文件缺失时生效。

ARGVUS 托管的配置位于 `~/.config/argvus/` 下。对于支持本地用户配置的应用程序，仍然可以使用可选的 `$XDG_CONFIG_HOME/<app>` 覆盖。

`argvus --setup` 调度器是可选的，用于显式手动覆盖，而非恢复。如果 `~/.config/argvus` 被删除，`SUPER + SHIFT + R` 将通过正常的重新加载流程重新创建规范配置文件和生成的运行时状态。

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all` 是支持的 TUI 配置文件的显式允许列表；它不克隆 `/usr/share/argvus`，也不复制脚本、服务、资源、生成的文件或 `config.json`。运行 `argvus --setup --help` 可查看当前选项。复制后，这些文件将成为用户拥有的覆盖版本，软件包升级不会替换它们。

要替换现有的复制配置，请使用 `--force`。ARGVUS 在替换之前会创建带时间戳的备份：

```sh
argvus --setup --copy <app> --force
```
## Hyprland 用户覆盖

有关完整的 Hyprland 覆盖指南，包括完整本地配置与增量 ARGVUS 片段之间的区别，请参见 [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/)].

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### 虚拟机回退

一些虚拟机在使用 Hyprland、Kitty 或 Quickshell 时可能会遇到渲染问题。ARGVUS 使用 `vmwgfx` 图形驱动检测虚拟机，并在必要时启用软件渲染。

如果在你的虚拟机上检测无法正常工作，请在 `user.lua` 中手动启用回退：

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
在创建或修改 `user.lua` 后，请注销 Hyprland 然后重新登录，以便会话启动的应用程序可以应用该变量。

使用以下方法验证：

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
预期输出是：

```text
1
```

> **注意：** `LIBGL_ALWAYS_SOFTWARE=1` 强制 Mesa/OpenGL 应用程序使用软件渲染，通常通过 `llvmpipe` 实现。这可能会降低图形性能，应作为受影响虚拟机的备用方案使用。
