---
title: 主题和壁纸故障排除
description: 诊断主题传播和生成的外观文件。
---
检查 $XDG_CONFIG_HOME/argvus/config.json 中的规范 `appearance` 部分，然后检查  `$XDG_CONFIG_HOME/argvus/config.json` `$XDG_CONFIG_HOME/argvus/data/generated` 下生成的文件。旧的点文件仅作为迁移输入。

在 data/generated/ 之下，`argvus-config` 是唯一的写入者。要恢复损坏的主题或壁纸，请从规范文档重新生成整个内容，并应用结果，而不是直接编辑生成的使用者：

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
 `data/generated/` `argvus theme-switch`（运行 `argvus config theme-apply`）和 `argvus config accent` 是协调器：它们负责验证、锁定、显示启动画面，通过 `argvus-config` 提交，然后调节无法读取 `config.json` 的外部消费者。它们从不写入 `data/generated/`，主题事务也不会通过单独的脚本写入 GTK、存储或 `mode.css` 文件。使用 `argvus config accent --apply` 仅重新运行重音调节。

官方主题更改会应用其保存的外观值并更新受影响的运行时使用者。应用自定义主题会重启任务栏、Hyprland 重载、壁纸和控制面板服务。切换 Sticky 或 Spaced 会重新应用该模式的间距、边框和任务栏边距，然后重启任务栏、遥测和 Hyprland 重载服务。Dunst 的通知颜色在“勿扰模式”激活时是例外：它们的重载会延迟到 DND 关闭后。如果另一个界面保持旧颜色，请检查主题构建是否处于激活状态并且是你选择的主题，然后再次运行重载操作:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
每个主题文件都来自构建：未安装的主题（在  `theme.toml` `/usr/share/argvus/themes/` 或 `~/.local/share/argvus/themes/` 下没有 theme.toml）或无法渲染的主题将被拒绝，并且保持以前的主题。`argvus theme check DIRECTORY` 显示每个渲染错误的模板和行。Waybar 工具栏仅在其 JSONC 中有 `reload_style_on_change` 时才会重新加载颜色。如果切换似乎卡住，请检查它是否仍在运行；它会在 `theme-switch.active` 中记录自己，以及它的锁：

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
标记中仍然存活的进程 ID 表示切换尚未完成。切换会最多等待 60 秒以获取主题锁和投影，重载命令则等待 5 秒；进程已退出的标记是过时的，下一次切换会忽略它。在报告问题时，请包含上述命令的输出。

如果只有预登录迎宾程序是白色的或使用了备用主题，请将私有源与其公共投影进行比较：

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
在不更改greetd配置的情况下修复投影：

```sh
sudo argvus greeter-setup --sync-themes
```
该文件由外观编排器发布以进行预认证，并且位于 `data/generated/` 之外，因此普通的 `argvus config apply` 不会刷新它。

如果迎宾程序在 Kitty 缓存中报告 `Invalid color name: 'null'`，请更新 `argvus terminal` 软件包，并通过再次打开迎宾程序重新生成缓存。缺失的规范重音符号是默认值，而不是 Kitty 颜色；当前的终端启动器在写入 active_tab_foreground 之前会拒绝 JSON `null` 和无效的遗留值。

如果在多次更改主题后默认终端停止打开，请检查派生的 Kitty 缓存，而不是手动编辑它：

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
上述命令读取规范的首选项。 `active_tab_foreground` `.active-theme` 仅是兼容性映射。缓存由 `argvus terminal` 重新生成，并与主题应用同步。缺失或过期的 `/usr/lib/argvus/loading-theme/splash` 表明软件包更新不完整；在评估实时会话结果之前，请重建并安装协调的软件包。

如果窗口边框保留之前的主题颜色，请勿手动编辑 Hyprland 的 Lua 文件。主题和强调色的更改由 `config.json` 投影，然后 `argvus sessionctl reload` 重新加载 Hyprland 并在会话生命周期中重新应用活动、非活动和组边框颜色：

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
边框*首选项*可通过控制中心和控制面板中的`argvus config borders`（或其`borders-switch.sh`代理）进行更改；重新加载会在会话生命周期中重新应用已提交的颜色，而不会重新推导这些颜色。

检查计划和有效的合成器值：

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
如果 `hyprctl` 无法连接当前的 Hyprland 套接字，边界投影将被报告为失败，并且依赖组件不会被视为成功重载。在重试之前，请检查用户会话日志。
