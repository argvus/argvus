---
title: 终端
description: 打开 ARGVUS 终端和终端应用程序配置文件。
---
使用以下方式打开已配置的终端：

```sh
argvus --terminal
```
`argvus terminal`（`argvus` 的一个子命令）提供终端集成和 Kitty 配置。`argvus-app-profiles` 提供终端应用程序的配置文件，如 Superfile 和 Yazi；它不提供 `argvus` 调度器。

`argvus-config` 将主题投射到 `data/generated/terminal/`，而 `argvus --yazi` 和 `argvus --spf` 则从投射的 `data/generated/yazi/` 和 `data/generated/superfile/` 树中读取配置，而不是从已生成到你的主目录中的副本中读取。在应用程序自身的配置目录中进行完整的本地覆盖仍然有效，因此你可以用自己的配置替换投射的树。

主题更改会重新生成 `$XDG_CACHE_HOME/argvus/argvus-terminal` 下 Kitty 的派生文件。生成器会序列化更新并以原子方式替换每个文件。如果终端仍然无法打开，请检查活动主题并使用以下方式重新生成缓存：

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```
终端首先从 `$XDG_CONFIG_HOME/argvus/config.json` 读取托管的主题和配色。旧的 `.active-theme` 和 `.accent-color` 文件仍然作为较旧安装的备用选项。

## 透明度和模糊

在 **控制中心 → 外观 → 终端** 下，可以独立配置 **透明度** 和 **模糊**（从 `0%` 到 `100%`）。更改通过 **应用** 按钮提交，并在 `$XDG_CACHE_HOME/argvus/argvus-terminal` 下重新生成 Kitty 配置文件；现有的 Kitty 窗口通过重载接收更新的配置。

`0%` 透明度保持背景不透明，`100%` 使其完全透明。模糊使用 Kitty 支持的背景模糊半径，仅在窗口透明时才有可见效果。

## 长时间运行的命令通知

运行时间为 30 秒或更长的命令在完成时会发送通知，只要终端窗口未获得焦点。该通知使用 Kitty 的 `notify_on_cmd_finish`，并使用已经启用的 shell 集成，因此 `~/.zshrc` 和 `~/.bashrc` 不需要更改。30 秒的阈值在 Kitty 配置中是固定的。

## TUI 应用模式

ARGVUS 控制中心和任务栏 TUI 弹出窗口使用 `argvus tui-terminal`，并配有专用的 Kitty 应用配置文件。它继承活动 ARGVUS 主题和字体，分配一个稳定的窗口类，并随其子进程一起退出，而不是打开 shell。控制中心隐藏标签页，并具有独立的透明度和模糊值；任务栏监控和温度弹出窗口使用底部标签栏显示其本地化的 `Press q to close` 标题。启动器接受 `--title TITLE` 来设置固定的 Kitty 窗口和标签标题。普通终端保持其标签和设置。

当启动器环境中缺少 LANG 时，`argvus terminal` 会在 Kitty 初始化键盘组合之前，从 `$XDG_CONFIG_HOME/locale.conf` 提供个人区域设置（默认英文）。现有的区域设置环境覆盖将被保留。更新后请重启 Kitty：在现有 shell 中导出 LANG 不会重新初始化 Kitty 的组合表。

会话初始化继承的空 LC_ALL 和 LC_CTYPE 会从 Kitty 子环境中移除。否则，即使有有效的 LANG，Kitty 也会将空的区域设置视为明确指定，从而无法加载其组合表。非空的覆盖会被保留。这同样适用于新配置文件，并需要在更新启动器后重新打开 Kitty。

专用的控制中心窗口（`argvus-control-center`）接收每次启动的 Kitty 覆盖 `confirm_os_window_close=0`，因此合成器关闭操作不会要求确认。该覆盖不会写入共享终端配置文件；其他 TUI 和常规终端保持其现有策略。
