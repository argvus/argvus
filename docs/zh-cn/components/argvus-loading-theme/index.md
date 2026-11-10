---
title: 加载覆盖层
description: 会话重新加载时显示的覆盖层。
---
加载覆盖层是一个 GTK 4 层壳表面，在 ARGVUS 直接重新加载会话时显示 (`argvus sessionctl reload`)。主题切换不会使用它：桌面会原地更改。它是随 `argvus` 包一起提供的一个辅助工具，位于 `/usr/lib/argvus/loading-theme/splash`，并且不会放置在 `/usr/bin`。

登录不使用它：问候界面启动画面和会话启动旋转器是交接助手的Wayland模式，它在没有GTK的情况下绘制相同的旋转器，并在毫秒级内显示其第一帧（参见[Greeter, lock and splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/))）。
使用`--spinner-only`时，覆盖层仅显示该旋转器，并在`SIGTERM`时退出。使用`--detached`时，当启动它的进程退出时，它不会退出；会话重新加载使用两个标志，并通过其pid文件停止它。
它从ARGVUS数据目录读取所选主题，并回退到打包的ARGVUS暗色调色板。它仅在映射的每个表面的第一帧之后报告就绪，而不是在进程启动时报告。
