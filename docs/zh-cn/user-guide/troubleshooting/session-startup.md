---
title: 会话启动故障排除
description: 诊断 greeter、Hyprland 和 systemd 用户会话故障。
---
跑

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
分别检查 greeter、`argvus start`、Hyprland 以及相关的用户单元。成功的包构建并不能证明已安装的 payload 或活动用户覆盖是最新的。

按以下顺序使用检查：

1. 确认 greeter 使用的是已安装的 ARGVUS greetd 设置。
2. 读取会话日志，并检查是否缺少或无效的 Hyprland Lua 配置。
3. 使用 `systemctl --user status argvus-session.target` 检查就绪状态和目标状态。
4. 检查失败的组件，而不是一次重启所有服务。

如果 `argvus-session.target` 在加载覆盖层激活时处于非活动状态，请在重启服务前检查投影错误：

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config` 是 `data/generated/` 之下的唯一编写者，因此 `project` 是在诊断时重现一代的权威方法。会话启动时运行它，因为尚未存在提交的计划；它写入 `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` 的计划就是后来 `argvus sessionctl reload` 应用的计划，而无需再次投影。

当严重准备或目标启动失败时，会话会停止加载覆盖层。视觉消费者失败应使用其文档中描述的回退方法，或明确报告；它不得被隐藏为成功会话。

## 登录缓慢

如果出现 `greeter.handoff-takeover-failed` 而没有 `handoff.master-acquired`，检查 `handoff.master-unavailable card=<name> errno=<n>` 是否有获取错误。当一个观察者在另一个合成器为主时打开，即使该合成器释放了主控，也可能返回 errno 13 (`EACCES`)。助手会重新打开相同的 GPU 来隐式获取主控；`handoff.master-reopened` 记录了此恢复，并且在该描述符上准备了新的帧缓冲区。迎宾程序取消了失败的助手 (`greeter.handoff-cancelled`)，所以在用户合成器启动之前，它无法添加另一个等待。会话在获取挂起时发送 `release` 会得到 `handoff.release-before-master`：助手会在确认之前关闭它的 DRM 描述符。Hyprland/Aquamarine 中的迎宾合成器崩溃是另一个发现；取消助手并不意味着驱动程序或合成器问题已经解决。

`greeter.vt-switch-returned active=<tty>` 记录 login 的 switch 调用返回时的活动 VT。绘图超时也会在其错误中记录 `active-vt=<tty>`。这些是快照，而不是 DRM 完成交接的证据。

为了进行前后比较，请保存工作并在同一台机器、主题和显示器设置下，每个版本使用五个登录。在每次随后的注销之前，将 `/run/argvus-greeter/timing.log`、会话日志和启动日志复制到 `/tmp/argvus/` 下的单独样本目录；下一次 greeter 运行会替换其定时日志。记录每个样本的启动 ID，并仅在该启动期间比较时间戳。测量从身份验证到第一帧交接、会话包装器进入、最终 Hyprland 启动及加载旋转器停止的时间。比较身份验证到旋转停止时间的中位数和最差值，并通过视觉检查黑屏、闪烁或旋转器卡住的情况。仅桌面表面持续时间并不等同于端到端登录时间。在一次性会话中重复失败和恢复测试，但不要终止包含未保存工作的会话。每个启动步骤都在会话日志中记录其持续时间。要查看最后一次登录的步骤，从最慢开始：

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
要计算从迎宾程序切换到桌面的时间，请将迎宾程序和交接标记与日志一起读取。它们都使用 `CLOCK_MONOTONIC`，因此可以对齐：

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` 仅保存最后一次登录：欢迎界面、DRM 交接表面（`handoff.master-acquired`、`handoff.output-found`、`handoff.first-frame`、`handoff.release-received`、`handoff.replaced-by-compositor`）和 `greeter.*` 标记。`handoff.crtc` 行记录每次 CRTC 更改，同时交接等待 DRM 主控；出现 `fb=0`、`mode_valid=false` 或 `handoff.output-found ... framebuffer=0` 的行，意味着欢迎界面合成器在交接获取前关闭了输出，因此显示器必须重新同步。`greeter.watchdog-stop-requested` 表示欢迎会话在未等待其合成器的情况下结束，而 `greeter.compositor-exit-deferred` 表示一旦交接表面消失，就会要求合成器退出；单独的 `greeter.compositor-exit-requested` 表示被要求立即退出。如果日志中 greetd 和 logind 行显示欢迎会话关闭与用户会话之间大约有半秒的间隔 在启动时，迎宾程序的合成器拆解在登录过程中运行。这是预期的，同时标记显示 `compositor-in-foreground=true`：迎宾程序终端的挂起在延迟退出之前结束了合成器。当迎宾程序在其合成器退出之前无法离开其 VT 时，就会发生这种情况：查找 `greeter.vt-switch-requested`，然后是 `greeter.handoff-drawing`；如果第二个缺失，`greeter.handoff-takeover-failed error=<reason>` 会说明原因。`handoff.page-flip` 表示交接替换了迎宾程序的最后一帧而没有进行模式设置；`handoff.page-flip-failed errno=<n>` 后跟 `handoff.modeset` 表示内核拒绝了页面翻转。`handoff.overlay plane=<id>` 然后是 `handoff.retired` 表示叠加层在会话加载动画显示之前隐藏了 Hyprland 的第一帧；`handoff.overlay-unavailable` 表示 CRTC 没有空闲的叠加平面，`handoff.retire-timeout` 表示会话加载动画从未发送。 `retire`。会话旋转器 (`argvus-greeter-handoff`) 将 `first layer-shell frame callback` 和 `stop requested` 写入日志。

在外观更改、更改主题文件、新的强调色或旧启动标记迁移后，预期会有一个 `project.theme-switch` 或非零的 `prepare.startup-theme` 条目。重新安装相同的文件会保留基于内容的启动缓存。如果每次登录时都出现该情况，表示已材料化的主题文件缺失；请运行 `argvus theme-switch <theme>` 一次。

会话日志保存在 `$XDG_STATE_HOME/argvus/session.log`（通常为 `~/.local/state/argvus/session.log`）下。在更改用户 Hyprland 覆盖后，注销并重新登录，以便会话入口点进行验证并再次选择它。请参阅 [图形会话](/docs/argvus-session/graphical-session/) 以了解正常启动流程。
