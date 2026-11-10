---
title: 架构概述
description: ARGVUS 如何形成一个集成的桌面环境。
---
ARGVUS 有五个运行时层：

1. `argvus` 中的入口点和分发；
2. `argvus-session` 中的会话生命周期；
3. `argvus-hyprland`、`argvus-waybar`、`argvus-taskbar` 和 `argvus-control-panel` 中的合成器和 Shell 窗口；
4. 用户应用程序，如控制中心、日历和系统监视器；
5. 显示、网络、电源、外观、账户和通知的提供者和资源。

共享状态连接这些层。`argvus-config` 保存规范文档，并且是 `data/generated/` 的唯一写入者，因此主题或效果的更改会在一个事务中重新生成被多个包使用的文件；会话服务提供生命周期所有权，而不会获取每个组件实现的所有权。

## 职责边界

| 层级 | 主要职责 | 示例 |
| --- | --- | --- |
| 入口点和分发 | 启动桌面应用程序并解析 ARGVUS 配置文件。 | `argvus`, `argvus-app-profiles` |
| 会话生命周期 | 准备环境，启动 Hyprland，同步用户服务环境并管理自身会话的关闭。 | `argvus-session`, `argvus sessionctl` |
| 合成器和桌面环境 | 定义窗口行为和可见桌面表面。 | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| 配置 | 管理 `config.json`、路径模型以及所有生成的消费者文件。 | `argvus-config` |
| 应用程序 | 提供设置和专注的桌面工作流。 | 控制中心、日历、启动器、系统监视器和可移动设备 |
| 提供者和资源 | 实现域操作和共享的视觉资源。 | 显示、网络、电源、通知、外观、字体和壁纸 |

`argvus` 包是安装目标和协调器，但它不是单片运行时。专注的包保留其二进制文件、配置和服务的所有权。会话管理器在图形会话的适当时机启动它们，而不会吸收它们的实现。

在调试时，这一区别很重要：会话故障首先属于 `argvus-session`；任务栏故障属于 `argvus-taskbar`/`argvus-waybar`；生成的主题或壁纸故障属于外观和会话配置路径。
