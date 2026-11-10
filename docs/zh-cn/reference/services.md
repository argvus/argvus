---
title: 服务
description: ARGVUS systemd 和用户服务。
---
主要用户目标是 `argvus-session.target`。常见的用户服务包括 `argvus-taskbar.service`、`argvus-control-panel.service`、`argvus-widget-telemetry.service`、`argvus-hyprland.service`、`argvus-dunst.service`、`argvus-wallpaper.service`、`argvus-hypridle.service`、剪贴板服务和 `argvus-session-loading.service`。

防火墙是名为 `argvus-firewall.service` 的系统服务。日历软件包将 `argvus-taskbar-calendar.service` 安装为用户单元，并且 `argvus-clock.service`（默认禁用）在 **启动时随会话** 打开时会在登录后隐藏启动 ARGVUS 时钟。

检查活动单元：

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## 会话拥有的单元

会话包将这些用户单元安装在 `/usr/lib/systemd/user` 下（源负载位于会话包的共享配置树下）：

| 单元 | 角色 |
| --- | --- |
| `argvus-session.target` | ARGVUS 图形会话的生命周期边界。 |
| `argvus-session-prepare.service` | 一次性启动准备和生成的配置。 |
| `argvus-control-panel.service` | 快速壳控制面板。 |
| `argvus-taskbar.service` | 主要 ARGVUS 任务栏。 |
| `argvus-widget-telemetry.service` | 可选的遥测/系统信息界面。 |
| `argvus-hyprland.service` | 按需的一次性单元，运行 `hyprctl reload`。 |
| `argvus-config.service` | 投影规范配置（`argvus config project`）；重新加载时运行 `argvus sessionctl reload-config-transaction`。 |
| `argvus-session-reload.service` | 登录重载 (`argvus sessionctl reload-desktop --login`)：仅在投影影响时重载 Hyprland。 |
| `argvus-ssh-agent.service` | 会话的 SSH 代理。 |
| `argvus-secret-service.service` | 通过 `argvus sessionctl start-secret-service` 启动秘密服务。 |
| `argvus-wallpaper.service` | 活动壁纸后端。 |
| `argvus-dunst.service` | Dunst 通知守护进程。 |
| `argvus-hypridle.service` | 空闲和锁定策略。 |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | 剪贴板历史监视器。 |
| `argvus-keyboard-layout.service` | 键盘布局通知。 |
| `argvus-polkit.service` | PolicyKit 代理集成。 |
| `argvus-session-loading.service` | 从合成器交接到桌面组件启动期间的会话加载覆盖层。 |

`argvus-blueman-applet.service` 也通过会话包发送。一些单元是可选的或有条件启动的；它们并非在每个会话中都激活。

要检查单元的有效定义，而不仅仅是其源代码检出：

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
系统级别的 `argvus-firewall.service` 属于 `argvus-firewall`。日历包单独将 `argvus-taskbar-calendar.service` 作为用户单元进行分发。`argvus-clock.service` 是 `argvus` 包的一个用户单元 `WantedBy=graphical-session.target`，可从 ARGVUS 时钟设置中启用。

## 会话管理命令

`argvus sessionctl` 命令管理会话生命周期，并且可以在不注销的情况下重启特定组件：

### 会话生命周期

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### 特定组件重启

在不影响整个会话的情况下重启单个服务：

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### 环境同步

将环境变量导入会话：

```sh
argvus sessionctl import-environment
```
这会在 shell 和 systemd 用户服务之间同步 Wayland、Hyprland、Qt、光标和 DBus 变量。

## 服务单元参考

可以直接使用 `systemctl --user` 管理其他用户单元：

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
初始会话引导的日志存储在:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
