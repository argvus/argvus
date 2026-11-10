---
title: 贡献
description: ARGVUS 项目和文档的贡献边界。
---
在更改功能之前，先追踪该功能在所有使用仓库中的情况。保留共享状态契约、包所有权、服务所有权以及两个本地化目录。

`argvus-config` 是 data/generated/ 的唯一写入者。在 `data/generated/` 中添加或更改消费者文件，在 `crates/argvus-config/src/project.rs` 及其 `project/` 下的子模块（`project/hyprland_projection.rs` 中的 Hyprland 文件）中进行，而不是在 shell 辅助工具中进行。如果消费者应用无法读取 `config.json`，请在拥有包中添加适配器，并在规范提交后从编排器调用它。删除 `data/generated/`，运行 `argvus config apply` 并确认树已经从 `config.json` 完全逐字重建。分别验证源代码、包负载和实时会话行为；仅构建源代码并不能证明已安装的 Wayland 会话正在使用新文件。
