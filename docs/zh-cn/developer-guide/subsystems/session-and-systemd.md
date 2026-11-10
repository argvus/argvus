---
title: 会话和 systemd
description: ARGVUS 会话目标及其用户服务。
---
单元定义由 `argvus-session` 打包在 `/usr/share/argvus/session/config/systemd/user/` 下。会话管理器负责生命周期和环境导入。组件仓库负责其单元执行的命令。

在诊断集成会话时，请使用 `argvus sessionctl`，而不是手动启动组件。
