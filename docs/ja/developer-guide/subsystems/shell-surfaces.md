---
title: 貝殻の表面
description: タスクバー、コントロールパネル、テレメトリの所有権。
---
`argvus-waybar` はパッチされた `/usr/bin/waybar` を提供します。`argvus-taskbar` はタスクバーの操作と UX を所有しています。タスクバーの JSONC と CSS は `config.json` の投影であり、`argvus-config` は `data/generated/waybar/` の下にアクティブレイヤーを書き込み、`data/waybar/` の下にミラーコピーを書き込みます。`argvus-control-panel` は Quickshell カードを所有しています。`argvus-widget-telemetry` はオプションのテレメトリーバーとそのブロックのセマンティクスを所有しています。その JSONC と CSS は、外観変更時にパッケージ化されたデフォルトから `data/waybar/` に書き込まれ、その後 `argvus-config` が区切られた `ARGVUS_TELEMETRY_*` とフォントブロックのみを書き換えます。`argvus-session` はそれらのユーザーサービスを所有しています。

このランタイムモデルには、現在 `argvus-shell` パッケージや `argvus-shell.service` は存在しません。
