---
title: 本地化
description: 使用 ARGVUS 英语或葡萄牙语。
---
`argvus-i18n` 提供共享目录、`argvus i18n validate` 命令和 QML 模块 `org.argvus.i18n`。ARGVUS 目前提供英语和葡萄牙语目录。

本地应用程序和 shell 组件使用共享目录。因此，翻译后的字符串可能被多个存储库使用。

使用 **控制中心 → 语言和地区** 配置语言、区域设置、生成的系统区域设置、时区、日期/时间和键盘。当前 ARGVUS 目录提供英语和葡萄牙语；这与操作系统生成的系统区域设置列表是分开的。

更改显示语言会保存 ARGVUS 的偏好设置，并自动重启 `argvus-control-panel.service`，因此控制面板会立即反映更改。其他应用程序和 shell 界面可能需要重新加载会话或启动新会话。生成或更改 `/etc/locale.gen` 条目需要授权，并会运行 `locale-gen`；选择未生成的区域设置会被拒绝。键盘布局、变体和控制台键盘映射是单独的设置，不应与界面语言混淆。

## 当环境变量覆盖更改时

ARGVUS 应用程序会首先从持久的 ARGVUS 语言文件中检测语言，然后依次检查 `LC_ALL`、`LC_MESSAGES` 和 `LANG`。因此，本地环境变量可能会导致控制中心似乎忽略语言更改，尤其是在用户服务或图形会话保留了旧环境时。

使用以下命令检查有效值:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```
如果 `~/.config/locale.conf` 存在并定义了 `LANG`、`LC_ALL` 或 `LC_MESSAGES`，请临时重命名它，以便不再重新引入这些值。请保留备份，而不是删除它：

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```
再次在 **控制中心 → 地区和语言** 中应用语言。控制面板会自动重启；如果其他组件仍显示之前的语言，请注销并重新登录，以便图形会话和 `systemd --user` 接收新的环境。之后重复诊断命令。请参阅 [环境变量](/docs/reference/environment-variables/)] 了解技术约定和优先级。
