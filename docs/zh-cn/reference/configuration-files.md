---
title: 配置文件
description: 用户、系统、包默认和生成的 ARGVUS 配置。
---

| 层 | 位置 | 含义 |
| --- | --- | --- |
| 系统 | `/etc/argvus/` | 服务和提供者的管理员配置。 |
| 包默认值 | `/usr/share/argvus/` | 只读组件默认值和运行时资源。 |
| 用户 | `$XDG_CONFIG_HOME/argvus/config.json` | 标准可移植偏好设置。 |
| 受管理数据 | `$XDG_CONFIG_HOME/argvus/data/` | 组件数据、兼容性文件和内部元数据。 |
| 生成 | `$XDG_CONFIG_HOME/argvus/data/generated/` | `config.json`的投影，仅由`argvus-config`写入。切勿编辑，切勿复制到其他地方作为配置，切勿作为源读取回。 |
| 运行时 | `$XDG_RUNTIME_DIR/` 和 `$XDG_CACHE_HOME/` | 套接字、锁和临时状态。 |

活动的主题、强调色、自定义壁纸和效果状态都是逻辑用户状态。受管理的逻辑模型记录在 [Canonical configuration](/docs/reference/argvus-config/)] 中。生成的 Hyprland、GTK、Qt6ct、Waybar、Quickshell、Dunst、Hyprlock、Yazi、Superfile、终端、字体和可移动设备文件不应被视为独立的真实来源。

`argvus-config` 还将主题层镜像到 `data/waybar/argvus-taskbar.{jsonc,css}` 和 `data/waybar/argvus-widget-telemetry.{jsonc,css}`。外观发生变化时，这些副本会从打包的默认值中替换，然后重写其界定的受管理区块，因此应将它们视为派生文件，而非手工维护的文件。`$XDG_CONFIG_HOME/<app>` 下的本地应用程序配置不同：它是由外观协调器编写的适配器目标，从不由投影器编写。

## 单一编写者规则的例外

`argvus config` 是 `data/generated/` 的唯一作者。迁移存在一个例外：当创建打包文件的用户副本时，一个旧版本在 `data/generated/` 下遗留的文件会被移动到它的用户位置（`data/`），这样用户可以保留他们的编辑。此移动仅适用于当前版本未生成的文件。不要依赖此路径来存放新文件：应在投影中添加生成的文件，而不是在这里。
