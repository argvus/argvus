---
title: 迎宾员
description: 在 argvus 包中提供的登录问候程序 greetd 启动。
---
`argvus-greeter` 是 greetd 的登录问候界面。它是一个键盘优先的终端前端，可以发现已安装的 Wayland 会话，显示带头像的帐户，并为所选帐户应用活动的 ARGVUS 主题。

它是 `argvus` 软件包的一部分，这是桌面环境的单个软件包。其源代码位于 `crates/argvus-greeter` 的 `argvus` 仓库中，作为根 Cargo 工作区的一部分，并且与 `argvus` 的其余部分一起构建。

问候界面本身不进行用户认证。greetd 通过 PAM 处理认证。启动器使用 `argvus tui-terminal` 打开问候界面，因此问候界面不需要单独的终端程序。

问候界面配置文件安装在 `/etc/argvus` 下。使用 `argvus greeter-setup --help` 查看软件包提供的设置选项。
有关登录流程，请参阅 [Sessions](/docs/user-guide/sessions/)。

登录交接套接字使用内核对等凭据授权每个客户端。只有 root、辅助程序所有者以及通过 greetd 认证的账户可以发送控制命令。即使套接字允许跨用户连接，其他本地账户也会被拒绝。准备好的 DRM 帧会保持绑定到其原始描述符；辅助程序会在获取到该描述符后才接管显示屏。

当认证字段获得焦点时，可打印字符（包括 `j` 和 `k`）会输入到响应中。Tab 和 Shift-Tab 可切换焦点；Backspace 删除最后一个字符。这适用于秘密和非秘密提示。

账户列表会在你导航或调整终端大小时滚动，以保持所选账户可见。
