---
title: 环境变量
description: ARGVUS 会话组件使用的环境契约。
---
会话建立了 XDG 和 Wayland 环境的集成，并且在检测到时可能导出虚拟化兼容性变量。`ARGVUS_CONFIG_HOME`、`ARGVUS_SYSTEM_CONFIG` 和 `ARGVUS_BOOTSTRAP` 被会话和调度程序助手用于解析配置根。

将环境变量视为运行时约定：在添加持久覆盖之前，请验证已安装助手的 `--help` 或源。

派生状态位于状态根下，而不是配置根下。`XDG_STATE_HOME`（通常为 `~/.local/state`）接收 `argvus/config-projection.json`，投影清单 `argvus-config` 重建并由 `argvus sessionctl reload` 应用，此外还有 `argvus/session.log`。两者都不是可信源。

## 语言环境

唯一的一般语言偏好是 `LANG`，位于 `$XDG_CONFIG_HOME/locale.conf`（默认 `~/.config/locale.conf`）。如果没有有效值，ARGVUS 会选择英语（`en-US`），即便 `/etc/locale.conf`、PAM 或登录管理器提供葡萄牙语。像 `pt_BR.UTF-8` 这样的值会规范化为 `pt-BR` 用于目录。

会话在启动桌面组件和导入 systemd 用户及 D-Bus 激活环境之前应用此偏好。它会替换继承的 `LANG`、`LC_MESSAGES`、`LANGUAGE`，并清除 `LC_ALL`；同一文件中的区域性 `LC_*` 设置会被保留。生成时，英语使用 `en_US.UTF-8`，否则使用 `C.UTF-8`。不自动创建偏好文件。更改语言后请注销并重新登录。

`session.language`、`argvus/data/language` 和 `argvus/data/internal/language` 是被忽略的遗留数据。它们既不会被导入，也不会被重新生成；现有文件将被保留。`argvus config set` 和 `patch` 拒绝向 `session.language` 写入新数据，并将调用者引导至 `locale.conf`。

用于诊断时，可比较 `argvus i18n locale`、`cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` 和 `systemctl --user show-environment`。`localectl status` 描述系统，而非用户的 ARGVUS 语言。`ARGVUS_I18N_DIR` 为开发选择目录，而非语言。
