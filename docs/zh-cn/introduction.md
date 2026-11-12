---
title: 介绍
description: ARGVUS 是什么、它的桌面如何工作以及它的组件如何配合。
---
ARGVUS 是一个用于 Arch Linux 的模块化 Hyprland 和 Wayland 桌面环境。它是一个集成的软件包集合，而不是单一的整体应用程序：会话、合成器、Shell 界面、设置应用程序、提供者和视觉资源通过共享状态和明确的运行时协议协同工作。

此页面是文档的地图。它在高层次上解释了整个桌面；链接的页面包含操作和实现细节。

## ARGVUS 的工作原理

正常会话遵循以下路径：

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session` 拥有会话生命周期和环境设置的管理权。它并不实现每一个桌面功能：每个组件保留其自身命令、配置和服务负载的管理权。

关于安装和启动，请参见 [Installation](/docs/getting-started/installation/) 和 [Sessions](/docs/user-guide/sessions/)。关于服务管理，请参见开发者 [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/)。

## 主要层级

### 入口点和基础设施

`argvus` 包提供了 `/usr/bin/argvus` 调度器和主包入口点。`argvus-session` 提供图形和 TTY 入口点、用户目标和 `argvus sessionctl`。共享的 TUI 和本地化功能来自 `argvus` 工作区（`argvus-tui` 和 `argvus-i18n` crate）以及 `argvus i18n` 命令。

### 合成器和桌面外壳

`argvus-hyprland` 提供 Hyprland 配置和键绑定。`argvus-waybar` 提供修补过的 Waybar 二进制文件，而 `argvus-taskbar` 拥有任务栏配置和动作。`argvus-control-panel` 提供 Quickshell 面板，`argvus widget-telemetry` 提供可选的系统小部件，这些小部件在 `argvus` 内部运行。

请参阅 [Desktop](/docs/user-guide/desktop/) 了解用法，以及 [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/) 了解实现所有权。

### 设置和应用程序

`argvus control-center` 是以键盘为主的设置应用程序，作为 `argvus` 的辅助工具提供。日历和可移动设备工具是 `argvus` 中的 GTK 4 辅助工具；系统监视器和终端是 `argvus` 的子命令；启动器是 `argvus-launcher`。所有这些都与 shell 和调度器集成。

请参阅 [Applications](/docs/user-guide/applications/) 和 [command reference](/docs/reference/command-line/)。

### 提供者和集成

显示、网络、蓝牙、电源、通知、防火墙和可移动存储由专注项目提供。控制中心和控制面板使用这些提供者；它们不会替代这些提供者。

登录、锁屏和启动视觉效果也是独立的：`argvus-greeter` 处理 greetd，`argvus-lock` 处理 Hyprlock，`argvus` 中提供的转接助手绘制登录旋转器，加载覆盖层处理主题过渡，`argvus-boot-splash` 处理启动画面。

请参阅 [Hardware](/docs/user-guide/hardware/)、[Privacy and security](/docs/user-guide/privacy-and-security/) 和开发者 [component reference](/docs/developer-guide/)。

## 共享外观状态

主题、字体、壁纸、字体和效果偏好作为逻辑状态存储在 `$XDG_CONFIG_HOME/argvus` 下（通常为 `~/.config/argvus`）。`argvus-config` 是唯一写入 `data/generated/` 的组件，它在其中为 Hyprland、GTK、Qt6ct、Waybar、Quickshell、Dunst、通知、锁屏、Yazi、Superfile 和终端投射消费者文件。Rofi 是例外：`argvus launcher` 将其配置和主题打包为普通文件，并且用户编辑的副本位于 `data/rofi/` 下。外观辅助程序提交状态更改，然后协调外部应用程序；它们从不写入生成的树。

生成的目录不是事实来源。请参阅 [Appearance](/docs/user-guide/appearance/) 和 [configuration and state](/docs/developer-guide/architecture/configuration-and-state/)。

视觉系统还控制布局。粘性模式紧凑且方形；间距模式使用较大的窗口间隙、圆角以及任务栏和外壳周围更宽的边距。任务栏默认为顶部边缘，但通过共享布局状态支持顶部/底部放置。效果控制合成器和桌面表面的透明度、模糊、阴影和动画。

请参阅[theme table](/docs/user-guide/appearance/)了解当前的主题系列和颜色，[effects](/docs/argvus-appearance/effects/)了解共享效果契约，以及[taskbar](/docs/argvus-taskbar/taskbar/)了解任务栏和窗口的放置。

## ARGVUS 是什么——以及不是

ARGVUS 是运行在 Arch Linux 之上的桌面环境层。Arch Linux 提供操作系统和软件包；Hyprland 提供 Wayland 合成；ARGVUS 协调图形会话、外壳表面、设置、外观、桌面服务和集成应用程序。

因此，它不仅仅是一个 Hyprland 配置或个人 dotfiles 的集合。ARGVUS 定义了一个受支持的会话入口点，管理其桌面服务的生命周期，提供通用的设置和外观模型，并连接专注的组件，如网络、显示器、电源、通知、账户和可移动存储。

ARGVUS 不需要替代每一个应用程序。它在最可靠的情况下集成选定的上游应用程序，同时保持整个桌面的行为和配置一致。

这个名字结合了两个概念。*ARGV*，来自 *argument vector*，是程序接收的参数列表，反映了通过命令和快捷键驱动桌面的键盘优先目标。*US*，来自 *User System*，反映了以用户为中心的桌面目标。有关完整说明和发音音频，请参见 [About ARGVUS](/about/).

有关实现边界和软件包所有权，请参阅开发者[架构概览](/docs/developer-guide/architecture/overview/)和[仓库角色](/docs/developer-guide/architecture/repository-roles/)。

## 按目标继续

- [阅读常见问题](/docs/user-guide/faq/)
- [安装ARGVUS](/docs/getting-started/installation/)
- [启动图形界面或TTY会话](/docs/user-guide/sessions/)
- [配置主题和壁纸](/docs/user-guide/appearance/)
- [使用任务栏和控制面板](/docs/user-guide/desktop/)
- [了解命令、路径和服务](/docs/reference/)
- [研究架构](/docs/developer-guide/architecture/overview/)
