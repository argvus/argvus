---
title: 软件包矩阵
description: ARGVUS 包的职责。
---

| 软件包 | 职责 |
| --- | --- |
| `argvus` | 主软件包：调度器、进程内命令（配置、翻译、密钥、项目、终端、系统监视器、网络、蓝牙、遥测、TUI 终端、设置）、共享数据和国际化(i18n)目录。每个组件软件包都依赖它。 |
| `argvus` (helpers) | 相同软件包中的辅助工具：控制中心和磁盘使用分析器（终端应用程序），以及用于任务栏日历、可移动设备、加载覆盖、ARGVUS Focus 和 ARGVUS Clock 的 GTK 4 应用程序。 |
| `argvus` (greeter) | greetd 的登录迎宾程序，与同一软件包一起提供，并由 `crates/argvus-greeter` 构建。 |
| `argvus-desktop` | 桌面和日常应用程序配置文件，由 `argvus` 主软件包所需。它不依赖 `argvus`，因此依赖关系保持单向。 |
| `argvus-devtools`, `argvus-pentest` | 可选的独立开发工具和授权安全审计元包。主包不安装这些配置。 |
| `argvus-session` | 会话入口点和用户生命周期。 |
| `argvus-hyprland` | Hyprland 配置。 |
| `argvus-control-panel` | Quickshell 控制面板。 |
| `argvus-taskbar`, `argvus-waybar` | Waybar 配置和补丁二进制文件。 |
| `argvus-branding` | 标志和字标资源。 |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | 外观状态和视觉资源。 |
| `argvus-display`, `argvus-power`, `argvus-notifications` | 桌面提供程序。 |
| `argvus-lock`, `argvus-boot-splash` | 锁屏和启动画面。 |
| `argvus-firewall` | 防火墙，具有自身权限。 |
| `argvus-launcher` | 启动器配置和入口点。 |
| `argvus-app-profiles` | 捆绑终端应用程序的数据专用配置文件。 |
| `argvus-portal` | XDG 入口和 Wayland/DBus 默认设置。 |
| `argvus-theme-*` | 每个官方主题系列的一个可选软件包。 |
| `argvus-game-snake`, `argvus-games` | 可选的。Snake 是一个独立项目；`argvus-games` 将可选游戏组合在一起。 |

## 替换的软件包

`argvus` 声明 `replaces`、`conflicts` 和 `provides` 为它所吸收的软件包，因此升级会移除它们及其文件，不会遗留为孤立文件：

`argvus-config`、`argvus-i18n`（含 `argvus-language-en-us` 和 `argvus-language-pt-br`）、`argvus-keys`、`argvus-projects`、`argvus-terminal`、`argvus-system-monitor`、`argvus-network`、`argvus-widget-telemetry` 和 `argvus-du`。`argvus` 也取代了 `argvus-control-center`、`argvus-taskbar-calendar`、`argvus-loading-theme`、`argvus-removable-devices`，以及之前的拆分软件包 `argvus-gtk` 和 `argvus-greeter`，它仍然提供这些包，以便依赖它们的软件包能够继续解析。账户管理器（`argvus accounts`）由 `crates/argvus-accounts` 构建并安装为 `/usr/lib/argvus/commands/accounts`，并附带其 polkit 规则。磁盘使用分析器由 `crates/argvus-du` 构建并安装为 `/usr/lib/argvus/commands/argvus-du`。

`/usr/bin/argvus-terminal` 保持安装状态，用作 `argvus terminal` 的兼容性启动器。Hyprland 的默认终端、控制中心目录和已保存的默认应用程序仍然使用该命令名称。

已移除的包 `argvus-pwvucontrol` 和 `argvus-snappy-switcher` 不属于任何包提供。

ARGVUS 组件包之间不需要安装顺序：`argvus` 是它们的主包，每个组件都依赖它。安装 `argvus` 也会安装 `argvus-desktop` 配置文件；`argvus-devtools` 和 `argvus-pentest` 仍为自选。包在此表中的存在并不意味着它拥有整个功能：例如，任务栏结合了 `argvus-waybar`、`argvus-taskbar`、`argvus-session` 和域提供者。
