---
title: 小部件遥测
description: Waybar 的遥测模块及其切换，通过 argvus widget-telemetry。
---
遥测小部件是显示在任务栏中的系统信息（标题、键、系统、CPU 和 GPU、内存、存储、进程、网络和开发者仪表板）。它位于 `argvus` 包中：没有外壳收集器。

## 命令

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block` 是任务栏的高频路径：它在每次刷新时都会被调用。`toggle` 更改任务栏显示的块并移动滚动窗口。

这些块是 `header`、`keys`、`system`、`cpu_gpu`、`memory`、`storage`、`processes`、`network` 和 `dev_dashboard`。任务栏还使用 `cpu-temp` 和 `gpu` 探针。GPU 探针接受 `--present`（DRM/sysfs 中可见的受支持 GPU）、`--available`（面板可用的 GPU 和 btop 命令）、`--available-temp`（可以读取的可归属温度）以及 `--temp`（最高有效温度，或 `N/A`）。

GPU 温度与 GPU 的 PCI 设备匹配。NVIDIA 专有读取使用 `nvidia-smi`/NVML 总线 ID；NVIDIA nouveau、AMD 和 Intel 读取仅在其 sysfs 设备链接唯一匹配该 GPU 时使用 hwmon。任务栏温度是检测到的 GPU 中的最大有效值。缺失、无效或模糊的读取将保持不可用。NVIDIA 专有温度需要驱动提供者；AMD 和 Intel 温度取决于内核是否暴露了关联的 hwmon 传感器。Intel 温度未由 btop 1.4.7 的 GPU 面板提供。

## 会话启动

`argvus-widget-telemetry.service` 用户单元仅在小部件启用时随会话启动。主开关是配置中的 `effects.widget_telemetry_enabled`，默认启用，因此新账户会在登录时启动小部件。

一旦切换器写入 `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`，该文件就会决定。只有 `enabled` 的第一行会启动小部件。较旧的 `desktop-telemetry-state` 和 `sysinfo-state` 文件在当前文件缺失时仍然会被读取。

遥测标签在所有十种支持的语言中都是紧凑的。诸如 CPU、GPU、IP、DE、WM 和 Kernel 等技术标识符保持其技术含义。正常运行时间从 `/proc/uptime` 读取，并以缩写单位显示（例如 `1d 12h 49m`）；不再根据系统语言环境扩展为完整单词。硬件型号名称和其他提供者的值仍然可以影响小部件的宽度。
