---
title: 运行时生命周期
description: 会话启动与用户服务所有权。
---
`argvus-session` 导入环境，通过 `argvus start` 启动 Hyprland，并在合成器准备好后启动 `argvus-session.target`。目标引入了 shell、通知、壁纸、空闲和剪贴板服务。`argvus sessionctl` 协调启动、停止、重载、重启、状态和日志。

当 Hyprland 退出时，会话会停止目标。各个项目仍需负责自己的二进制文件和配置。

## 启动

图形入口是 `/usr/share/wayland-sessions/argvus.desktop`，它调用 `argvus session`。会话导入 ARGVUS 和 XDG 配置环境，将其与 `systemd --user` 及 D-Bus 激活同步，并将合成器启动委托给 `argvus start`。

`argvus start` 验证 Hyprland 配置，当发现系统生成的文件或用户配置无效时，应用打包的回退，检测虚拟化兼容性，并启动 Hyprland。合成器的准备桥随后调用 `argvus sessionctl ready`，该程序使用当前的 Wayland 和 Hyprland 环境启动 ARGVUS 目标。

该目标在启动长期运行的组件之前使用一次性准备单元。准备阶段应用生成的配置、主题和强调状态、显示器状态、应用集成和会话环境。持久服务由 `systemd --user` 管理，而不是由临时后台进程管理。

`argvus start` 在启动 Hyprland 之前排队 `argvus-config.service`（`--no-block`）：投影不需要合成器，因此它会与合成器启动重叠。然后 `argvus sessionctl ready` 启动目标，目标发现该单元处于活动状态或等待其作业，因此在准备运行之前，规范配置仍然被投影。一旦目标启动，`sessionctl ready` 会等待最多 1.5 秒让壁纸（背景层表面）和任务栏被映射，然后才移除加载覆盖层，因为一个简单的服务在绘制之前就被认为已启动；其单元未运行的表面不会被等待，并且等待会被记录为 `session.desktop-surfaces`，同时记录每个表面所花费的时间为 `session.wallpaper-surface` 和 `session.taskbar-surface`。致命的投影或目标启动失败会被记录，并停止独立加载覆盖层，从而防止出现看似无尽的旋转器。可选 视觉消费者必须解决回退问题或报告降级失败，而无需替换会话目标。

### 登录成本

无配置更改的登录不会重复进行昂贵的操作：

- 投影只在外观或效果部分发生更改时，或当其实际输出（已实现的用户副本和 `data/state/effects/<theme>.conf`）缺失时，才运行主题切换和效果适配器。
- 准备阶段仅在 `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp` 中的时间戳不再匹配打包的主题文件和强调标记时才应用启动主题和其上的强调。时间戳会对文件内容进行指纹识别，因此重新安装相同文件不会使其失效。更改的主题输入或新的强调会使其失效一次。迁移较旧的基于时间戳的时间戳也会重新构建一次。当主题被重新应用时，准备阶段还会重新加载已在之前文件上启动的 Hyprland。
- GTK 设置 (`gsettings`) 仅在重新应用启动主题时写入。在登录之间，dconf 会保留它们，因此稳定登录时不会再次运行它们。  
- `argvus-dunst.service` 是 `Type=dbus`：一旦 Dunst 拥有 `org.freedesktop.Notifications`，它就算作已启动，因此会话目标不会等待暂停状态恢复。  
- 准备阶段并行运行 `spaces-switch.sh` 和 `borders-switch.sh` (`--apply-static`)：它们写入不相交的文件（面板的 JSONC 边距和有效几何，以及面板的 CSS 和 Rofi 主题）。  
- 投影仅在生成文件内容更改时重写文件。未更改的文件将保持不变，因此稳定登录时不会再次 fsync 它们。  
- 准备阶段的显示步骤 (`argvus config display session-prepare`) 仅应用 DPMS 和禁用的输出：它既不再次投影，也不重新加载合成器。
- `argvus-session-reload.service` 运行 `argvus sessionctl reload-desktop --login`，只有在投影清单中将 `hyprland` 列为受影响的消费者时才重新加载 Hyprland。桌面组件不会等待它。

会话指示器（Wayland 模式下的交接助手）从合成器交接映射，直到 `argvus sessionctl ready` 启动目标为止，因此单个指示器覆盖整个启动过程。它没有 `PartOf=`，因为在新登录时重新创建目标不能导致其关闭；`sessionctl` 会显式停止它，而 `RuntimeMaxSec=30` 会在没有停止信号时终止它。

每个准备、重新加载和投影适配器步骤都会将其持续时间作为 `<epoch-ms> timing <step> <N>ms` 附加到会话日志中；参见 [会话启动故障排除](/docs/user-guide/troubleshooting/session-startup/)]。

路径解析、会话准备和 Hyprland 重载计划由 Rust 负责。
`argvus config session-prepare` 和 `argvus config session-reload-config` 运行启动和重载步骤；旧的 `hypr-init.sh` 脚本已被移除。

## 服务和关机

目标一般拥有 `argvus-control-panel.service`、`argvus-taskbar.service`、`argvus-widget-telemetry.service`（启用时）、`argvus-wallpaper.service`、`argvus-dunst.service`、`argvus-hypridle.service`、剪贴板监视器、键盘布局通知和 PolicyKit 集成。会话加载覆盖由 `argvus start` 启动，而不是由目标启动。可选服务的失败不会替代整个会话。

需要 Wayland 客户端连接的服务，包括控制面板和空闲管理器，会在启动其客户端之前等待可用的 Hyprland 运行环境。这避免了在合成器切换期间对过时套接字启动 Quickshell 或 hypridle；如果 Hyprland 被替换，systemd 可以在新的运行环境可用后再次启动该组件。

主题切换将在主题锁下将其提交、构建、投影和消费者重新加载序列化为一个生命周期操作。它不会重新加载 `argvus-config.service`，也不会重启服务：它执行的投影仅写入文件，并在持有锁的情况下将主题交给切换，正在运行的消费者将在原地重新加载（参见 [外观与主题](/docs/developer-guide/subsystems/appearance-and-themes/)]）。

当 Hyprland 退出时，`argvus start` 会停止 `argvus-session.target` 及与之相关的服务。这可以防止任务栏、外壳、通知、空闲和剪贴板进程在注销后继续运行。

## 重新加载

```sh
argvus sessionctl reload
```

Reload 会重新导入图形环境，然后在前一个域命令（如 `apply-theme`、`accent`、`set`/`patch`）或小部件遥测块更改已经投影了更改的情况下，**应用**已提交的生成。在这种情况下，它会恢复清单计划中列出的消费者，并且不会再次调用 `argvus config project`，因为重新投影会重建一个相同的树并丢弃计划。投影、迁移和 `ensure` 仅在不存在有效的待处理计划时执行——在会话启动时，或在 `argvus config recover` 需要恢复先前文档之后。当投影和托管运行时都是最新的情况下，允许进行干净的无操作。如果标准状态没有改变，但组合器套接字、服务或消费者过时，reload 会进行调和，而不是提前返回。正常的分发包括 Hyprland、任务栏、小部件遥测、控制面板、通知、壁纸和空闲管理。 根据投影清单，保持不变的可选消费者将保持不变。此时，`data/generated/` 下的每个文件都是由 `argvus-config` 写入的，因此会话从不自行重新渲染主题。然后重新加载会运行 `theme-adapters.sh`，它协调那些无法自行读取规范文档或已发布投影的消费者。这些适配器只读取已提交的版本：它们写入自己的消费者文件和缓存，但从不写入规范文档或 `data/generated/`，因此单个用户操作仍然消耗一次变更、一次投影、一次发布和一次重新加载。仅当单个拥有的服务需要明确隔离的重启时使用 `argvus sessionctl restart <component>`。查看 [services reference](/docs/reference/services/) 获取确切的单元名称。

投影和重载操作通过用户运行时锁进行序列化。对于临时时间跟踪，在调用重载之前设置 `ARGVUS_SESSION_PERF=1` 并检查 `~/.local/state/argvus/session.log`；该跟踪记录了阶段边界，但不会改变正常的重载行为。
