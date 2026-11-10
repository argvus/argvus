---
title: 常见问题
description: 关于ARGVUS、其架构和日常使用的常见问题。
---
本页面解答了有关 ARGVUS 是什么、如何分发以及在哪里配置的常见问题。有关详细流程，请按照每个答案中的链接操作。

## 什么是 ARGVUS？

ARGVUS 是一个基于 Hyprland 的模块化 Wayland 桌面环境，适用于 Arch Linux。它不是独立于 Hyprland 的桌面环境；而是在 Hyprland 之上组装的桌面体验，并针对 Arch Linux 进行了定制。ARGVUS 通过一系列软件包协调图形会话、外壳界面、设置应用、外观、桌面服务和集成工具。

它不仅仅是一个 Hyprland 配置或一组个人 dotfiles：它提供了一个受支持的会话入口点、共享的配置和外观状态，以及为任务栏、控制中心、通知、网络和显示等组件定义的所有权。请参阅 [introduction](/docs/introduction/) 以了解各部分如何协同工作。

## ARGVUS 是免费的吗？

是的。ARGVUS 可以免费下载和使用，其核心项目包以 [GNU 通用公共许可证第 3 版](https://www.gnu.org/licenses/gpl-3.0.html) 作为自由软件分发。这赋予用户在许可条款下运行、研究、修改和重新分发所涵盖软件的自由。

ARGVUS 以签名的 Arch Linux 软件包形式分发。软件本身没有购买价格，但下载软件包仍可能涉及正常的互联网或托管成本。第三方提供的组件和应用程序可能有其自己的许可协议。

## ARGVUS 是 Linux 发行版吗？

不是。ARGVUS 是安装在 Arch Linux 之上的桌面环境层。Arch Linux 提供操作系统、内核、包管理器和基础软件包；Hyprland 提供 Wayland 合成；ARGVUS 协调桌面会话及其集成组件。

有关支持的安装模式，请参阅 [installation](/docs/getting-started/installation/)；有关架构区别，请参阅 [What ARGVUS is—and is not](/docs/introduction/#what-argvus-isand-is-not)。

## 为什么选择桌面环境而不是发行版？

将 ARGVUS 保留为桌面环境可以让用户更好地控制系统的其余部分。发行版通常选择并维护完整的基线，包括内核、软件包选择、存储库、默认值和系统策略。 ARGVUS 专注于桌面体验，同时将更广泛的决定留给用户和底层 Arch Linux 安装。

该模型还保持项目模块化：会话管理、Hyprland 配置、外观、任务栏、设置和提供程序可以单独开发和打包，同时作为一个桌面一起工作。

## 支持哪些系统？

当前支持的基础是 Arch Linux。 ARGVUS 是针对 Arch Linux 打包的，依赖于 [ 安装要求 ](/docs/getting-started/installation/requirements/) 中描述的 Wayland、Hyprland 和系统组件。

其他发行版可能提供一些相同的上游组件，但这并不意味着完整的 ARGVUS 软件包集或会话集成在那些发行版上受到支持。

## 我如何安装 ARGVUS？

请按照 [安装指南](/docs/getting-started/installation/) 操作。该指南解释了安装要求、如何配置签名的 ARGVUS 软件包仓库、如何安装软件包集以及如何启动第一个会话。

ARGVUS 以协调的 Arch 软件包集合形式提供。`argvus` 软件包是主要的安装目标，它会拉取其协调的桌面组件。

## 我如何更改 ARGVUS 设置？

使用 **控制中心** 进行持久的桌面配置，包括主题、壁纸、特效、布局、字体、键盘、输入、语言、地区和默认应用。打开它的最有效方法是使用 `SUPER + Alt + C` 快捷方式，或者通过启动器使用 `SUPER + D`：输入 **ARGVUS 控制中心**。你也可以输入 `argvus` 来列出 ARGVUS 应用程序。

作为额外选项，你可以通过终端打开它，命令如下：

```sh
argvus --control-center
```
您还可以使用 `argvus control-center` 打开一个集中区域，例如 `argvus control-center appearance themes`。运行 `argvus control-center --help` 以查看已安装版本中可用的路线。

使用 **控制面板** 查看当前状态和常用操作，例如音量、亮度、网络、通知、电源和会话控制。它不是第二个配置数据库。请参见 [在哪里配置内容](/docs/user-guide/where-to-configure/) 以及 [控制中心](/docs/components/argvus-control-center/) 以了解区别。

## ARGVUS 设置存储在哪里？

用户拥有的配置存储在 `$XDG_CONFIG_HOME/argvus`（通常为 `~/.config/argvus`）下。规范的逻辑配置保存在那里，而为 Hyprland、Waybar 和遥测小部件、GTK、Qt6ct、Quickshell、Dunst、Hyprlock、Yazi、Superfile、终端、字体、效果、日历、输入、键盘、电源和可移动设备生成的派生消费文件则位于 `~/.config/argvus/data/generated/` 下。该树仅由 `argvus-config` 写入，并从规范配置中重建；`argvus-appearance` 提交外观更改，然后只协调无法自行读取规范配置的外部消费者。Rofi 是例外：`argvus launcher` 打包其配置和主题，用户编辑的副本则存在于 `~/.config/argvus/data/rofi/` 下。

不要将生成的文件作为永久配置方法进行编辑。请使用控制中心或你希望自定义的组件的文档化本地覆盖路径。请参阅[配置文件](/docs/reference/configuration-files/)以及[配置与状态](/docs/developer-guide/architecture/configuration-and-state/)。

## 如果更改未生效，我该怎么办？

首先确认更改是否已在所属页面保存或应用。然后检查受影响的组件是否需要会话或服务重新加载。已提交的更改会通过`argvus-config`投射到使用者文件中，但投射操作并不能保证所有正在运行的使用者都已重新加载它们。当规范状态未更改时，`argvus sessionctl reload`会从投射清单应用已提交的生成，而不是再次投射，同时会跳过图形重新加载。

使用相关的[故障排除指南](/docs/user-guide/troubleshooting/)进行安装、会话启动、图形、显示器、Hyprland、主题和壁纸设置。在报告问题时，请包括组件、更改的设置、使用的命令或页面，以及相关状态或诊断命令的输出。

## 我可以在哪里了解更多？

- [首次配置](/docs/user-guide/getting-started/)，用于推荐的首次会话流程。
- [用户指南](/docs/user-guide/)，提供功能特定的说明。
- [参考资料](/docs/reference/)，包括命令、路径、服务和配置契约。
- [开发者指南](/docs/developer-guide/)，介绍架构、软件包拥有权和贡献工作流程。
