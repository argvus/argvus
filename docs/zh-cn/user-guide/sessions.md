---
title: 会话
description: 启动并管理 ARGVUS 图形会话。
---
`argvus-session` 拥有会话入口点和 `systemd --user` 生命周期。它通过 `argvus start` 启动 Hyprland，导入会话环境并管理 ARGVUS 目标。

- [图形会话](/docs/argvus-session/graphical-session/)
- [TTY 会话](/docs/argvus-session/tty-session/)
- [登录界面](/docs/components/argvus-greeter/)

有用的诊断信息：

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
请查看开发者 [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) 以了解服务所有权。
