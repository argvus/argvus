---
title: 文件位置
description: 重要的已安装 ARGVUS 文件和目录。
---

- `/usr/bin/argvus*` — 公共命令。
- `/usr/share/argvus/hyprland/` — Hyprland 配置和辅助工具。
- `/usr/share/argvus/control-panel/` — 控制面板 QML 和脚本。
- `/usr/share/argvus/taskbar/` — 任务栏配置和脚本。
- `/usr/share/argvus/lock/` — Hyprlock 配置。
- `/usr/share/argvus/portal/` — 门户集成文件。
- `/usr/share/backgrounds/argvus/` — 壁纸资源。
- `/etc/argvus/` — 系统提供者配置。
- `/etc/argvus/greeter.toml` — 欢迎界面配置。
- `/etc/argvus/removable-devices/config.json` — 可移动存储的系统默认值。
- `$XDG_CONFIG_HOME/argvus/config.json` — 标准便携式偏好设置。
- `$XDG_CONFIG_HOME/argvus/data/` — 管理组件数据、备份、锁定和生成的输出。
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — 可选的完整原生 Waybar 任务栏覆盖。
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — 管理 ARGVUS 任务栏副本。
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — 生成 Waybar 配置文件，包括 `argvus-taskbar.css` 和 `argvus-widget-telemetry.{jsonc,css}`；这些优先于上述副本。
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — 也在 `data/waybar/` 下镜像，当外观发生变化时从打包的默认值替换，然后重写其 `ARGVUS_TELEMETRY_*` 和字体块。
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — 投影 Yazi 配置树，对于未提供 Yazi 版本的主题，提供支持变体的备用方案。
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — 投影 Superfile 主题集。
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — 投影终端配置文件。
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — 预期的 Qt6ct 调色板。
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — 预期的 GTK 主题文件。
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — 用户的 Rofi 配置、主题和模式副本。当文件首次被修改时，ARGVUS 创建每个副本；未修改的文件从 `/usr/share/argvus/launcher/config/` 读取。
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — 预期的可移动设备样式表。
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — ARGVUS Focus 设置、会话历史和当前会话检查点。
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — 临时的 ARGVUS Focus 状态，由任务栏使用；在 Focus 退出时移除。
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS 时钟设置和世界时钟、闹钟，以及计时器/秒表检查点。
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — ARGVUS Focus 和 ARGVUS 时钟的颜色（主题使用者 `time`）。
- `/usr/share/argvus/sounds/` — ARGVUS Focus 和 ARGVUS 时钟的声音（`argvus-chime.oga`，`argvus-beacon.oga`，`argvus-bell.oga`）。
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — 主题构建（`builds/<id>-<variant>-<hash>/`），由 `argvus theme render` 编写，并由每次主题切换和强调更改触发，以及指向活动主题构建的 `current` 符号链接。
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — 用户 Hyprland Lua 覆盖和原生 Hyprland 投射。
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — 由 `argvus start` 选择的可选完整 Hyprland 原生配置；它不是增量覆盖。
- `$XDG_STATE_HOME/argvus/config-projection.json` — 投影清单：生成、有效配置哈希、已更改的部分和待处理的运行时工作。由 `argvus sessionctl reload` 读取；从不作为真源。
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — 传统遥测块首选项。仅用于迁移输入；规范值保存在 `config.json` 中，这个文件不再写入。
- `$XDG_STATE_HOME/argvus/session.log` — 图形会话日志（通常是 `~/.local/state/argvus/session.log`）。
- `/usr/share/wayland-sessions/argvus.desktop` — 图形会话条目。
- `/usr/share/plymouth/themes/argvus/` — 打包的启动界面资源。

具体的有效负载为包特定；请参阅开发者组件参考和相关的软件包清单。
