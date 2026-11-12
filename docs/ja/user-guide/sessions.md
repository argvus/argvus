---
title: セッション
description: ARGVUSのグラフィカルセッションを開始して管理する。
---
`argvus-session` はセッションのエントリーポイントと `systemd --user` のライフサイクルを所有しています。`argvus start` を通じて Hyprland を起動し、セッション環境をインポートし、ARGVUS ターゲットを管理します。

- [グラフィカルセッション](/docs/argvus-session/graphical-session/)
- [TTY セッション](/docs/argvus-session/tty-session/)
- [グリーター](/docs/components/argvus-greeter/)

便利な診断:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
サービスの所有権については、開発者 [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) を参照してください。
