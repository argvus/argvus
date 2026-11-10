---
title: 贝壳表面
description: 任务栏、控制面板和遥测的所有权。
---
`argvus-waybar` 提供了补丁后的 `/usr/bin/waybar`。`argvus-taskbar` 拥有任务栏操作和用户体验；它提供的任务栏 JSONC 和 CSS 是 `config.json` 的投影，而 `argvus-config` 在 `data/generated/waybar/` 下写入活动层，并在 `data/waybar/` 下写入一个镜像副本。`argvus-control-panel` 拥有 Quickshell 卡。`argvus-widget-telemetry` 拥有可选的遥测栏及其块语义；其 JSONC 和 CSS 会在外观更改时从打包的默认值写入 `data/waybar/`，之后 `argvus-config` 仅重写带分隔符的 `ARGVUS_TELEMETRY_*` 和字体块。`argvus-session` 拥有他们的用户服务。

在此运行时模型中，目前没有 `argvus-shell` 包或 `argvus-shell.service`。
