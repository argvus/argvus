---
title: ARGVUS 焦点
description: 番茄工作法和专注会话，带有休息、通知和本地会话历史记录。
---
ARGVUS Focus 是一个 GTK 4 应用程序，用于在间隔休息的专注会话中工作。它会作为弹出窗口在任务栏旁打开。可以通过启动器打开，点击任务栏指示器，通过 **SUPER + Shift + F**，或使用以下方式打开：

```sh
argvus focus              # open the popup near the pointer
argvus focus --toggle     # hide the popup if it is visible, show it otherwise
argvus focus --toggle --x 1600 --y 30  # open next to a taskbar click position
argvus focus --start      # start or resume the current phase without opening the window
argvus focus --pause      # pause the running phase
argvus focus status --waybar         # print one Waybar JSON status and exit
argvus focus status --waybar --follow  # stream status changes for Waybar
```
任务栏传递根点击坐标，因此弹出窗口使用被点击的显示器并与任务栏对齐；如果启动时没有坐标，则使用指针位置。如果指针在弹出窗口外停留5秒，它会自动关闭；将指针移入弹出窗口会取消该倒计时。主页面上的 **Esc** 仍会立即隐藏它，而子页面上的 **Esc** 会返回到前一页。任务栏指示器在空闲时只显示其图标，而在某阶段显示剩余的 `MM:SS` 并带有该阶段颜色；暂停阶段保留其剩余时间并添加暂停状态。`status` 命令在 `argvus` 内部运行而不启动 GTK。`argvus --focus` 是别名。每个会话中只运行一个 Focus 实例：第二次启动会将其选项传递给正在运行的实例。

ARGVUS Focus、ARGVUS Clock 和控制面板共享相同的屏幕边缘。打开 Focus 会首先关闭 Clock 和控制面板；打开其中任何一个界面都会关闭 Focus。

## 会话

- **番茄钟模式** 交替进行专注和短暂休息，每个循环结束后有一次长休息。默认时间长度为 25 分钟（专注）、5 分钟（短休息）和 15 分钟（长休息），每进行 4 个专注周期后会有一次长休息。
- **单次专注模式** 运行一个单独的专注会话，持续其自身长度（默认 45 分钟），且没有休息。
- 窗口显示当前阶段（专注、短休息、长休息，每个阶段有不同颜色）、进度圈内剩余时间、循环中的位置以及今日总计时间。
- **开始专注 / 开始休息**、**暂停**、**恢复**、**取消会话**（放弃当前阶段，不记录任何时间）以及 **立即完成**（提前结束专注会话并记录实际花费的时间）。休息时，**跳过休息** 会返回专注。
- 当一个阶段结束时，下一个阶段会等待你，除非已启用 **自动开始休息** 或 **休息后自动开始专注**。

键盘：**空格** 开始或暂停，**Ctrl+,** 打开设置，**Ctrl+H** 打开历史记录，**Esc** 返回，**Ctrl+W** 隐藏窗口，**Ctrl+Q** 退出。

## 设置

预设（经典 25/5/15，扩展 50/10/30，深度工作 90/20/30）填充时长，且可继续编辑。设置还包括长休前的工作次数、自动启动、阶段结束通知，以及**会话声音**：一个短铃声，可以单独调节音量，并带有**测试声音**按钮。这些声音与 [ARGVUS Clock](/docs/components/argvus-clock/)] 的闹钟声是分开的。

## 历史记录

每个完成的专注会话都会记录开始时间、结束时间和花费时间；休息和取消的会话不记录。历史页面显示当天的会话和分钟数，总计和最近会话。超过 365 天的记录会被删除。**清空历史记录** 会要求确认。

## 窗口、后台和挂起

- 在周期进行中关闭窗口只是将其隐藏：会话仍在继续，并在阶段结束时通知你。如果没有任何进程在运行，关闭窗口会退出。
- 如果在阶段运行时系统挂起，该阶段会在挂起时暂停，并显示系统休眠了多长时间。休眠时间不计入专注时间。
- 会话会在每次更改和每30秒保存一次。如果在阶段进行中关闭 ARGVUS Focus（登出、崩溃、**Ctrl+Q**），下次打开时会在该点暂停；它从不假装在无法通知时计时器仍在运行。
- 时间是按系统启动时间的时钟测量的，所以更改墙面时钟或时区不会缩短或延长阶段。

## 路径

| 路径 | 内容 |
| --- | --- |
| `$XDG_CONFIG_HOME/argvus/data/focus/settings.json` | 设置 |
| `$XDG_DATA_HOME/argvus/focus/history.json` | 完成的会话 |
| `$XDG_STATE_HOME/argvus/focus/session.json` | 当前会话的检查点 |
| `/usr/share/argvus/sounds/argvus-bell.oga` | 会话提示音 |

文件以原子方式写入。无法读取的文件会被重命名为 `<name>.corrupt-<time>`，并使用默认值；系统会提示文件的位置。

颜色遵循当前活动 ARGVUS 主题和高亮颜色（主题使用者 `time`）。字符串来自 `focus` 和 `time` 翻译域。

任务栏日历、ARGVUS 焦点、ARGVUS 时钟和控制面板互斥：打开其中一个会关闭其他所有组件。
