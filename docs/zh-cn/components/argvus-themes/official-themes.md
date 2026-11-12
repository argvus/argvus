---
title: 官方主题
description: 官方 ARGVUS 主题目录、默认高亮颜色，以及如何选择主题及其固定或间隔模式。
---
此页面列出了可作为主题包提供的官方主题，并解释了如何选择主题。有关安装它们，请参见 [Installing themes](/docs/components/argvus-themes/installing-themes/)。

## 选择主题

1. 打开 **控制中心 → 外观 → 主题**。
2. 选择 **官方主题**。
3. 选择 **深色** 或 **浅色**。
4. 选择主题系列。选定主题后会立即应用。

所选主题会立即应用到整个桌面：正在运行的任务栏、面板、通知、窗口和壁纸会实时更改颜色，无需加载屏幕，也无需重启。ARGVUS 会保留您的壁纸选择，除非主题自带其壁纸关联。

您也可以按 `SUPER + SHIFT + T` 从键盘打开主题选择器。使用箭头键在分类间移动，按 `Enter` 选择，按 `Esc` 关闭。

## 粘性与间隔

每个主题都有两种模式。模式与主题无关：更改主题会保持当前模式，更改模式会保持当前主题。

| 模式 | 窗口间隙 | 外部间隙 | 窗口圆角 | 任务栏和系统边距 |
| --- | ---: | ---: | ---: | ---: |
| 固定 | `2` | `0` | `0` | 靠近边缘 |
| 间隔 | `10` | `18` | `4` | `18` |

要切换模式，请打开 **控制中心 → 外观 → 模式**，然后选择 **固定** 或 **间隔**。您对这些布局值的手动调整将在下一次显式选择主题之前保持有效。

## 暗色主题

| 主题 | 套餐 | 默认高亮 |
| --- | --- | --- |
|一暗 | `argvus-theme-one-dark` | `#61AFEF` |
|德古拉 | `argvus-theme-dracula` | `#BD93F9` |
| 银暗 | `argvus-theme-silver-dark` | `#595959` |
| Slate 暗 | `argvus-theme-slate-dark` | `#7391A5` |
| 宇宙 | `argvus-theme-universe` | `#EEEEEE` |
| Gruvbox 高暗 | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| Gruvbox 暗 | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| 玫瑰松 | `argvus-theme-rose-pine` | `#C4A7E7` |
| 东京之夜 | `argvus-theme-tokyo-night` | `#7AA2F7` |
| 孤寂 | `argvus-theme-solitude` | `#798186` |
| 日落 | `argvus-theme-sunset` | `#E2BE8A` |
| 矩阵 | `argvus-theme-matrix` | `#82FB9C` |
| Monokai 暗 | `argvus-theme-monokai-dark` | `#78DCE8` |
|雾女士 | `argvus-theme-lady-mist` | `#354B78` |

## 亮色主题

| 主题 | 包 | 默认高亮 |
| --- | --- | --- |
| GitHub 灯 | `argvus-theme-github-light` | `#0969DA` |
|日光灯| `argvus-theme-solarized-light` | `#268BD2` |
|北欧之光 | `argvus-theme-nord-light` | `#5E81AC` |
|一盏灯| `argvus-theme-one-light` | `#4078F2` |
|永林之光 | `argvus-theme-everforest-light` | `#3A94C5` |
|卡布奇诺拿铁| `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox 灯 | `argvus-theme-gruvbox-light` | `#458588` |
|水彩| `argvus-theme-watercolor` | `#1A639C` |
|黑曜石灯 | `argvus-theme-obsidian-light` | `#334A78` |

## 内置主题

**ARGVUS Dark** 和 **ARGVUS Light** 由 `argvus-appearance` 提供，并始终在 **官方主题** 下可用。ARGVUS Dark 使用默认高亮 `#3590BD`；ARGVUS Light 使用 `#181818`。

## 主题控制内容

主题定义了桌面组件使用的调色板和表面处理、默认高亮颜色、默认壁纸关联以及透明度和模糊默认值。桌面组件读取此状态，而不是存储自己的副本，因此主题更改会同时影响合成器、任务栏、控制面板、锁屏、通知和应用程序。

默认高亮颜色只是起点。您可以将其替换为任意六位数颜色，详情请参见 [自定义高亮颜色](/docs/components/argvus-themes/highlight-color/)。
