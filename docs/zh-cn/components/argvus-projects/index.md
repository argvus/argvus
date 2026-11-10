---
title: 项目
description: 通过 argvus 项目，项目选择器和项目配置。
---
`argvus projects` 列出并打开你的项目。它是 `argvus` 包的一个正在进行中的命令，而 `SUPER + O` 打开选择器。

## 命令

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## 什么是项目

项目是配置（`/projects/paths`）中列出的目录，或`/projects/roots`中列出的根目录的子目录。最多列出九个项目：`SUPER + CTRL + N`快捷方式的数量。打开一个项目会聚焦其Hyprland工作区。第一次打开项目时，ARGVUS会启动一个终端，并在其中启动`/projects/editor`中设置的编辑器。

配置通过`argvus config`进行读取和写入，因此控制中心和终端看到的是相同的列表。

## 项目工作区与任务栏

项目使用普通命名的 Hyprland 工作区，而不是特殊工作区。`SUPER + O` 显示项目编号、目录和直接 `SUPER + CTRL + N` 快捷方式。两个入口点都聚焦相同的工作区。只有打开的工作区会出现在任务栏上，带有 `Project: name` 标签、激活状态，以及包含完整路径和快捷方式的工具提示。点击工作区按钮以返回该工作区。任务栏遵循工作区所在的显示器。

规范目录路径决定稳定的工作区身份；更改列表顺序会更改快捷方式，而不是工作区。具有相同目录名称的项目保持独立，对指向同一目录的符号链接会去重。终端和编辑器的启动明确针对项目的工作区。重复使用现有进程/窗口的编辑器可能会保留该应用程序自身的工作区行为。

更新前打开的工作区会保留其现有的窗口和本地名称。更新后打开项目将使用其新的基于路径的身份；在方便的时候关闭旧项目窗口。不会尝试自动关联或移动，因为旧名称不能可靠地识别目录。对于重复的目录名称，请使用选择器、编号或完整的规范路径，而不是模糊的名称。

打开项目会根据配置的项目列表和翻译原子性地重新生成 `$XDG_RUNTIME_DIR/argvus/project-workspaces.json`。这是用于工作区标签和工具提示的会话元数据，而不是配置；请勿编辑。更改列表或语言后重新打开项目以刷新其标签。请同时更新 `argvus` 和 `argvus-waybar`，以及 `argvus-taskbar` 配置，以进行此集成。
