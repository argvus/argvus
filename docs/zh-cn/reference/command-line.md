---
title: 命令行
description: argvus 命令、它的子命令、为键绑定保留的标志，以及每个命令的运行位置。
---
`/usr/bin/argvus` 是 ARGVUS 的公共入口点。它是一个 Rust 程序，由 `argvus-cli` crate 构建。每个子命令要么在该程序内运行，要么从 commands 目录运行辅助可执行文件，或者运行打包的脚本。

运行 `argvus --help` 查看与你的版本一起安装的列表。子命令在书写时不带前导 `--`。`--` 形式只在使用快捷键或桌面条目时保留。

## 每个命令的运行位置

- **进程内运行：** 该命令是工作区的库，并在 `argvus` 内运行。没有单独的可执行文件。这些命令包括 `config`、`theme`、`i18n`、`keys`、`gpg`、`projects`、`terminal`、`system-monitor`、`network`、`bluetooth`、`focus status`、`widget-telemetry`、`tui-terminal`、`sensors` 和 `setup`。
- **帮助程序：** `/usr/lib/argvus/commands/` 中的单独可执行文件，由 `argvus` 软件包安装。它们是 `control-center`（终端应用程序）、`argvus-du`（与 `argvus du` 一起启动）和 GTK 4 应用程序 `taskbar-calendar`、`removable-devices`、`focus` 和 `clock`。
- **私有可执行文件：**另一个包安装在 `/usr/lib/argvus/commands/` 中的可执行文件。 `argvus` 只启动它。其中包括 `session`、`start`、`tty`、`sessionctl`、`launcher`、`snippets`、`ssh`、`git`、 `app-profiles-yazi-keymap`、`notifications`、`displayctl`、`firewall`、`accounts`、`taskbar-actions` 和 `greeter-*` 入口点。
- **脚本：** 使用 `/bin/sh` 运行的打包 shell 脚本。这些是下面列出的 Hyprland 脚本命令。

`ARGVUS_COMMANDS_DIR` 更改命令目录，用于测试和 `DESTDIR` 检查。在 `PATH` 上查找未找到的私有可执行文件。

## 子命令

|命令|运行|笔记|
| --- | --- | --- |
| `argvus config ...` | 处理中 | 配置模型和工具。也可通过 `theme-switch` 和 `effects-toggle` 访问。 |
| `argvus theme ...` | 处理中 | `apply ID` 是主题切换（与 `theme-switch` 相同）。主题构建：`list` 已安装的主题，`check DIRECTORY` 针对已安装模板的主题，`render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`。请参见 [外观和主题](/docs/developer-guide/subsystems/appearance-and-themes/)。 |
| `argvus i18n ...` | 处理中 | 翻译：`get DOMAIN KEY [name=value...]`、`locale`、`list`、`validate [LOCALE]`。 |
| `argvus keys ...` | 处理中 | SSH 密钥管理器。`gpg` 是 GPG 子命令的别名。 |
| `argvus gpg ...` | 处理中 | GPG 密钥：`list`、`status`、`generate`、`export-public`、`export-private`、`import`、`import-private`、`delete`、`edit`、`keyserver`。 |
| `argvus projects ...` | 进行中 | 项目选择器。|
| `argvus terminal ...` | 进行中 | ARGVUS 终端配置文件。|
| `argvus system-monitor ...` | 进行中 | btop 系统监视器；`--gpu` 选择仅 GPU 的框，`--panel [--cpu|--mem|--gpu]` 将其嵌入 ARGVUS TUI shell。|
| `argvus du` | 辅助（终端应用程序） | 在独立的 Kitty 窗口中打开 ARGVUS 磁盘使用分析器并运行 ncdu。|
| `argvus network ...` | 进行中 | NetworkManager 状态和控制（`networkctl` 保留为别名）。|
| `argvus bluetooth ...` | 进行中 | 蓝牙状态和控制（`bluetoothctl` 保留为别名）。|
| `argvus widget-telemetry block ...` | 进行中 | Waybar 遥测模块。这是状态栏的关键路径。|
| `argvus widget-telemetry toggle ...` | 进行中 | 切换遥测模块的开关及其顺序。|
| `argvus game-snake` | 私有可执行文件 | 终端贪吃蛇游戏，来自独立的 `argvus-game-snake` 项目，可选安装包。在终端外运行时会打开一个终端。 |
| `argvus control-center [page]` | 辅助程序（终端应用） | 打开控制中心，或聚焦已打开的窗口。 |
| `argvus welcome [--if-needed]` | 调度器 → 控制中心 | 打开控制中心的集成欢迎页面。`--if-needed` 只会打开一次，除非每个用户的记录显示欢迎页面已显示、已完成或已关闭。 |
| `argvus taskbar-calendar [args]` | GTK 4 应用 | 默认使用 `toggle`。 |
| `argvus removable-devices [args]` | GTK 4 应用 | 默认使用 `menu`。 |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 应用 | ARGVUS 聚焦弹出窗口。每个会话仅允许一个实例。 |
| `argvus focus status --waybar [--follow]` | 处理中 | 以 Waybar JSON 的形式发出焦点任务栏指示器；`--follow` 在不加载 GTK 的情况下流式传输更改。 |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 应用程序 | ARGVUS 时钟弹出窗口。每个会话一个实例；`--background` 在没有弹出窗口的情况下保持闹钟活动。 |
| `argvus loading-theme ...` | 打包的 `splash` 在 `/usr/lib/argvus/loading-theme/splash` | 直接会话重载的覆盖；主题切换不使用它。 |
| `argvus session`, `start`, `tty`, `sessionctl` | 私有可执行文件 | 会话入口点和会话控制器。 |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | 私有可执行文件 | 欢迎界面入口点。 |
| `argvus accounts`, `firewall` | 私有可执行文件 | 帐户和防火墙工具。 |
| `argvus launcher`, `snippets`, `ssh`, `git` | 私有可执行文件 | 启动器和选择器工具。|
| `argvus tui-terminal` | 进程内 | TUI 应用程序的终端包装器。|
| `argvus sensors` | 进程内 | 来自 `sensors -j` 的实时温度概览。|
| `app-profiles-yazi-keymap` | 私有可执行文件 | 键位映射助手。|
| `argvus notifications`, `displayctl`, `taskbar-actions` | 私有可执行文件 | 通知、显示和任务栏操作。|
| `argvus setup [options]` | 进程内 | 将打包的默认值复制到用户配置中。见下文。|

参数将原样转发，包括空格和空值。标准输入、输出、错误、信号和退出状态属于运行的命令。未知名称会提示运行 `argvus --help`。进程内运行的命令会拒绝非有效 UTF-8 的参数（退出状态为 2），而不会传递修改过的值；运行单独程序的命令会接收未经更改的字节。

## 标志和别名

这些名称存在是为了让按键绑定、桌面条目和旧脚本继续工作。在新代码中，建议优先使用子命令形式。

| 标志或别名 | 等效 |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center`（包括 `about` 和 `apps` 页面） |
| `--displays`, `--monitors` 页面 | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`、`--removable-devices`、`--storage` | `taskbar-calendar`，`removable-devices` |
| `--focus`，`--clock` | `focus`，`clock` |
| `--spf`，`--superfile` |文件管理器(`spf`) |
| `--yazi`，`--yazy` |文件浏览器(`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` |开始 `tty`（不转发任何参数）|
| `--version`，`-V`，`version` |打印 `ARGVUS <version>` |
| `help`、`--help`、`-h` |打印用法 |
| `networkctl`，`bluetoothctl` | `network`，`bluetooth` |
| `widget-telemetry-toggle`，`widget-telemetry-block` | `widget-telemetry toggle`，`widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

控制中心接受旧版页面标志作为其第一个参数，并将它们映射到页面名称。示例：`--apps` 变为 `apps`、`--keyboard-layout` 变为 `locale`、`--hostname` 变为 `system`。

## Hyprland 脚本命令

这些运行打包脚本，并被 Hyprland 配置使用：

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`。

用户配置中的脚本会覆盖打包脚本。查找顺序与 hyprland.lua 中的 `_config_path` 相同：先是用户的配置副本，然后是 ARGVUS 数据副本，再是状态副本，然后是生成的副本，最后才是  `hyprland.lua` `/usr/share/argvus` 下的打包文件。

## 示例

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## 设置

`argvus setup` 将 `/usr/share/argvus/<app>` 中的打包默认值复制到 `~/.config/argvus/data/<app>` 中，以便您可以自定义它们。它从不编辑打包的文件。

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
已存在的目标会被报告并保持不动，除非提供 `--force`。使用 `--force` 时，现有目录会在复制前重命名为 `<name>.bak-<timestamp>`。

`argvus` 应用程序是特殊的：它会将其打包目录的内容复制到配置根目录。因为在设置运行时配置根目录已存在，`--copy argvus` 在当前代码中需要 `--force`。

设置在更改任何内容之前会验证每个选项。无效的应用程序名称（为空、以 `.` 开头或包含 `/`）将被拒绝。

## 会话和重新加载

`SUPER + SHIFT + R` 运行 `argvus sessionctl reload`。会话控制器应用生成的配置并仅重启其拥有的组件。它不会将 `/usr/share/argvus` 复制到主目录，也不会替换格式错误的 JSON 文件。

## 夜灯

打开 **桌面与输入 > 显示 > 夜间模式** 或运行 `argvus control-center displays night-light`。通过 `argvus config display night-light` 配置色温、手动激活和固定本地计划。有关命令、状态和运行时要求，请参见 [夜间模式](/docs/user-guide/night-light/)。

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]` 在共享的 ARGVUS 终端框架中运行交互式命令。Ctrl+E 关闭该框架；调用者提供本地化页脚。在终端中使用此命令，或将其作为传递给 `argvus tui-terminal --term kitty -- ...` 的命令。

`argvus tui-terminal --term kitty --profile ai-agents` 使用专用的 Kitty 配置，带隐藏标签页，仅有剪贴板快捷键。调用者提供命令和标题；窗口位置由 `argvus-ai` 类的 Hyprland 规则处理。
