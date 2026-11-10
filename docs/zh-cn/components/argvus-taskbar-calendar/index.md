---
title: 日历
description: 使用 ARGVUS 任务栏日历。
---
`argvus-taskbar-calendar` 是从 Waybar 日期模块或通过以下方式打开的 GTK4 日历弹出窗口：

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
它支持本地事件、提醒以及 ICS 的导入/导出。其用户服务是 `argvus-taskbar-calendar.service`。已安装版本支持的选项请使用 `argvus taskbar-calendar --help`。

日历遵循当前的 ARGVUS 主题，包括 Sticky 和 Spaced 变体。主题更改通过生成的用户缓存应用；如果正在运行的弹出窗口需要显式刷新，请使用 `argvus taskbar-calendar reload`。

从任务栏日期模块打开时，弹出窗口锚定在任务栏边缘：在顶部任务栏下方打开，在底部任务栏上方打开，并与日期模块的右边缘对齐。未使用任务栏几何信息的直接启动将使用指针或显示器备选位置。

## 命令

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## 事件和提醒

事件默认是关闭的。可以在弹出窗口的 **设置 > 事件** 中打开；选择会保存在状态目录中，同时也会启用并启动 `argvus-taskbar-calendar.service`，该服务用于显示提醒。关闭它们会停止该服务。

- **+ 事件**（或 `n`）会打开所选日期的编辑器。事件可以从今天开始创建和更改；过去日期的事件仍然可以打开和删除。
- 新事件默认从 09:00 开始，持续时间为设置中设定的默认长度。移动开始时间不会改变事件长度；结束时间早于开始时间表示事件将在次日结束。
- 每个事件在开始时都会显示带声音的通知，不论 **提醒** 和 **重复** 选项如何。事件在开始后保存（例如在当天创建的全天事件）会在 30 秒内提醒。
- **提醒** 会在事件开始前的选定时间添加提前通知（如果该时间已过，则在开始时通知）。
- 对于全天事件，时间字段变为 **在以下时间通知**：事件覆盖整天，并在该时间通知（00:00 在当天开始时通知）。重新打开事件时会保留时间。
- 对于全天事件，**重复** 会在该时间之后按选定间隔再次通知。
- 编辑事件而不移动其开始时间，不会重复已显示的提醒。
- 通知遵循日历语言设置（设置 > 语言）。
- 当某天有事件发生时，该天会显示一个点 (`•`)，包括过夜或多日事件的每一天。日期遵循本地时区。
- 本地事件在结束后 10 分钟会自动删除。如果事件在保存时已结束，则至少在保存后可用 10 分钟。

导入的 `.ics` 文件会保留其日期、时区（`TZID`）和提醒（`VALARM`）。
导入的事件除了自身的提醒外，还会在开始时发送通知。循环事件（`RRULE`）
会被存储和导出，但只显示并提醒其第一次出现。
在软件包更新后，提醒服务会退出，systemd 会在 30 秒内启动新版。
`sync` 仅报告已配置的远程日历；尚未实现 CalDAV 同步。

## 路径

- 配置：`/etc/argvus/taskbar/calendar/config.toml`（系统范围；软件包自带默认配置）。
- 样式：`/etc/argvus/taskbar/calendar/style.css` 和 `theme.css`；活动 ARGVUS 主题的颜色来自通过以下缓存构建的主题。
- 数据：`$XDG_DATA_HOME/argvus-taskbar-calendar/`（或 `~/.local/share/argvus-taskbar-calendar/`）。
- 状态：`$XDG_STATE_HOME/argvus/taskbar/calendar/`（或 `~/.local/state/...`）；事件切换保存在这里。
- 缓存：`$XDG_CACHE_HOME/argvus-taskbar-calendar/`（或 `~/.cache/argvus-taskbar-calendar/`）。

高亮颜色从 `~/.config/argvus/data/.accent-color` 读取，并在运行时应用于主题颜色之上。

自动语言选择遵循用户的 `locale.conf`，默认为英语。明确的日历语言偏好仍然会覆盖自动选择。

任务栏日历、ARGVUS Focus、ARGVUS 时钟和控制面板是互斥的：打开一个会关闭其他的。 

日历读取失败会单独显示，而不是显示为空的议程。打开数据库、加载每日事件和加载每月指标会有不同的提示信息。再次选择某一天以重试。刷新失败时，仅保留之前加载的该天或该月的数据；其他查询的有效结果仍然可用。 

## 共享语言目录

界面对所有十种语言使用共享的 `argvus-i18n` 目录。文本存储在主工作区的 `resources/locales/<locale>/taskbar-calendar.json` 中，并由 `argvus` 安装。英语为备用语言；新语言不再需要本地双语字典。
