---
title: 配置与状态
description: 共享状态、覆盖和生成的运行时文件。
---
普通用户 root 是 `$XDG_CONFIG_HOME/argvus`（通常是 `~/.config/argvus`）。托管的逻辑首选项存储在 `config.json`。主题、强调色、壁纸、间距和边框首选项在此是标准定义；点文件仍然是为当前 Hyprland Lua 和 shell 使用者提供的派生兼容性投影，同时这些使用者迁移到 `data/`。

`argvus-config` 是 `generated/` 之下的唯一编写者。每个 ARGVUS 拥有的消费者文件都是 `config.json` 的投影，包括 Hyprland、GTK、Qt6ct、Waybar、Quickshell、Dunst、Hyprlock、Waybar 任务栏和小部件遥测配置文件、Yazi 和 Superfile 配置树、终端和 Qt 配置树、字体管理块以及可移动设备样式表。组件首先解析显式用户覆盖，然后是生成的输出，最后是包默认值。Rofi 不是投影：`argvus-launcher` 拥有其文件，而用户副本则实时存在于 `data/rofi/` 下。正常的主题/重新加载投影从不会将生成的或打包的文件提升为用户覆盖。文档和工具应识别逻辑来源，而不是要求用户编辑生成的输出。

`argvus-appearance` 因此拥有主题和重音编排，而 `argvus-config` 拥有规范状态和结构化投影。`argvus theme-switch`（`argvus config theme-apply` 事务）和 `argvus config accent` 在可用时通过 `argvus-config` 进行验证、锁定和提交，然后调和无法直接读取 `config.json` 的兼容性消费者。这些适配器包括原生 Qt6ct 文件、GTK `settings.ini` 和 `gsettings` 提示、终端和系统监控应用程序、foot、superfile 自身的配置目录以及预认证问候界面。需要新适配器的助手必须将其添加到那里，绝不能放入 `generated/` 内。

运行时所有权是根据职责故意划分的：Rust 拥有规范存储、架构验证、迁移、路径模型、锁、原子写入、投影哈希和结构化渲染器。Shell 仅作为针对外部消费者（如 `hyprctl` 和资产拥有的主题脚本）的少量兼容性入口或适配器存在。Python 保留用于测试、审计和开发辅助；它不是关键桌面运行时的一部分。

`argvus config apply` 是重建边界（`project` 保持兼容拼写）。它会在规范文档未更改时重新生成缺失的输出，在暂存中渲染投影器，并仅在每个投影成功后才原子性地交换 `data/` 树。会话生命周期仍由 `argvus sessionctl` 拥有，它使用该清单来计划有针对性的重新加载。

当调用者已经提交了一个生成——`apply-theme`、一个口音更改、一个`set`/`patch`，或者一个 widget-telemetry 块更改——`reload` **应用**该生成，而不是再次投影它。它从清单中读取计划，恢复目标消费者，仅在清单不存在或没有待处理工作时才进行投影。在那时重新投影将重建一个相同的树并丢弃突变刚刚生成的计划。

清单还记录操作生成、有效配置哈希、已更改的域和生成的文件列表。`data/internal/generation.json` 是一个一次性本地投影标记，永远不是用户偏好。`argvus config status` 将这些记录与解析的配置进行比较，并报告缺失或多余的生成文件。

突变遵循相同的引擎边界：`UserConfig` 被克隆到 `NextUserConfig`，经过验证，解析为 `EffectiveConfig`，投影，然后才被持久化。`.config-transaction.json` 是操作恢复状态；`argvus config recover` 在发布 `data/` 和 `config.json` 之间发生崩溃后，确定性地重新投影之前的规范文档。只有在提交之后才会请求重新加载，重新加载失败不会回滚一个一致的持久配置。

## 解析模型

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```
具体候选项因组件而异，但原则是一致的：用户覆盖优先，生成的输出是派生状态，打包文件保持只读输入。会话导出 `ARGVUS_CONFIG_HOME` 和 `ARGVUS_SYSTEM_CONFIG`，以便辅助程序和用户服务解析相同的根。

重新加载恢复遵循 `recover → ensure → migrate → apply → runtime reload`，当传入清单已描述提交的生成时，投影步骤会被跳过。`ensure` 仅在配置文件缺失时从模式默认值生成 `config.json`，并填充缺失字段而不替换显式的 `false`、`0` 或自定义值。现有的格式错误文档仍然被视为错误，而不会被静默替换。

持久性变更通过`ChangedDomains`比较已解析的旧配置和新配置。在提交后，通过`argvus sessionctl`发送一次重新加载请求；如果重新加载失败，会报告并记录，但不会回滚已提交的配置。会话启动在引导材料化和投影之前执行恢复，因此消费者不会在放弃的事务上启动。

| 状态 | 目的 |
| --- | --- |
| `config.json` | 规范化管理的逻辑偏好；是`generated/`以下所有内容的唯一来源。 |
| `.active-theme`, `.accent-color`, `.accent-custom` | 旧版迁移输入；不是运行时来源，也不会重新生成。 |
| `.wallpaper-custom` | `appearance.wallpaper` 的旧版迁移输入；不是运行时来源。 |
| `.spaces`, `.borders` | 旧版布局迁移输入；规范值存储在`layout`中的`config.json`下。 |
| `fonts.conf` | 规范字体选择的兼容性投影。|
| `generated/effects` | 规范效果的兼容性投影；一次性使用。|
| `$XDG_STATE_HOME/argvus/config-projection.json` | 投影清单：更改的部分、受影响的用户及是否需要重新加载。由 `reload` 读取；从不作为真源。|
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | 旧版遥测块偏好设置。仅作为迁移回退读取；规范值存储在 `/control_panel/widget_telemetry_blocks` 中，该文件不再被写入。|

当属性从 `config.json` 读取并从那里进行投影时，该属性为规范属性。当属性被投影但也可作为迁移输入使用时，该属性为派生属性。组件在运行时仍然写入的任何内容——缓存、位于 `$XDG_CONFIG_HOME/<app>` 下的本地配置文件、欢迎界面状态文件——都是适配器目标，不得被回读作为配置。

Projection 覆盖了 Hyprland、GTK/Qt、Waybar、Quickshell、终端、通知、锁屏和原生应用的消费者。Hyprland 从所选主题读取活动和非活动边框颜色；明确的 `appearance.accent_custom=true` 是唯一会用用户强调色覆盖活动/组边框的情况。主题切换通过复合器重新加载重新应用这些值，因此无法保留之前主题的颜色。如果用户配置被重新创建，效果默认为启用动画。删除 `generated/` 并运行 `argvus config apply` 会从 `config.json` 按字节重新创建它；直接编辑生成的文件不可持久。运行时诊断必须识别完整的执行元组：二进制路径、版本/修订、资源根、配置根、数据根、生成根、内部根和投影清单。`status --verbose` 暴露了此元组并
将已发布的生成/哈希与当前解析配置进行比较。开发和已安装的软件包测试是独立模式；两者都不能悄悄回退到无关的可执行文件或开发者的主目录。`apply` 也是传统组合主题标识的对账边界：诸如 `argvus-dark-spaced` 的值在解析之前会变为 `theme=argvus-dark` 和 `variant=spaced`。本版本中，打包资产的身份未记录在清单中，因此没有诊断文件描述它。
