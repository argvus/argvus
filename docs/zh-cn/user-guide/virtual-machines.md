---
title: 虚拟机
description: 使用虚拟图形硬件运行 ARGVUS。
---
`argvus start` 检测虚拟化并导出受影响会话的兼容性设置，包括在检测到的环境需要时的软件渲染和鼠标/修饰符回退。

如果虚拟机仍然无法渲染，请在添加覆盖之前检查会话环境和日志。请参见 [graphics troubleshooting](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/)。

当检测到虚拟化时，`argvus start` 会为会话导出兼容性变量，包括软件渲染、软件鼠标光标和修饰符回退。这使得正常登录路径成为首先尝试的步骤；不要仅因为桌面在虚拟机中运行就添加永久覆盖。

如果检测不足，请使用用户 Hyprland 覆盖作为诊断回退：

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
仅在虚拟机图形堆栈需要时添加此项：

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
在更改会话环境后，请注销并重新登录。使用 `echo $LIBGL_ALWAYS_SOFTWARE` 确认活动值；软件渲染可以使桌面可用，但可能会降低性能。
