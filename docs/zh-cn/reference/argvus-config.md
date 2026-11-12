---
title: ARGVUS 配置
description: ARGVUS 使用的逻辑 JSON 配置合同。
---
ARGVUS 将托管的用户偏好保存在 `$XDG_CONFIG_HOME/argvus/config/`（通常为 `~/.config/argvus/config/`）下的模块化 JSON 文件中。它们构成了 ARGVUS 管理的偏好的标准逻辑模型，而 `argvus-config` 是唯一在 `data/generated/` 下写入文件的组件。本地文件和生成的文件保持兼容性投影或特定于应用程序的覆盖。

## 用户指南：快速开始

对于大多数用户，配置通过 ARGVUS 控制中心或此命令行工具完成：

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
对于管理员和高级配置，请继续阅读以下内容。

用户 root 仅包含规范文档和管理数据：

```text
~/.config/argvus/
├── config.json
└── data/
```
该文件以原子方式写入，对用户保持私密，并由 `.config.lock` 保护，位于可以在投影期间交换的 `data/` 树之外。之前的版本存储在 `data/backups/config.json.bak`。

## 命令行

`argvus config` 子命令由 `argvus` 软件包在进程中提供：

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure` 根据模式默认值生成新的 `config.json`，或在不替换显式值的情况下填充缺失的字段。在 JSON 格式错误时会失败。
`migrate` 导入现有的旧外观状态，但不会替换 `config.json` 中已存在的值。`argvus sessionctl` 在应用配置更改后的重新加载之前运行 `recover`、`ensure`、`migrate` 和 `project`：

```sh
argvus sessionctl apply-config
```
`apply` 和 `rebuild` 是用于从 `config.json` 重建每个投影的别名；`project` 是历史拼写，并接受 `--force`。`apply` 还将提交后运行时重新加载的任务委派给 `argvus sessionctl`。设置 `ARGVUS_NO_RUNTIME=1` 用于准备或孤立测试。生成发生在暂存环境中，且 `data/` 树只有在所有投影器成功后才会被替换。因此，删除 `data/generated/` 并运行 `argvus config apply` 是在不发布部分 Hyprland 或任务栏输出序列的情况下可重复的重建。该命令会将派生的部分哈希值和运行时决策记录在 `$XDG_STATE_HOME/argvus/config-projection.json`（通常为 `~/.local/state/argvus/config-projection.json`）中。清单文件和生成的文件都不是事实来源。`apply` 通过 `argvus sessionctl` 执行提交后的重新加载；`project` 保持仅投影功能以保证兼容性。

每一个持续的变更都使用相同的规则：如果规范文档发生了变化，CLI 会在提交后调用一个集中式的 `argvus sessionctl reload`。重载结果记录在 `data/internal/last-reload.json` 中。

因此，`argvus sessionctl reload` 是一个 *应用* 步骤，而不是投影步骤。当传入的清单报告有待处理的工作时，它会还原计划中描述的目标消费者，并且不会再次调用 `argvus config project`；重新投影会重建一个相同的树，并丢弃变更刚产生的计划。只有在没有已提交计划存在的情况下，例如在会话启动期间，它才会回退到迁移和投影。

### 重载目标

`argvus config session-plan` 对 reload 的回答包含了已更改部分的目标，而 `argvus sessionctl` 只运行这些目标。只有当目标重新创建一个表面时，splash 才会覆盖 reload：Waybar 栏或控制面板移回屏幕时会移动每个窗口。其他所有内容都在原地 reload。

| 变更 | 目标 | 启动画面 |
| --- | --- | --- |
| 主题、高亮颜色或 GTK 模式 | `hyprland`, `consumers`, `wallpaper`, `polkit` | 否 |
| 仅壁纸 | `wallpaper`（hyprpaper IPC；服务仅为 swaybg 重启） | 否 |
| `effects`（模糊、动画、透明度、阴影） | `hyprland`, `consumers` | 否 |
| 窗口的 `layout`（间隙、边框） | `hyprland` | 否 |
| `layout.taskbar`（位置、边距） | `hyprland`, `taskbar`, `widget_telemetry` | 是 |
| `fonts` | `consumers` | 否 |
| `control_panel`（切换、卡片） | `control_panel` | 是 |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | 否 |
| `power` | `hypridle` | 否 |

`consumers` 运行 `argvus config theme-runtime consumers`（包括栏样式表、通过 IPC 的控制面板、Hyprland 颜色、Dunst 的主题重载、btop 和 kitty）。当“请勿打扰”激活时，Dunst 会推迟其主题重载。`wallpaper` 运行 `argvus config theme-runtime wallpaper`，显示有效的 `/appearance/wallpaper`。`layout.taskbar` 哈希与投影清单中的窗口布局分开保存，因此间隙更改不会重启栏。备用 `full` 计划在不存在已提交计划时使用，会重新启动启动画面下的每个界面。

每个有效配置都有一个操作生成。它存储在 `data/internal/generation.json` 中，并且在投影清单下位于 `$XDG_STATE_HOME/argvus/config-projection.json`；它不是点文件配置的一部分。重新应用未更改的有效配置会保留该生成。`status` 报告清单哈希、生成元数据和所有生成文件是否一致；`apply` 修复过时或不完整的投影。

`explain /path` 直接读取解析器并打印有效值、来源、主题和变体。它不会解析生成的文件或创建第二个优先规则。

主题和效果投影在 `argvus-config` 本身实现。
主题事务（`argvus theme-switch`，运行 `theme-apply`）、`argvus config accent` 和 `argvus effects-toggle` 不进行投影：
它们通过此 CLI 验证、锁定、提交，然后
调节无法读取 `config.json` 的外部消费者。`argvus effects-toggle`
是一个薄代理，本身没有输出。外观辅助程序从未被投影器调用，并且它们绝不能在 `data/generated/` 以下写入。

### 投影表面

`project_effective` 在发布前在一个暂存树中呈现每个 ARGVUS 拥有的消费者：

| 表面 | 输出 |
| --- | --- |
| Hyprland | `generated/hypr/`，包括主题配置文件、`spaces-effective.conf`、`borders-effective.conf`、`hypridle.conf` 和 `monitors.lua` |
| 布局/效果 | `generated/effects/`、`generated/spaces.conf`、`generated/borders.conf` |
| Waybar | `generated/waybar/`，包括 `argvus-taskbar.css` 和 `argvus-widget-telemetry.{jsonc,css}` |
| Widget 遥测主题 | `generated/widget-telemetry/` 和 `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| 通知 | `generated/notifications/` 和 `dunstrc` |
| 锁屏 | `generated/hypr/hyprlock.conf` 以及 Hyprtoolkit 主题配置文件 |
| GTK | `generated/gtk/` |
|睚眦| `generated/yazi/` |
|超级文件 | `generated/superfile/` |
| 终端和 Qt 配置文件 | `generated/terminal/`, `generated/qt6ct/` |
| 字体 | `generated/fonts.conf` 以及在 Waybar 样式表中管理的字体块 |
| 可移动设备 | `generated/removable-devices/theme.css` |
| 壁纸 | `generated/hypr/hyprpaper.conf` 和 `data/hypr/hyprpaper.conf` |
| 控制面板和默认设置 | `data/control-panel/`, `data/control-center/defaults.json` |
| 日历、输入、键盘、电源 | 日历 TOML、输入片段、键盘状态、防休眠标记 |

主题层在生成的根之外也有镜像：
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` 和
`data/hypr/application-style.conf`。当外观部分发生变化时，这些副本会被打包的默认值替换，然后投影器只重写其分隔的管理块——Waybar 样式表中的字体块和遥测配置文件中的 `ARGVUS_TELEMETRY_*` 块——因此标记之外的文本不会在切换主题时保留。`fonts.targets` 和
`control_panel.widget_telemetry_blocks` 保持为规范：控制中心通过 `argvus-config` 保留它们，并且自己从不写入这些文件。

Widget 遥测块选择在 `/control_panel/widget_telemetry_blocks` 上是规范的，它是七个块名称的数组（`system`、`cpu_gpu`、`memory`、`storage`、`processes`、`network`、`keys`）。遗留的 `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` 文件仅作为迁移回退读取，从不被写入。` and ` and blocks all 会以单次原子写入方式修改规范数组，而 `apply-state` 假设其调用者已经持久化了选择。

Hyprland 间距和边框偏好遵循相同的边界。argvus config spaces 和 argvus config borders 命令（及其历史兼容包装器）会更新 /layout；Rust 生成 data/generated/spaces-effective.conf 和 data/generated/borders-effective.conf 并负责验证和实时应用到 hyprctl。 .spaces 和 .borders 仅作为迁移状态，从不作为真实数据来源。

Accent 和 GTK 模式状态使用相同的 Rust 边界。使用 ` or ` or argvus config accent --theme-default 来获得 accent 所有权，GTK 模式使用 `argvus config mode toggle|set dark|light`。历史的 accent 和模式脚本是兼容性包装；`.accent-color`、`.accent-custom` 和 `.gtk-mode` 仅为迁移/投影文件。

逻辑 ARGVUS 路径也由 `argvus config paths` 解析。传统的 `paths.sh` 文件仅暴露兼容性函数；本地覆盖、用户数据、生成数据和系统回退由 Rust 路径模型解析。

`audio` 在一步操作中更改默认输出接收器并将结果记录在 `/audio` 中：`argvus config audio volume <N|+N|-N> [--limit PERCENT]` 和 `argvus config audio mute <on|off|toggle>`。它在不重新加载配置的情况下写入存储，因为 PipeWire 已经具有该值。Waybar 音量模块、Hyprland 媒体按键和控制面板使用它，因此 `argvus config project`（将 `/audio` 投影到接收器上）从不恢复较旧的音量。超过 100% 的音量（Hyprland 按键允许最高 150%）会被存储为 100。

`patch` 应用一个 JSON 对象，其键是 JSON 指针，在一次锁定的读-改-写操作中完成。`apply-theme` 执行等效的原子主题事务，并替换主题的可选外观配置文件或打包默认值中的托管外观部分。变更命令构建一个 `NextUserConfig`，解析一个 `EffectiveConfig`，暂存所有投影器，然后才发布规范 JSON。一个操作性的 `.config-transaction.json` 标记允许 `argvus config recover` 在数据与 JSON 发布之间发生崩溃后重新投影先前的配置。主题默认值由每个主题的清单声明（`themes/<theme>/theme.toml`）：默认强调色（`palette.accent`）、GTK 颜色方案和壁纸。CSS、Lua、Rasi 和 Qt 文件从其渲染到主题构建中。

更改主题（包括明亮/黑暗模式）会将高亮颜色重置为主题的默认值，即使用户选择了自定义颜色：自定义颜色只会保留到下次主题更改。主题更改后使用 `argvus config accent` 可再次设置。应用带有 `argvus config theme-apply` 的主题会从清单中写入 `/appearance/accent` 和 `/appearance/gtk_mode`，`argvus config accent` 会在设置了 `accent_custom` 的情况下写入所选颜色，一次命令即可完成。

`apply-theme` 明确将变量视为 `--variant sticky|spaced` 并通过清单解析请求的名称，因此请求永远不会解析为另一个变量的资源。`theme-manifest THEME` 是相同的解析，以只读方式公开：它打印清单条目以及它解析到的 `asset_id` 和 `layout_variant`，这正是调用者需要的来访问打包的资产。Sticky 是未添加后缀的 ID，而只有 Spaced 带有 `-spaced` 后缀，因此 `dracula-sticky` 和 `argvus-dark-dracula-sticky` 被接受并归一化为 `dracula`。

使用 `--raw` 时，字符串值会以无 JSON 引号的形式打印，而对象和数组仍然保持 JSON 值。主题和布局点文件不再由投影助手写入；`config.json` 仍然是这些偏好的唯一持久来源。如果投影失败，会话重新加载将在重新启动桌面界面之前停止，因此不会将较旧的部分主题显示为当前主题。

## 导出和导入

个人资料交换使用相同的逻辑模型：

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
使用 `--scope desktop` 来获取完整的托管桌面模型。缺失的字段使用 ARGVUS 默认值；生成的输出不应直接编辑。

控制中心主题的导出和导入用于交换主题及其视觉配置：存档包含一个模式 2 的主题目录（`<id>/theme.toml`，可选 `appearance.json` 和 `effects.conf`，`overrides/` 即壁纸），安装在 `$XDG_DATA_HOME/argvus/themes/` 下。配置文件包含外观、布局几何、视觉效果、字体和面板呈现。音频、键盘、网络及其他非视觉偏好不包含在主题内。使用上方的 `appearance` 范围来移动完整的外观配置，使用 `desktop` 范围进行完整配置备份。

当前的模式将偏好设置分组在 `appearance`、`layout`、`effects`、`fonts`、`control_panel`、`default_apps`、`keyboard_shortcuts`、`hyprland`、`displays`、`power`、`session`、`calendar` 和 `removable_devices` 下。`default_apps` 是唯一用于首选应用程序的持久来源，而 `keyboard_shortcuts` 存储具有字符串值的稳定英语、与区域无关的键，或者用于禁用绑定的 `null`。

控制中心在打开外观页面时会读取规范的外观值，并通过 `argvus-config` 写入更改。现有的本机状态文件仍然是外观脚本和旧版安装的投影。

控制面板使用 `control_panel.cards` 来管理卡片顺序和可见性，使用 `control_panel.enabled` 来管理面板主开关。它的 `control-panel/cards.json` 和 `state/control-panel` 文件被保留作为兼容性投影。因此，在控制面板中所做的更改是 `desktop` 导出范围的一部分。

字体偏好使用 `fonts.targets` 来设置任务栏、遥测、控制面板、系统、应用程序、终端和浏览器目标的字体家族、样式和大小。`fonts.rendering` 存储抗锯齿、提示、子像素顺序和 DPI 偏好。`fonts.conf`、字体配置规则、桌面设置以及 Waybar 样式表中的管理字体块都是投影：控制中心通过 `/fonts/targets/*` 到 `argvus-config` 持久化，投影器仅重写其界定的块。

Hyprland 首选项使用 `keyboard_shortcuts` 进行快捷键设置，`hyprland.input` 进行鼠标和触控板设置，`hyprland.keyboard` 进行 XKB 布局、变体和选项设置。清单绑定 ID 映射到一个稳定的 `config_key`（例如 `window.close` → `close_window` 和 `window.drag_mouse` → `drag_window__floating_window_only`）。显示首选项使用 `displays.monitors`、`displays.workspaces` 和 `displays.primary_monitor`。位于 `data/generated/` 下的原生 TOML 文件和生成的 Lua 文件仍然是 Hyprland 使用的投影。

每个 `effects` 开关默认启用（动画、模糊、小部件遥测和每个透明表面），即使在虚拟机和低功耗会话中也是如此；窗口阴影（`shadow_enabled`）和摆动（`wobble_enabled`）默认关闭。效果偏好使用 `effects` 作为动画主开关（`animations`，应用于 `hyprland.lua` 的投影 `hypr/state.lua`中，与效果开关无关，效果开关为 `blur_global_enabled` 加上 `transparency_<surface>_enabled` 的 `taskbar`、`control-panel`、`widget-telemetry`、`terminal` 和 `launchers`，并且当其中任何一个启用时显示为启用；`argvus effects-toggle animations|effects` 可以切换它们），全局模糊开关（`blur_global_enabled`）、Hyprland 模糊参数（`blur_size`、`blur_passes`、`blur_brightness`、`blur_noise`、`blur_contrast`、`blur_vibrancy`、`blur_vibrancy_darkness`）、窗口阴影（`shadow_enabled`、`shadow_sharp`、`shadow_color`, `shadow_range`、`shadow_render_power`、`shadow_offset_x`、`shadow_offset_y`、`shadow_scale`)，动画值（`animation_<leaf>_speed`、`_curve`、`_style`、`_popin` 和 `_enabled` 对每个动画，以及 `animation_bezier_x1` 到 `animation_bezier_y2` 和 `animation_spring_mass`、`animation_spring_stiffness` 和 `animation_spring_dampening` 对于自定义曲线）以及每个表面的透明度值。效果归主题所有，而非变体所有：Sticky 和 Spaced 使用相同的 `<theme>.conf` 效果投影，而它们的布局投影不同。当前表面包括 `taskbar`、`control-panel` 和 `widget-telemetry`；透明度值的百分比范围从 `0` 到 `100`（控制中心和启动器表面默认为 `15`，其他表面默认为 `50`），模糊参数为 `schema.json` 中声明范围内的数字，启用状态为布尔值。动画值也由 `argvus-config` 验证：速度为 从 `1` 到 `10` 的数字（允许使用小数），弹出百分比是从 `1` 到 `100` 的整数，曲线必须是动画页面中的名称，样式必须是动画页面中的名称并被动画自己的叶子接受（例如 `popin`对于 `animation_workspace_in_style` 被拒绝，`gnomed` 仅在窗口打开和关闭时被接受，并且幻灯片变体在窗口和图层叶上被拒绝），贝塞尔 Y 点保持在 `-1` 和 `2` 之间，Hyprland 0.56.2 解析范围，其他曲线参数保持在其记录的范围内。 `layers` 和 `switching` 叶子不再存在，因此在删除之前保存的配置在加载时会丢失 `animation_layers_enabled`、`animation_layers_speed`、`animation_layers_curve` 和 `animation_switching_*` 密钥。当前集合之外的样式将被删除，并且 `-1` 到 `2` 之外的 Bézier Y 点将被钳制，因此打包的默认值再次应用于 无论移除了什么。

电源偏好设置使用 `power.screen_off_minutes`、`power.lock_minutes`、`power.keep_awake`，以及盖子/电源按钮策略。投影表单是 `data/generated/hypr/hypridle.conf`，标记是 `data/power/keep-awake`；`.keep-awake` 仅用于迁移输入。`session.language` 已停用：现有值将被忽略，新写入会被拒绝，并且不会投影语言兼容性文件。请改为在用户 locale.conf 中设置 `LANG`。日历偏好设置使用  `locale.conf` `calendar`，并投影到任务栏日历用户 TOML。可移动设备偏好设置使用 `removable_devices`；可移动设备应用程序将此部分视为对打包默认值的用户覆盖。

`migrate` 会导入旧的根文件和部分一次性内容，同时保留现有的规范值。旧版的 `defaults.json`、`/defaults`、`/hyprland/keybindings`、`keybindings.toml`、点文件、`generated/` 和组件目录会被移动到 `data/` 下；重复重载不会重新创建旧的根布局。再次选择当前主题是幂等的：如果规范状态和投影已经一致，投影计划会报告没有更改的部分，并且会话会跳过启动画面和服务重启。

电源遵循相同的投影规范：控制中心在 config.json 中写入 `/power`， `config.json` `argvus config project` 原子地渲染 `data/hypr/hypridle.conf`，只有在此之后 `argvus sessionctl` 才可能重启 `argvus-hypridle.service`。删除该文件可以通过 `argvus config project` 恢复；`0` 的值会省略相应的超时并表示永不。

锁屏外观由 Rust 从规范主题、强调色、特效、壁纸和字体状态投射到 `data/hypr/hyprlock.conf`。历史的 `hyprlock-theme.sh` 入口点仅用于兼容性，并委托给 `argvus config lock apply`；`--invalidate` 仅使生成的模糊壁纸缓存失效。

电源超时更改使用 `/power/lock_minutes` 和 `/power/screen_off_minutes` 作为其可信源。`argvus config project` 复用现有的 Hypridle 渲染器，而 `idle-timeout.sh` 入口点仅是兼容适配器；服务重启仍委托给 `argvus sessionctl`。

通知外观由规范主题、强调色和特效派生到 `data/notifications/dunstrc`。历史通知 `theme.sh` 文件仅公开调用 `argvus config notifications` 的兼容性函数；它不解析或更改 Dunst 配置。

面向硬件的适配器，如显示器检测和蓝牙控制，现在由 Rust 显示领域导入：`nwg-displays` 是外部导入器，而 `/displays` 和 `data/generated/hypr/monitors.lua` 分别保持为规范状态和派生状态。显示工具通过 `argvus config monitor-set`、`monitor-primary`、`monitor-workspaces` 和 `monitor-remove` 更改布局；显示命令是 `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`。Hyprland 文件从 `/displays` 投影；不再由任何 shell 脚本编写。壁纸选择、任务栏实用程序组模式和默认应用程序查找也使用 `argvus-config` API；它们的历史 Shell 入口点是 UI 或兼容性包装器。保持唤醒是持续的 `/power/keep_awake` 偏好，投影到 `data/power/keep-awake` 标记，而锁定 DPMS 是会话运行时状态。
`$XDG_CACHE_HOME/argvus/lock-dpms`。键盘布局现在由 Rust 拥有
`argvus-keyboard-layout` 助手：配置的顺序来自生成的 Hyprland 输入投影或本地回退，而活动布局保持从 Hyprland 发现的会话运行时状态。循环入口点和键盘布局守护进程只是兼容性包装器；用户服务直接运行 Rust 守护进程，`argvus sessionctl keyboard-layout cycle` 是调用方的稳定命令。

蓝牙运行时状态遵循外部适配器边界，但不属于 `argvus-config`：`argvus bluetooth` 拥有围绕 `bluetoothctl` 的类型化解析和验证、有限重试、系统服务检查以及 RFKill 状态。`argvus bluetoothctl` 是入口点。适配器电源、配对设备和已连接设备仍然是 BlueZ 运行时状态，而不是规范的 JSON 状态；硬件实时蓝牙验证依赖于环境。
## 运行时身份和调和

使用 `argvus config status --verbose` 检查确切的二进制文件、构建身份、配置根、生成根、资产根、清单源和活动生成。它报告 `synchronized`、`stale`、`incomplete`、`legacy` 和 `pending-transaction` 投影状态。

`argvus config --version` 报告包版本、构建修订版本（由构建环境提供）以及配置文件。开发测试必须显式注入它们的二进制文件和资源；它们不能默默使用 `argvus config` 或 `~/.config/argvus`。`apply` 可以安全地将诸如 `argvus-dark-spaced` 的旧组合标识符规范化为独立的主题和变体值，然后再重新构建投影。在此版本中，打包的资源标识不会记录在清单中。

## 夜间模式

打开 **桌面与输入 > 显示 > 夜间模式** 或运行 `argvus control-center displays night-light`。通过 `argvus config display night-light` 配置色温、手动激活和固定的本地计划。有关命令、状态和运行时要求，请参见 [Night Light](/docs/user-guide/night-light/)。

### 摇摆设置（Hyprland 0.57+）

`effects.wobble_enabled`、`wobble_stiffness`、`wobble_damping`、`wobble_mass`、`wobble_intensity` 和 `wobble_mesh` 项目直接投影到 `hypr/state.lua`。使用 `argvus effects-toggle wobble-settings get` 来读取它们，或者使用 `argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8` 来保存并请求重新加载。`enabled=disabled` 用于关闭该效果。

控制中心共享动画草稿：`animations-settings get` 还返回 `wobble_*` 条目，并且 `animations-settings set` 接受这些全名 alongside 它现有的动画参数。这会在一个经过验证的补丁中持久保存整个草稿并请求一次重新加载。例如：

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
全球 `enabled` 控制动画；`wobble_enabled` 是其独立的偏好设置。禁用全局动画会在 Lua 中抑制晃动，但不会删除该偏好设置。无效值会拒绝整个补丁。缺失的键会采用 ARGVUS 默认值，但不会替换现有选择。请参阅 [Appearance](/docs/user-guide/appearance/#window-wobble) 获取范围和默认值。

### 用户创建的键盘快捷键

`/keyboard_shortcuts` 对官方清单操作保留其现有的字符串或空值。自定义全局键盘快捷键使用单独的 `/hyprland/custom_shortcuts` 对象，存储在 `config/hyprland.json` 中。其稳定的 `custom_` ID 独立于描述和语言环境。每条记录都有 `description`、`keys`、`enabled` 和一个类型化的 `action`：

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
该示例是 `/hyprland/custom_shortcuts` 的值，而不是完整的配置文档。调度器名称来自受支持的 Lua 调度器工厂；参数是可选的 JSON 对象或 `null`，永远不是可执行代码。自定义快捷方式没有子映射、鼠标绑定或高级标志。投影将 `data/generated/hypr/custom-shortcuts.lua` 写为转义的数据表，包括在最后一条记录被移除时的空表。完整投影和 `project --only keyboard_shortcuts` 都会生成它。Hyprland 使用 `hl.bind` 注册已启用的条目；可选文件和单独注册失败都会被捕获并记录。

控制中心使用一个 `argvus config patch` 事务创建/编辑记录并禁用已确认的冲突。它在通过现有配置服务应用之前进行保存。应用失败不会撤销已成功保存的内容。恢复官方默认设置会保留自定义记录，如果会与当前活动的自定义快捷方式冲突则会被拒绝。用户描述在所有语言环境和生成的有效备忘单中都保持为文字原貌。

## 全局窗口模式

打开 **控制中心 → 桌面与输入 → Hyprland → 窗口 → 模式**，并选择 **平铺窗口（默认）** 或 **浮动窗口**。任务栏实用程序组中的窗口模式图标也可以使用相同的选择器。

该偏好适用于每个显示器和工作区，并在注销后仍会被记住。浮动会将现有应用程序窗口和新打开的窗口转换为浮动。平铺仅恢复由此策略转换的窗口；已经处于浮动状态的对话框和其他窗口保持浮动。在浮动激活时明确固定的窗口也保持浮动，以保留其固定状态。平铺布局会重新计算；其之前的具体分割树不会被恢复。

`SUPER + SHIFT + Space` 在平铺模式下仍保持其正常的每窗口行为。在浮动模式下，它不会将窗口切换回平铺。面板、任务栏和通知是层壳表面，不受影响。全屏和最大化仍然使用 Hyprland 的正常行为。

规范首选项是在 $XDG_CONFIG_HOME/argvus/config/hyprland.json 中的 `/hyprland/window_mode`。它生成的投影是  `$XDG_CONFIG_HOME/argvus/config/hyprland.json` `data/generated/hypr/window-mode.lua`；请不要编辑它。每个窗口的恢复注册表位于 `$XDG_RUNTIME_DIR/argvus` 下，作用域为合成器实例，并且在配置重新加载时仍然存在。它不会在新会话中重复使用。

如果应用程序正在等待或失败，界面会将其与保存的首选项区分开。再次选择该选项以重试。没有合成器时，首选项会保存到下次登录。

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## 窗口按钮和应用程序栏

浮动模式为普通应用程序窗口添加了带有 **最小化**、**最大化 / 还原** 和 **关闭** 的 ARGVUS 标题栏。双击标题栏可切换最大化。在平铺和全屏模式下，附加的装饰会被隐藏。自绘控件的应用程序可能会同时显示两个工具栏；统一的 ARGVUS 控件是有意为之。

每个显示器上都会出现第二个 Waybar，位于主任务栏的相对边缘。它只列出属于该显示器当前工作区的窗口，包括该工作区已最小化的窗口。已打开的特殊工作区具有优先权。固定窗口保持可见；隐藏的组标签不会产生额外条目，最小化的组只有一个恢复条目。应用程序内的标签不会单独列出。最小化的条目保持其原始的显示器/工作区关联。点击非活动窗口以激活它，点击活动窗口以最小化它，或点击已最小化的窗口以恢复它；其上下文菜单提供恢复、最小化、最大化/恢复和关闭选项。未知的应用图标会退回使用窗口符号。激活或恢复条目时会保留指针位置。辅助栏使用有效的主 CSS（颜色、字体、边框和圆角），其高度和在相对边缘的镜像边距也会同步。

`SUPER + Alt + Up` 将主栏放在顶部，窗口栏放在底部。`SUPER + Alt + Down` 则相反。重复一个方向将保持位置不变。平铺模式只有主栏。返回到平铺模式会在移除窗口栏之前恢复所有最小化的窗口，然后保留原来的平铺/浮动策略。

最小化使用保留的 `special:argvus-minimized` 工作区。不要手动使用该工作区。`argvus windows service` 进程在特定于合成器实例的运行时注册表中，在 `$XDG_RUNTIME_DIR/argvus` 下记录身份、原始工作区/显示器、几何形状、全屏/最大化和固定状态。记录在服务重启或配置重新加载后仍然存在，但不会跨合成器会话。关闭窗口会删除其记录。如果显示器消失，恢复将使用可用的输出。缺失或恢复条失败会阻止新的最小化；恢复失败会保留记录和恢复条。

会话拥有 `argvus-windows.service` 和 `argvus-window-bar.service`。后者保持其 Waybar 子项在 Tile 中处于休眠状态。hyprbars 插件由 `argvus-hyprland` 打包，为 x86_64 上的 Hyprland 0.57.0 ABI 构建；它未在用户的主目录中编译。更新合成器 ABI 后重新构建该插件。命令：`argvus windows list --json`、`watch --json`、`position top|bottom`、`restore-all` 和 `minimize|restore|activate|maximize|close ADDRESS`。窗口地址来自列表。`position top --persist` 存储/投射偏好设置而无需重启主栏。配置和运行时错误详情会记录到日志中；可见的操作失败也会显示翻译后的通知。

在 **Spaced + Floating** 模式下，窗口会遵循外部间隙和两个栏位预留的区域。边界包括边框和标题，标题位于同一窗口边框内。通过 ARGVUS 控制最大化时，会用边距填充该区域；恢复时会拾回之前的几何状态，包括服务重启后。全屏模式仍使用整个输出区域。浮动窗口可能重叠；铺砖式内部间隙不会将它们分隔开。

由工具栏保留的内边距不会在窗口外间隙中被重复计算。使用对称间距时，可见的窗口与工具栏的距离上下相等，包括在交换工具栏之后。

`SUPER + Alt + Space` 切换全局平铺/浮动模式，使用与任务栏和控制中心相同的偏好设置。等效命令是 `argvus config window-mode toggle`。并发调用会被顺序处理；即使应用程序待处理时，切换也会使用保存的偏好设置。快捷键可以在键盘快捷键中更改或禁用；`SUPER + Shift + Space` 保持其单窗口操作。
