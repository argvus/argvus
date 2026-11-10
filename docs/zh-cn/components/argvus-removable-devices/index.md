---
title: 可移动设备
description: 从 ARGVUS 挂载和管理可移动存储。
---
`argvus-removable-devices` 监控 UDisks2 并提供 ARGVUS 任务栏存储模块及其设备菜单。该菜单也可作为独立的 Rofi 风格界面使用，因此可以从任务栏或启动器工作流中执行相同的设备操作。

可用的操作有 `open`、`mount`、`unmount`、`eject`、`poweroff`、`lock`、`unlock` 和 `copy`。`list`、`watch` 和 `menu` 命令用于暴露相应的状态、任务栏流和菜单接口以便集成。

## 任务栏菜单

使用 `"mode": "gui"`（系统默认），点击任务栏存储图标会打开一个附加到 ARGVUS 任务栏的 GTK 菜单，就像日历弹出窗口一样：当任务栏在屏幕顶部时，菜单显示在条的下方；当任务栏在屏幕底部时，菜单显示在条的上方。每个设备行都有一个操作子菜单，仅显示该设备状态允许的操作。

- **指针操作:** 将指针悬停或点击设备行会打开其子菜单。当指针穿过菜单边缘或离开菜单时，子菜单仍会保持打开状态，且只有当指针在另一行短暂停留后才会跟随，所以对角线移动到子菜单不会关闭它。点击两个菜单之外的区域会关闭弹出窗口。
- **键盘操作:** `Up`/`Down` 选择一行，`Right`、`Enter` 或 `Space` 打开子菜单（或运行 **刷新**/**退出**），`Enter` 或 `Space` 执行选定操作，`Left` 或 `Escape` 返回设备列表，`Escape` 则关闭弹出窗口。

**复制路径** 通过 `copy_command`（默认 `wl-copy`，来自 `wl-clipboard`）将挂载点复制到剪贴板，因此菜单关闭后路径仍然保留；开启 `show_notification` 时会有通知确认。**解锁** 提供给每个未锁定的加密卷，必要时会先卸载它。

默认的文件管理器操作遵循在 ARGVUS 控制中心中选择的文件管理器。这使可移动设备的操作与用户的应用程序默认设置保持一致。

系统配置为 `/etc/argvus/removable-devices/config.json`。ARGBUS 管理的每用户投影为 `~/.config/argvus/data/removable-devices/config.json`，其旁边有一个匹配的 `theme.css`。该样式表也从标准主题、强调色和模式投影到 `~/.config/argvus/data/generated/removable-devices/theme.css`，对于未提供专用可移动设备资源的主题，则使用家庭/变体回退，因此主题更改可以在不进行任何手动操作的情况下保持模块一致。旧组件路径是迁移输入；后续的用户配置会覆盖系统默认设置。

有用的设置包括模块在没有设备时是否隐藏（`hide_when_empty`）、是否显示名称和容量、设备排序顺序、通知行为、菜单后端和状态图标。其样式表由活动的 ARGVUS 主题渲染（主题构建的 `removable-devices/theme.css`）并在每次主题或强调色更改时复制到面板的 `theme.css`。

界面使用 $XDG_CONFIG_HOME/locale.conf 中 `LANG` 的用户语言，对于未配置用户默认使用英语。继承的区域变量不会选择界面语言。

## 共享语言目录

界面使用适用于所有十种语言的共享  `$XDG_CONFIG_HOME/locale.conf` `argvus-i18n` 目录。文本存储在主工作区的 `resources/locales/<locale>/removable-devices.json` 中，并由 `argvus` 安装。英语是回退语言；新语言不再需要本地双语词典。
