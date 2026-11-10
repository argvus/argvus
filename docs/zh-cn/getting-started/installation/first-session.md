---
title: 第一次会议
description: 安装后启动 ARGVUS。
---
对于图形登录，请使用 `argvus greeter-setup --help` 配置打包的 greetd 集成，并使用 [Sessions](/docs/user-guide/sessions/) 中描述的显示管理器流程。

对于 TTY 会话，请在虚拟终端上登录并运行：

```sh
argvus --start-desktop
```
登录后，使用 `argvus sessionctl status` 检查 ARGVUS 用户目标。如果合成器或外壳未启动，请按照 [session troubleshooting](/docs/user-guide/troubleshooting/session-startup/)] 操作。
