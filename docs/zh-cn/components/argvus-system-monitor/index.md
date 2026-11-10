---
title: 系统监视器
description: 打开 ARGVUS 系统监视器。
---
使用以下方式打开已配置的监视器：

```sh
argvus --system-monitor
```
`argvus system-monitor` 子命令提供本地入口点和 btop 集成。任务栏中显示的遥测是由 `argvus widget-telemetry` 提供的独立功能。点击其 CPU、内存和 GPU 模块，会在 ARGVUS TUI 面板中打开带有品牌标题和 `Esc Close` 页脚的 btop。GPU 模式显示最多六个 GPU 框，不显示 CPU、内存、网络或进程框。按下 `q` 或 `Esc` 关闭面板。底部的 Kitty 标签显示本地化的 `Press q to close` 提示。独立的 `argvus system-monitor --cpu`、`--mem` 和 `--gpu` 调用会直接启动带有所选框的 btop。

GPU 面板取决于已安装的 btop 构建版本的功能以及系统的设备提供程序。NVIDIA 监控需要 btop 的 GPU 支持、已安装 NVIDIA 驱动的 NVIDIA 管理库，以及对设备的访问权限。AMD 监控使用 ROCm SMI，这是可选的，可能未安装。Intel GPU 数据从 sysfs 读取，取决于用户对这些接口的访问权限；btop 1.4.7 报告 Intel 的使用率、电源和时钟，但不提供 Intel 温度。NVIDIA nouveau 和其他驱动可能通过与设备关联的 hwmon 将 GPU 温度显示在任务栏上，而 btop 的提供程序支持取决于可用的驱动和库。ARGVUS 不会更改设备权限或以提升权限运行 btop。如果没有 GPU 被识别，或者 btop 在三秒内未渲染 GPU 框，因为缺乏支持或提供程序不可用，面板将显示本地化的不可用信息。

监视器首先从 `$XDG_CONFIG_HOME/argvus/config.json` 读取所选的主题和强调。它生成的 btop 文件保持派生缓存状态；`.active-theme` 和 `.accent-color` 是兼容性回退。因为 btop 保留了自己的主题文件，监视器通过主题事务（`argvus theme-switch`）作为外部适配器在规范更改提交后进行调和；它不是生成树的一部分。

## 运行时契约

- 主题和强调色来自规范配置（` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme，`.accent-color` 是备用选项）。
- 生成的 btop 配置和主题写入 `$XDG_CACHE_HOME/argvus/argvus-system-monitor`。它们是派生状态；请勿编辑。
- 默认值安装在 `/usr/share/argvus-system-monitor/btop` 下。
- 模块快捷方式（`--mem`、`--cpu`、`--net`、`--proc`）在作为第一个参数时为只读。
  传入 btop 自带的 `-c`/`--config` 或 `--themes-dir` 会使用这些路径运行 btop，但不会写入任何配置。
- `--panel [--cpu|--mem|--gpu]` 将 btop 嵌入 ARGVUS TUI 外壳。此模式由任务栏使用；
  如果没有 `--panel`，监视器会继续用 btop 替换自身。
