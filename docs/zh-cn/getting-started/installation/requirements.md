---
title: 要求
description: ARGVUS 的系统要求和先决条件。
---
ARGVUS 以 Arch Linux 为目标，并需要支持 Wayland 的图形堆栈、Hyprland 以及由软件包拉取的运行时依赖项。显示管理器是可选的：ARGVUS 支持通过 greetd 的图形登录流以及手动 TTY 流。

在安装之前，请确保：

- 系统已通过 `pacman` 完全更新；
- 已安装可正常工作的内核和图形驱动；
- 用户可以运行 Wayland 会话；
- 配置软件包仓库时网络可用。

虚拟机可能需要按照 [Virtual machines](/docs/user-guide/virtual-machines/) 中描述的兼容性设置。
