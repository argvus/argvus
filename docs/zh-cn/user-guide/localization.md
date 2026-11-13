---
title: 本地化
description: 使用 ARGVUS 英语或葡萄牙语。
---

`argvus-i18n` 提供共享目录 `argvus i18n validate` 命令和 QML 模块 `org.argvus.i18n`。 。 。 。 ARGVUS 18艘相交语言:阿拉伯语、孟加拉语、德语、英语、西班牙语、法语、印地语、印度尼西亚语、意大利语、日语、朝鲜语、巴西葡萄牙语、欧洲葡萄牙语、俄语、土耳其语、乌尔都语、越南语和简化中文。

土著 ARGVUS 应用程序和 shell 组件消耗共享目录。 因此,一个被翻译的字符串可能被一个以上的寄存器所使用. 阿拉伯语和乌尔都语在所支持的区域使用从右到左的布局 GTK, (中文(简体) ). Quickshell 和终端接口。

使用 ** 控制中心 → 首选项 → 语言和地区 → 语言 ** 选择 ARGVUS 界面语言。 这个写着 `ARGVUS_LANGUAGE` 改为 `$XDG_CONFIG_HOME/locale.conf` (通常情况下) `~/.config/locale.conf`) (中文(简体) ). 譬如说. `ARGVUS_LANGUAGE=pt-PT` 选择欧洲葡萄牙语。 巴西葡萄牙语`pt-BR`)是一个单独的选项。 语言目录已安装 ARGVUS,所以它不依赖于 `locale-gen`。 。 。 。

控制面板在界面语言变化后自动重启. 启动新会话, 将其应用到会话组件 。 不同的** 区域语言** 设置写道 `LANG` 并控制区域格式和第三方应用程序。 它需要一个生成的系统 locale; 允许在 `/etc/locale.gen` 运行 `locale-gen` 有管理员授权。 键盘布局,变体和控制台键图是独立的设置.

□ 当环境变量覆盖变化时

ARGVUS 应用程序已读取 `ARGVUS_LANGUAGE` 从用户的 `locale.conf`如果缺少这种机制,它们将解决区域问题。 `LANG` 值为已安装目录。 继承的地语变量不会覆盖一个明确的 `ARGVUS_LANGUAGE` 优先选择。 第三方应用程序继续使用系统区域地块.

检查有效值:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

检查 `~/.config/locale.conf` 包含一个有效的 `ARGVUS_LANGUAGE=<catalog>` 数值。 要改变区域格式化或第三方应用语言,请使用**区域地语**并选择生成地语. Control 面板在任一偏好更改后重新启动; 关闭, 如果其他会话组件仍然显示旧界面语言 。 见[环境变量] (/docs/reference/environment-variables技术合同和优先权。
