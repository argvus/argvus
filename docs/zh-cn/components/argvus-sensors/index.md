---
title: 温度传感器
description: 由 lm_sensors JSON 提供支持的实时温度概览。
---
`argvus sensors` 打开由任务栏 CPU 温度弹出窗口使用的正在进行的 Ratatui 面板。它运行 `sensors -j`，根据硬件报告的芯片名称和标签对读数进行分组，并每两秒刷新一次。GPU 监控由 `argvus system-monitor --gpu` 处理。

读数使用摄氏度。面板显示 `max`、`crit` 以及在 lm_sensors 提供时的警报状态。只有在有正值限制时才显示仪表。缺失的 `lm_sensors`、命令失败、无效 JSON 和空读数仍然可见，作为可恢复状态；下一轮轮询将重试。

按 `q` 或 `Esc` 关闭。任务栏通过 `argvus tui-terminal` 在 Kitty 中打开面板；其本地化底部标签标题显示关闭提示。此命令拥有 CPU 温度详细信息；GPU 温度图标打开仅 GPU 的 btop 监控器。任务栏软件包已经依赖 `lm_sensors`，因此不需要单独的软件包或依赖。
