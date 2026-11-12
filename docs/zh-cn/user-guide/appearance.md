---
title: 外观
description: 配置 ARGVUS 组件共享的视觉语言。
---
外观系统将一个主题状态应用于 Hyprland、GTK、Qt、Waybar、Quickshell、Rofi、Dunst、Hyprlock 和 ARGVUS 应用程序。

## 主题系列

下面的二十三个系列由当前主题负载定义。颜色列表示该系列默认的活动边框/强调色；用户可以选择其他有效的 RGB 强调色。

| 系列 | 重置高亮颜色 | 基础背景 | 模式 |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | 固定, 间距 |
| Dracula | `#BD93F9` | `#282A36` | 固定, 间距 |
| ARGVUS Dark | `#3590bd` | `#111316` | 固定, 间距 |
| Dark Silver | `#595959` | `#111316` | 固定, 间距 |
| Dark Slate | `#7391a5` | `#2f3541` | 固定, 间距 |
| Dark Universe | `#eeeeee` | `#000000` | 固定, 间距 |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | 固定, 间距 |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | 固定, 间距 |
| ARGVUS 光 | `#181818` | `#f7f7f7` |粘性，间隔|
| GitHub 之光 | `#0969DA` | `#FFFFFF` |粘性，间隔|
|卡布奇诺拿铁| `#1E66F5` | `#EFF1F5` |粘性，间隔|
| ARGVUS 轻型 Gruvbox | `#458588` | `#FBF1C7` |粘性，间隔|
|玫瑰松 | `#C4A7E7` | `#191724` |粘性，间隔|
|东京之夜 | `#7AA2F7` | `#1A1B26` |粘性，间隔|
|孤独| `#798186` | `#101315` |粘性，间隔|
|日落| `#E2BE8A` | `#0F0F0F` |粘性，间隔|
|矩阵| `#82FB9C` | `#0B0C16` |粘性，间隔|
|莫诺凯黑暗| `#78DCE8` | `#2D2A2E` |粘性，间隔|
|雾女士 | `#354B78` | `#131313` |粘性，间隔|
|一盏灯| `#4078F2` | `#FAFAFA` |粘性，间隔|
|永林之光 | `#3A94C5` | `#FDF6E3` |粘性，间隔|
|日光灯| `#268BD2` | `#FDF6E3` |粘性，间隔|
|北欧之光 | `#5E81AC` | `#ECEFF4` |粘性，间隔|
|水彩| `#1A639C` | `#E9E3D3` |粘性，间隔|

表格中显示的颜色是当选择该系列或使用 **重置为主题默认值** 时写入 `config.json` 的值。手动选择的六位数 RGB 强调色会一直应用，直到下一次主题切换，切换后会被所选主题的默认值替换。

**紧凑型（Sticky）** 是紧凑布局：平铺窗口使用较小的内部间隙，外部间隙为零，任务栏靠近屏幕边缘。**宽松型（Spaced）** 增加了更大的间隙、圆角、阴影，以及任务栏和界面表面的边距。

现有的 `float` 布局值和 `-float` 主题标识符被识别为宽松型的旧名称。新的布局值和解析后的主题标识符使用 `spaced` 和 `-spaced`。

模式更改几何和表面处理；它不会创建第六个颜色系列。请参阅 [Themes](/docs/user-guide/appearance/) 以进行选择，以及 [Desktop](/docs/user-guide/desktop/) 了解任务栏和窗口布局。

- [主题](/docs/user-guide/appearance/)
- [壁纸](/docs/argvus-appearance/wallpapers/)
- [字体和图标](/docs/argvus-appearance/fonts-and-icons/)
- [效果](/docs/argvus-appearance/effects/)

控制中心公开了面向用户的设置。该实现由`argvus-appearance`、`argvus-session`、外壳组件和资产包共享。参见 [外观架构](/docs/developer-guide/subsystems/appearance-and-themes/)。

## 窗口晃动

打开 **桌面与输入 → Hyprland → 动画 → 窗口** 并使用
**晃动** 控件。搜索 `wobble`，或直接打开页面：

```sh
argvus control-center hyprland animations windows
```
Wobble 默认情况下是禁用的。其 ARGVUS 默认值为：刚度 `100`、阻尼 `10`、质量 `1`、强度 `1` 和网格分辨率 `8`。刚度和质量接受 `0.0001–1000`，阻尼 `0–1000`，强度 `0–2`，网格为整数 `2–32`。更高的网格分辨率会增加渲染成本。使用箭头键调整数值或按 Enter 键输入数值。**恢复 Wobble 默认值** 会更改草稿；**应用** 会保存所有待处理的动画编辑并重新加载 Hyprland 一次，而无需重启会话。在动画中返回会保留共享草稿。

全局动画开关会在保留 Wobble 个别设置的同时抑制 Wobble。规范值位于 ARGVUS 状态下的 `effects.wobble_*`；`hypr/state.lua` 是生成的，不能手动编辑。

这需要 Hyprland **0.57.0 或更高版本**，由 `argvus-hyprland` 的软件包依赖强制要求。版本 0.56 不支持此配置。强度最大值遵循 [发布的 0.57.0 源代码](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp)，限制为 `2`，尽管较新的文档可能列出 `3`。
