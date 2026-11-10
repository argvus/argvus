---
title: 外观与主题
description: 主题状态在 ARGVUS 中的传播。
---
`argvus-appearance` 拥有主题、配色、壁纸和字体*状态*。它使用来自 `argvus-wallpapers`、`argvus-fonts` 和 `argvus-icons` 的资源，但它不会生成主题输出：所有 ARGVUS 拥有的 Hyprland、GTK、Qt、Waybar、Quickshell、Dunst、Hyprlock、Yazi、Superfile、终端和本地应用的消费文件都是由 `argvus-config` 单独编写的投影，Rofi 除外：`argvus-launcher` 拥有其文件，`argvus-appearance` 在 `data/rofi/` 下编辑用户副本。

控制中心和控制面板是用户界面；两者都不是主题状态的唯一来源。

`argvus theme-switch`（也包括 `argvus theme apply`）呈现所选主题，应用其导出的外观配置文件（或打包的默认设置），写入主题构建和投影文件，并通过会话控制器应用结果。官方主题会用其配置文件或默认设置替换独立的视觉覆盖。应用自定义用户主题还会重新启动 `argvus-taskbar.service`、`argvus-hyprland.service`、`argvus-wallpaper.service` 和 `argvus-control-panel.service`。粘性/间隔模式更改总是会重新确定该模式的间距、边框和任务栏几何，然后重新启动 `argvus-taskbar.service`、`argvus-widget-telemetry.service` 和 `argvus-hyprland.service`，包括在模式未更改时。模式几何优先于配置文件几何；用户可以通过将其视觉编辑导出为自定义主题来保留这些编辑。会话目标本身不会重新启动。

`argvus-config`（`argvus theme-switch`）的主题事务拥有每个无法自行读取`config.json`或其发布投影的消费者——本地Qt6ct文件、GTK `settings.ini`和`gsettings`提示、Foot和Kitty、问候主题状态、任务栏日历缓存、系统监视器、壁纸运行时和Hyprtoolkit。它从`argvus sessionctl reload`运行，因此主题切换、快捷键重载和会话启动都达到相同的运行时状态。适配器只读取已提交的生成：它们写入自己的消费者文件和缓存，而从不写入规范文档或`data/generated/`。锁屏模糊壁纸只能被作废，从不重写，因为它已经是已提交投影的一部分。`argvus config accent`记录规范的强调色，重新渲染构建并原地重新加载每个已注册的消费者，因此强调色更改和主题更改不会产生不一致情况。 变体。传统任务栏实用程序组标记在事务中被导入，`argvus config taskbar status` 报告有效值而不写入生成。

Hyprland 的 `SUPER + Shift + T` 绑定会打开 `theme-menu.sh`，它会在调用该脚本之前选择一个主题。在切换运行时，它会将 `theme-switch.active` 保留在缓存目录中，并记录其 PID 和进程启动时间。切换启动的投影会读取该标记并将主题保留给切换，因此锁不会被重复获取。等待锁和投影的时间各自限制为 60 秒，而消费者的重新加载命令总共限制为 5 秒。直接 `argvus sessionctl reload`（包括 `SUPER + Shift + R`）应用已经提交的生成而无需再次投影，并且在规范值未改变时还会修复过时的运行时消费者。`argvus terminal --apply` 在用户的缓存下生成派生的 Kitty 配置文件，使用共享锁和原子替换。

## 主题构建 (`argvus-theme-core`)

`argvus-theme-core` 从数据中呈现主题，上述交易通过活动主题的构建来应用它们：

- **活动主题的构建。** `argvus theme-switch` 渲染活动主题（包括布局和高亮颜色）并将 `generated/theme/current` 指向该构建。`argvus config accent` 使用新颜色再次渲染它，并在会话开始时重写从中复制的文件。此构建是必需的：每个主题文件都来自它。用户主题，无论是在控制中心导出还是导入，呈现方式相同。
- **消费者读取构建。** 每个消费者从 `current` 获取其文件：Waybar 任务栏和遥测 `@import` 的构建样式表；rofi 的 `theme.rasi` 导入构建调色板；qt6ct 的 `color_scheme_path` 指向它；Hyprtoolkit、应用程序样式、Yazi、Superfile、日历缓存和可移动设备样式表从中复制；`hyprland.lua`、控制面板 `Theme.qml`、锁屏和 Dunst 读取它；kitty 包含它，`argvus system-monitor` 从中构建 btop 的主题；TUI 调色板将其用于用户自己的活动主题。没有其主题构建的读取器从清单中在内存中呈现：迎宾程序（仅限官方主题，因为它无法读取用户的主目录），`argvus terminal` 和 `argvus system-monitor` 用于另一个主题，以及第一次切换之前的会话。没有构建时，`hyprland.lua` 使用 ARGVUS Dark 颜色，控制面板使用其内置默认值。每主题 旧开关（`data/<app>/themes/`）的副本被移动一次到 `data/backups/theme-copies-<seconds>/`（`theme_migration`）。
- **重新加载。** 每个注册表项声明其运行的消费者如何重新加载（`rewrite`、`signal` 或 `command`；一个命令可以使用 `${ARGVUS_DATA_HOME}` 和 `${ARGVUS_CONFIG_HOME}`）以及何时（`reload_stage`）。这些栏会重写其样式表的第一行（`/* argvus theme build: <build> */`），其 `reload_style_on_change` 将其转化为样式重新加载而不重新创建栏；控制面板使用 `qs -c argvus-control-panel ipc call theme reload` 被调用；Hyprland 评估活动 `theme.lua` 的颜色（`argvus config theme-runtime hyprland`，用户的强调色在活动边框上）；Dunst 通过 `argvus notifications theme reload` 重新加载，当勿扰模式激活时会延迟重新加载，并在关闭 DND 时应用；btop 接收 SIGUSR2，kitty 接收 SIGUSR1。`argvus config theme-runtime consumers` 手动运行每次重新加载。控制中心在构建或活动时重新读取其调色板 主题更改。

- **主题清单。** 一个主题是 `themes/<id>/theme.toml`，具有 `schema = 2`：`id`、`name`、`category`（`dark` 或 `light`），可选的 `wallpaper` 和 `preview`（相对于主题目录或绝对路径），`[name_i18n]`、`[palette]`，可选的 `[ansi]`（`color0` 到 `color15`），可选的 `[gtk] scheme` 和可选的 `[overrides]`。官方主题存放在 `/usr/share/argvus/themes/` 下，用户主题存放在 `$XDG_DATA_HOME/argvus/themes/` 下；官方主题优先于具有相同 ID 的用户主题，并且目录名称必须等于 ID。`argvus_theme_core::package` 为控制中心编写并检查用户主题：`derive_manifest` 从活动主题制作导出主题的清单（新的 `id`、`name`、`[name_i18n]`，有效的强调色作为 `palette.accent`，壁纸放在旁边），`check_theme_dir` 加载它并在两种布局中渲染每个使用者，`install_user_theme` 通过临时方式安装检查过的目录 目录和重命名，保留被替换的主题作为备份，直到成功。控制中心拒绝具有官方主题 ID 的归档，并将早期版本的 `profile.tar.gz` 配置文件一次性转换为主题，然后将它们移动到 `data/backups/theme-profiles-<seconds>/`。
- **调色板。** `bg`、`fg` 和 `accent` 是必需的；可以添加任何其他小写名称。一个条目可以是颜色（`#RGB`、`#RRGGBB`、`#RRGGBBAA`）或基于其他条目的表达式，例如 `"@accent | alpha(0.13)"`。用户选择的高亮颜色会替换 `accent`，因此由其派生的条目会随之更新。循环和未知名称是错误。
- **模板。** 文本文件，其中 `{{ expression }}` 被替换，`\{{` 写入字面值 `{{`。一个表达式读取 `palette.<name>`（或 `@<name>`）、`ansi.colorN`、`layout.<name>` 和 `theme.id`、`theme.name`、`theme.category`、`theme.variant`、`theme.gtk_scheme`、`theme.wallpaper`，随后是过滤器：`hex`、`hexa`、`argb`（`#AARRGGBB`）、`bare`、`rgb`、`rgba`、`rgba_percent`、`rgb_hex`、`hypr`（`rgba(rrggbbaa)`）、`argb_int`（`0xAARRGGBB`）、`alpha(x)`、`mix(colour, x)`、`lighten(x)`、`darken(x)`、`contrast`、`upper` 和 `lower`。
- **消费者注册表。** 每个组件在 `/usr/share/argvus/theme/consumers.d/<id>.toml` 中注册其主题文件：`template`（相对于 `/usr/share/argvus`）、`output`（相对于构建）和 `[reload]`，使用 `kind` `none`、`rewrite`（`path`）、`signal`（`process`, `signal`）或 `command`（`argv`）。`reload_stage = "build"` 会在构建激活后立即重新加载消费者；默认值 `files` 会等到开关写入它读取的文件后再进行。主题可以通过 `[overrides]` 替换一个模板。`[palette]` 为模板读取的标记提供默认值，作为调色板表达式（`btop_cpu_mid = "@fg | mix(@bg, 0.3)"`）；定义标记的主题将优先，并且两个消费者只能将同一标记默认到相同的表达式。通过这些默认设置，主题只需要 `bg`、`fg` 和 `accent`。
- **布局。** `/usr/share/argvus/theme/layouts/{sticky,spaced}.toml` 保存几何值模板，读取为 `layout.<name>`，因此主题不再需要 `-spaced` 副本。
- **构建。** 构建在内存中呈现，通过临时目录和重命名写入 `~/.config/argvus/data/generated/theme/builds/<id>-<variant>-<hash>/`，并由其 `build.toml` 描述（主题、变体、类别、GTK 方案、有效强调色、背景颜色、壁纸及使用者）。相同内容的构建会被重用。活动构建是 `current` 符号链接在 `builds/` 旁的目标，用 `rename(2)` 之一替换。除了活动构建和最近的其他构建，旧构建会被移除。
- **官方数据。** 每个官方主题包，以及 ARGVUS Dark 和 Light 的 `argvus-appearance`，都仅随附 `/usr/share/argvus/themes/<id>/theme.toml`（在需要时带 `overrides/`）及其壁纸；旧格式的每应用程序副本已不再提供。每个组件在 `/usr/share/argvus/<component>/theme-template/` 下提供其模板，并在 `/usr/share/argvus/theme/consumers.d/` 中为每个文件提供一个注册表条目（19 个使用者：控制面板、任务栏及其菜单图标、遥测、TUI、日历、启动器、通知、Hyprtoolkit、应用程序样式、Hyprland、Qt6ct、锁屏、终端、系统监视器、Superfile、Yazi 风格和语法主题、可移动设备）。`argvus-appearance` 提供 Sticky 和 Spaced 布局。调色板和模板是从旧的每主题文件中提取并检查输出是否一致；清单只保留与注册表默认值不同的条目，而一个主题文件若有 它自身的结构将其保持为覆盖。`argvus-appearance/tools/test-theme-switch.py` 渲染每个官方主题，以及仅包含所需颜色的主题，两种布局均适用。
- **模板部分。** `{{#if expression}}…{{/if}}` 仅在值为非零数字、非空文本或颜色时保留其文本；日历和 TUI 样式表使用它来处理仅存在于 Spaced 中的规则。`contrast` 过滤器为颜色上的文本提供黑色或白色，`rgba_percent` 编写 `rgba(r, g, b, NN%)`，`rgb_hex` 编写 `rgb(rrggbb)`。
- **命令。** `argvus theme list`、`argvus theme check DIRECTORY`（在两种布局中渲染每个注册的使用者，并报告其模板及行的每个错误）以及 `argvus theme render ID`（编写构建，或者 `--output` 下的文件，而不激活它）。

## Wobble 集成决策

Wobble 属于 Hyprland → 动画 → 窗口，并共享动画草稿。  
`effects.wobble_*` 是权威；`hypr/state.lua` 和 `decoration.wobble` 是其投影和消费者。组合动画设置命令保存一个原子补丁并请求一次重新加载。Hyprland 0.57.0 是最低依赖（选项 A）；对于包支持的安装，无需运行时版本检测。Wobble 默认关闭，因为在普通硬件上的渲染成本尚未测量。ARGVUS 网格的默认值是 8。版本 0.57.0 的源码将强度限制为 2，因此方案、CLI、UI 和 Lua 都使用该限制。
