---
title: 夜灯
description: 在 Hyprland 上调整色温和固定本地计划。
---
打开 **控制中心 > 桌面与输入 > 显示 > 夜光**，搜索 **夜光**，或运行 `argvus control-center displays night-light`。

夜光及其计划默认关闭。从 **1000 到 20000 开尔文** 选择一个温度（默认 **4500 K**）。较低的值会产生更暖的颜色。页面显示保存的手动激活、实际状态及下一次切换。输入可编辑值；Esc 可取消或返回显示。无效值在保存前会被拒绝。更改会立即保存并应用。如果保存后应用失败，保存的偏好仍会与实际状态一同显示，并显示错误信息。

## 计划与快速操作

固定计划使用当前当地时区。开始时间包含在内，结束时间不包含；**20:00–07:00** 跨越午夜。开始和结束时间必须不同，并使用 `HH:MM`。未使用位置或太阳计算。

使用控制面板的**显示**卡或按 **SUPER + Shift + B** 进行快速切换。亮度选择器保留所有亮度操作，并在后端可用时添加夜间模式，即使是在没有亮度后端的显示器上也可以使用。没有永久的任务栏模块。

在关闭调度的情况下，手动激活是持续的。开启调度后，快速切换会在下一次计划转换之前创建运行时覆盖。更改调度或温度会清除此覆盖。注销、新的合成器实例或下一个计划转换的变化（包括时区变化）也会使其无效。

## 命令和状态

所有接口使用相同的公共命令：

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status` 返回 JSON 架构版本 1：`available`, `unavailable_reason`, `settings`（启用、温度、计划启用、开始、结束）、`active`、`effective_temperature`、`mode`（`manual` 或 `scheduled`）、`manual_override`、`next_transition`（带偏移的 ISO 8601）、`minimum_temperature` 和 `maximum_temperature`。不可用时，有效字段为 null；当身份处于激活状态时，有效温度为 null。可用性失败会在状态中报告；操作失败返回非零并附带诊断信息。

规范偏好是 `/displays/night_light`，由 `argvus config` 在 `$XDG_CONFIG_HOME/argvus/config/displays.json` 中原子存储。易失性覆盖和操作锁位于 `$XDG_RUNTIME_DIR/argvus/night-light/` 下。生成的监控文件是独立的。请勿编辑生成的文件来配置此功能。

## 运行时

需要 Hyprland 和 **hyprsunset >= 0.4.0**。会话会启动上游 `hyprsunset.service` 一次，通过 `argvus-night-light-init.service` 应用保存的状态，并通过 `argvus-night-light.timer` 和 `argvus-night-light.service` 每分钟评估计划。时钟和时区的更改也会触发评估。关闭调度时，定期操作不会更改手动状态。重复应用会查询有效身份/温度，并跳过未更改的写入。无需重新加载合成器。

ARGVUS 提供了上游服务的 systemd 插件：它以 `--identity` 开头，并将其配置发现隔离到 `/usr/share/argvus/session/config/night-light`。这避免了与本地配置文件的冲突，并且不会影响个人的 `hyprsunset.conf` 文件。该插件适用于上游服务，同时安装 ARGVUS 会话包时，包括在 ARGVUS 外部的启动。现有的用户 systemd 覆盖可以优先；在诊断该情况时请检查 `systemctl --user cat hyprsunset.service`。Hyprsunset 0.4.0 不接受 `--config` 的单独路径参数，因此配置发现改为通过服务本地 XDG 变量进行隔离。

后端使用官方 Hyprsunset IPC（`temperature`、`identity`、`identity get`），具有有限的 I/O 和更改后的回读功能。不使用着色器、替代合成器后端、系统配置写入或权限提升。不可用的控制在控制中心中被禁用，并在显示卡中隐藏。
