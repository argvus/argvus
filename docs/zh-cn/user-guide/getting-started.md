---
title: 第一次配置
description: 配置 ARGVUS 的实用首次启动路径。
---
在第一次图形会话之后，你不需要配置 ARGVUS 的每个部分。可以从影响桌面体验的设置开始，然后根据需要配置硬件和服务。

## 一个有用的顺序

1. 阅读 [ 桌面概览 ](/docs/user-guide/desktop/)，以识别窗口、工作区、任务栏、控制面板和启动器。
2. 打开 [ 控制中心 ](/docs/components/argvus-control-center/)，并在 **外观** 中选择主题、模式、强调色和壁纸。
3. 如果默认的间距、边框或任务栏间距不适合你的屏幕，请调整 [ 窗口和布局 ](/docs/argvus-hyprland/windows-and-layout/)。
4. 配置 [ 任务栏 ](/docs/argvus-taskbar/taskbar/)，并决定希望显示哪些控制面板卡片。
5. 查看 [ 键盘快捷键 ](/docs/argvus-hyprland/keyboard-shortcuts/)，仅更改你实际使用的绑定。
6. 通过相关设置页面配置 [ 鼠标和触控板 ](/docs/argvus-hyprland/input/)、显示器、网络和电源。

前四个步骤是可选的个性化设置。其余步骤通常仅在你的硬件、语言或工作流程需要时才需要。

## 两个地方可以更改设置

使用**控制中心**来设置ARGVUS的配置方式：外观、布局、字体、输入、快捷方式、语言、地区和默认应用程序。使用**控制面板**来管理状态和常用会话操作，例如音量、亮度、网络状态、通知、电源和会话控制。

请参阅[控制中心与控制面板](/docs/components/argvus-control-center/)以了解完整区别，以及[配置位置](/docs/user-guide/where-to-configure/)以获取快速索引。

## 更改、持久性和恢复

大多数控制中心的更改会在会话运行时应用，并保存为ARGVUS用户状态。外观更改会提交到规范配置，`argvus-config`项目展示了Hyprland和shell使用的文件。控制面板的切换可能是直接操作或会话状态更改，而不是永久配置。

当页面提供 **重置为默认值** 时，请使用该页面操作，而不是删除生成的文件。外观有针对重音、壁纸和布局值的单独重置操作；键绑定页面有自己的恢复操作。没有一个通用的重置按钮可以重置所有桌面设置。

## 下一步

- [外观](/docs/user-guide/appearance/) — 主题、壁纸、效果和布局。
- [桌面布局](/docs/argvus-hyprland/windows-and-layout/) — 了解间隙、边框和面板空间。
- [输入设备](/docs/argvus-hyprland/input/) — 鼠标和触控板控制。
- [主题和壁纸故障排除](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — 当视觉更改不是预期时的恢复。
