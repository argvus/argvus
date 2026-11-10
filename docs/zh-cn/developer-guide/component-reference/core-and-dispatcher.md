---
title: 核心与调度器
description: argvus 工作区，其调度器以及共享的 Rust crate。
---
`argvus` 是 `de/argvus` 的 Cargo 工作区，以及提供 `/usr/bin/argvus` 的包。`argvus-cli` 这个 crate 是调度器：它读取第一个参数，然后要么在进程中运行库命令，要么运行 GTK 4 辅助程序或私有可执行文件，或者运行打包的脚本。完整的子命令列表在 [命令行参考](/docs/reference/command-line/)] 中。

该工作区还包含命令共享的 crates：

- `argvus-config`：规范配置存储，也是生成树 (`data/generated/`) 的唯一写入者。它还是一个库，被 `argvus config`、`theme-switch` 和 `effects-toggle` 在进程中使用。
- `argvus-i18n` 和 `argvus-i18n-core`：`resources/locales/` 中的目录，它们的验证，以及 `org.argvus.i18n` QML 模块。
- `argvus-tui` 和 `argvus-theme`：共享的终端 UI 和主题加载。
- `argvus-runtime`：翻译的消息和命令共享的进程辅助程序。

`argvus-session` 保持为一个独立的存储库。它提供会话入口点（`session`、`start`、`tty`）和生命周期单元，而 `argvus sessionctl` 达到其控制器。
