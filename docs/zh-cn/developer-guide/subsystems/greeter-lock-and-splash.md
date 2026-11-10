---
title: 问候者、锁和飞溅
description: 将启动和身份验证的视觉效果分开。
---
`argvus-greeter` 提供 greetd 接口。会话加载旋转器是交接助手（`/usr/lib/argvus/greeter/handoff --wayland`）的 Wayland 模式，由 `argvus-session-loading.service` 启动；GTK 覆盖 `argvus loading-theme` 处理主题过渡和直接重新加载。`argvus-boot-splash` 提供单独的启动画面。`argvus-lock` 提供 Hyprlock 配置。

主题状态在迎宾界面和会话切换之间传递，但这些软件包具有不同的生命周期和已安装的负载。

欢迎程序在用户认证会话之前运行，并使用 greetd 的 IPC/PAM 边界。当新的 Wayland 套接字可用时，会加载会话启动覆盖层，并在 `argvus sessionctl ready` 启动桌面组件后结束，因此它覆盖整个会话启动过程。Plymouth 主题属于启动路径，而不是登录屏幕。锁屏组件在登录后使用，并不替代任何一个欢迎程序。

登录交接有其自身的 DRM 表面。一旦身份验证成功，迎宾程序在请求 greetd 启动会话之前，会启动 `/usr/lib/argvus/greeter/handoff`（`argvus-greeter-handoff`）。当迎宾合成器释放 DRM 主控时，助手会立即获取它，并在用户合成器接管之前保持屏幕上的一帧画面。迎宾程序退出后，它仍然继续运行。一旦 greetd 接受会话，迎宾程序就会结束其合成器：greetd 仅在迎宾会话结束后才启动用户会话，并且合成器不会因 greetd 的 SIGTERM 而退出，因此如果没有这一点，登录会等待 greetd 的 5 秒钟终止。一个活跃的 Hyprland 在退出时会关闭每个输出，因此显示器随后会丢失信号。所以在结束其合成器之前，迎宾程序会将座席切换到空的 VT 12（logind `Seat.SwitchTo`，polkit 允许一个活跃会话；VT 12 高于 autovt 范围，因此那里不会启动 getty）。它 处理来自 `XDG_SEAT` 的座位（未设置时为 `seat0`），而不是作为 `seat/self`：Kitty 将问候程序移动到自己的 systemd 范围内，位于 logind 会话之外，logind 无法解析调用者的座位。logind 停用问候会话并从其合成器中获取 DRM 主权，合成器继续运行并在屏幕上显示其最后一帧。助手获取主权，用一次页面翻转替换该帧并隐藏问候程序的光标。问候程序等待它（助手仅在绘制后才响应 `ping`），然后才结束合成器。它仅停止 `start-hyprland` 看门狗，即问候会话命令，通过 SIGKILL 停止（时间日志中的 `greeter.watchdog-stop-requested`）：greetd 随即立即启动用户会话，而已停止的看门狗无法重新启动 Hyprland。稍后要求已失效且没有 DRM 主权的合成器退出（`greeter.compositor-exit-deferred`）：其自身组的一个进程等待交接表面。 移除其插座，最多10秒，然后再3秒，并运行`argvus config hyprland exit`。合成器的拆卸将其设备交回给logind，而在NVIDIA驱动程序上，logind随后花大约半秒钟关闭DRM客户端，对其他没有回应，包括用户会话的`CreateSession`；无论合成器是退出还是被杀死，这种情况都会发生，因此退出被移出登录过程。然而在greetd下，合成器位于欢迎器终端的前台进程组中，greetd的会话工作程序控制该终端：当工作程序在看门狗之后退出时，终端挂起会立即结束合成器（标记中的`compositor-in-foreground=true`），其关闭仍然在登录过程中运行，并且延迟请求找不到合成器。将合成器保持在登录之后需要它在自己的会话中。看门狗的PID是实例命名的合成器的父进程 `hyprland.lock`，并且两者都被检查为问候用户的预期程序。该标记还记录了看门狗是否是控制进程以及合成器是否在终端的前台组中，因为退出的控制进程会挂起该组。没有看门狗时，问候程序会立即请求合成器退出（`greeter.compositor-exit-requested`）。当非活动的合成器退出时，它不会更改输出，并且当它关闭设备时内核移除的帧缓冲器不再显示在屏幕上。greetd 切换回用户会话的虚拟终端。当切换或等待失败时，问候程序会将 `greeter.handoff-takeover-failed error=<reason>` 写入时序日志（其自身日志位于运行目录中，logind 在注销时会移除），并无论如何请求合成器退出，同时输出短暂变黑，在助手进行模式设置时显示器重新同步。在使用 fbcon 的系统上 绑定到虚拟终端（VTs）后，控制台可以在VT切换时瞬间显示。在等待之前，辅助程序会找到输出并通过一个观察者描述符（`handoff.prepared`）绘制它的第一帧，因为两者都不需要DRM主控，所以帧会在主控可用后立即显示在屏幕上。它仅在主平面仍有帧时尝试页面翻转；如果平面为空，内核会拒绝翻转，辅助程序则改为使用相同模式设置CRTC。在等待DRM主控期间，辅助程序保持在每个显卡上的描述符打开，因此合成器退出时不会成为触发内核恢复fbdev模式的最后一次关闭，并且它将每次CRTC更改记录为时间日志中的`handoff.crtc`（`fb=0`或`mode_valid=false`行意味着输出在两个主控之间关闭）。它从缓存状态读取连接器，而不会再次探测，并在页面翻转被拒绝时记录`handoff.page-flip-failed errno=<n>` 它会回退到模式设置。`greeter-session` 禁用该合成器（`ulimit -c 0`）的核心转储，因此在其拆卸过程中崩溃不会在写入核心时保持显示关闭。问候界面合成器在没有 Xwayland 的情况下运行：Kitty 和交接表面是 Wayland 客户端。

欢迎界面闪屏、此表面和会话旋转器被视为单个闪屏。在Wayland模式下，欢迎界面闪屏和会话旋转器是相同的助手：一个普通的共享内存 layer-shell 客户端，在连接后几毫秒显示其第一帧（GTK 覆盖层大约需要 300 毫秒，而合成器已经显示了其背景），登录路径中没有 GTK。三者都绘制相同的旋转器，从 `argvus-loading-theme-core`：四个方块不断增长，移动到下一个角落后缩回，循环周期为 1.5 秒（CSS loader 关键帧，每个片段上带有 `cubic-bezier(0.3, 1, 0, 1)`），物理像素大小相同，使用用户强调色（欢迎界面将 `--accent` 传递给两个助手），帧取自 `CLOCK_MONOTONIC`，因此当一个进程接手另一个进程时动画不会重新开始。当欢迎界面合成器以相同模式保持输出开启时，该表面显示其第一帧时 翻页而不是模式设置，因此显示器不会重新同步。Hyprland 的第一次模式设置会扫描一个它尚未绘制的缓冲区（黑色），并且它的第一帧会在 150–210 毫秒后出现，一旦其事件循环运行。在 `release` 上，当它仍然持有 DRM 主控权限时，该表面还会在 CRTC 的一个空闲叠加平面上放置其帧的副本（`handoff.overlay plane=<id>`）。Hyprland 只驱动主平面和光标平面，因此叠加层会保持在它的第一帧之上，旋转动画仍在叠加缓冲区中运行（写入该缓冲区不需要 DRM 主控权限）。会话旋转动画在 `--retire-handoff` 上运行：一旦其第一帧在每个输出上显示，它会发送 `retire`，表面移除叠加帧缓冲区，这只会关闭该平面（`handoff.retired`）。如果在 Hyprland 替换主平面后的 3 秒内没有收到 `retire`，表面会自己移除叠加层（`handoff.retire-timeout`），并在 10 秒后 在没有替换的情况下几秒钟后它会放弃（`handoff.replacement-timeout`）。没有可用的 XRGB8888 覆盖平面（`handoff.overlay-unavailable`），该表面只保留其主缓冲区，同时加载动画器继续动画，直到 Hyprland 替换它，而 Hyprland 未绘制的第一个缓冲区会在那一刻显示。Hyprland 在没有动画的情况下映射 `argvus-loading-theme` 图层（`no_anim` 图层规则），因此加载器在一帧内出现并消失。在映射之前，Hyprland 会绘制活动主题的启动背景，该背景由 `argvus config project` 在生成的 hypr/state.lua 中写为 `background_color`。

  `hypr/state.lua` `argvus-start` 在启动 Hyprland 之前将 `release` 发送到它的套接字 (`/run/argvus-greeter/handoff.sock`)，助手仅在放弃 DRM 主控后才确认。没有迎宾程序的会话发送相同的命令，但找不到任何表面，因此什么也不会发生。启动表面失败会被记录，但不会阻止登录。在运行时，通过 `/usr/lib/argvus/greeter/handoff --send theme=<name>`、`--send ping`（在绘制后会得到响应）、`--send release` 或 `--send retire` 向其发送命令。套接字保持打开状态直到表面退出。套接字和迎宾程序的登录前 `session.log` 位于 `/run/argvus-greeter`，`systemd-tmpfiles` 创建它并由 `greeter` 用户拥有，权限为 0755：只有迎宾程序在此写入，而登录的会话仅读取日志并连接到套接字。

在诊断视觉问题时，这种区分很重要：缺少启动画面是 Plymouth/软件包问题，空白过渡是会话加载或合成器准备问题，而认证失败属于 greetd/PAM 问题，而不是桌面会话问题。

主题切换不会显示覆盖层：正在运行的桌面会就地更改。直接会话重载会将所选主题显式传递给`/usr/lib/argvus/loading-theme/splash`。直接重载（`argvus sessionctl reload`，`SUPER + Shift + R`）仅在重新创建表面时显示会话启动画面：例如重新启动Waybar状态栏或控制面板，或回退的`full`方案。投影运行时不会显示它，并且那些就地重载的更改（Hyprland、主题使用者、壁纸、Dunst、polkit、hypridle）从不显示它；请参见[argvus-config](/docs/reference/argvus-config/)中的重载目标。控制中心本身保持相同的启动画面（`argvus config transition-overlay hold`），用于**应用**操作，该操作会重启状态栏或面板，因此它触发的重载不会启动第二个启动画面。它使用仅显示加载旋转器模式，因此其第一个映射的帧是 непрозрач的，并且它是通过`--detached`启动的：启动画面比短暂存在的`argvus config session-splash start`寿命更长。 session-splash start 是启动它的辅助程序，因为父进程终止信号会在该辅助程序退出时立即停止它。重新加载会在运行时阶段结束后停止 splash，无论哪条退出路径。splash 调色板仅作为备用。在源代码更改在运行会话中不可见时，请验证已安装的二进制文件和软件包内容。

登录交接套接字使用内核对等凭据授权每个客户端。只有 root、辅助程序所有者以及通过 greetd 认证的账户可以发送控制命令。其他本地账户会被拒绝，即使套接字允许跨用户连接。已准备的 DRM 帧会保持绑定到其原始描述符。如果该非特权描述符无法获取主控权，辅助程序会在保持观察者开启的情况下重新打开相同的 GPU，然后在获取到的描述符上准备一个新的帧。它从不在描述符之间重用帧缓冲区。

## 交接失败与取消

欢迎程序在接管成功之前拥有 DRM 辅助程序。启动失败、会话 IPC 错误以及接管失败会停止并清理该子进程，并移除其套接字（`greeter.handoff-cancelled`）。只有确认的第一帧会将辅助程序从用户会话中分离。接管超时时间仍然为两秒；取消失败的辅助程序可防止在 `argvus-start` 中额外等待。

辅助程序在获取 DRM 主控权限时以及绘制之后，提供经过身份验证的命令。提前的 `release` 会取消获取，关闭已准备的缓冲区和观察者描述符，然后回复 `released`（`handoff.release-before-master`）。悬而未决的 `ping` 仅在第一帧之后得到确认；显示失败将关闭它且不成功。一次只保留一个悬而未决的 ping。在显示之前，主题更新会应用到准备好的背景中。获取期间也会应用对等凭证和受限命令读取。

获取错误被记录为 ` when the errno changes. ` when the errno changes. handoff.acquisition-failed，记录最终失败；正常错误退出会移除套接字。这些标记区分了等待 DRM 的助手与实际上已经呈现帧的助手。

此获取遵循 [Linux DRM 主权限规则](https://github.com/torvalds/linux/blob/v6.18/drivers/gpu/drm/drm_auth.c)：一个从未成为主控的非特权描述符无法通过 `SET_MASTER` 提升自己，而在没有当前主控的情况下打开主节点会隐式获取主控。重新打开可以避免向问候程序添加权限。
