---
title: 仓库角色
description: 工作区整合后 ARGVUS 源代码库的职责。
---
这些仓库按职责而非导航页面进行分组：

- workspace: `argvus`（调度器、进程内命令、GTK 4 辅助工具、共享 crates、`crates/argvus-greeter` 中的欢迎界面以及命令的数据）；
- session 和 integration: `argvus-session`、`argvus-hyprland`、`argvus-portal`、`argvus-lock`、`argvus-boot-splash`，以及欢迎界面（`argvus-greeter`，由 `argvus/crates/argvus-greeter` 构建）；
- workspace 外的 UI: 控制面板、任务栏（`argvus-taskbar`）、启动器和 Waybar；
- providers: 显示、电源、防火墙、通知；
- 资源和打包: 外观、壁纸、字体、图标、配置文件。

## 整合到 workspace

这些仓库已合并到 `de/argvus`。它们的文档现在位于 workspace 的 `docs/` 下，其仓库仅保留用于历史记录：

| 原仓库 | 现在 | 文档 |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [命令行](/docs/reference/command-line/), [配置文件](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [国际化组件](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | 命令行 |
| `argvus-terminal` | `argvus terminal` | [终端组件](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [系统监控组件](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [网络组件](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | 命令行 |
| `argvus-control-center` | `argvus control-center` (终端应用) | [控制中心组件](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 助手) | [任务栏日历组件](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 助手) | [可移动设备组件](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` 助手) | 命令行 |
| `argvus-tui-terminal` | `argvus tui-terminal` (工作区中的 crate) | 命令行 |
| `argvus-tui`, `argvus-theme` | 共享 crates `crates/argvus-tui` 和 `crates/argvus-theme` | 开发者笔记 |
| `argvus-greeter` | 源码在 `crates/argvus-greeter`，打包在 `argvus` 包中 | [问候组件](/docs/components/argvus-greeter/) |
| `argvus-themes` | 已退役。每个官方主题都是其自己的 `argvus-theme-*` 包 | [安装主题](/docs/user-guide/appearance/) |

合并组件的开发者说明在 `docs/internal/<repository>/`。

语言不是所有权的界限：每个组件保持其运行时辅助工具在其包内。跨包的唯一硬性规则是：只有 `argvus-config` 可以写入 `data/generated/` 之下的内容。需要访问无法读取 `config.json` 的应用程序的组件，会在其自己的辅助工具中添加适配器，并在标准提交后让协调器调用它；它从不写入生成的文件。公共 CLI 属于 `/usr/bin`；内部辅助工具属于 `/usr/lib/argvus/<component>` 或在运行时需要时位于命令目录下；与架构无关的数据保留在 `/usr/share/argvus/<component>` 下。

请参阅 [package matrix](/docs/reference/package-matrix/) 了解已安装包的边界。
