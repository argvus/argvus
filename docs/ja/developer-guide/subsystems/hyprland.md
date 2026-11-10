---
title: Hyprland の統合
description: ARGVUS コンポジターの構成および統合契約。
---
`argvus-hyprland` は `/usr/share/argvus/hyprland/config/hyprland.lua`、キーバインディングおよびヘルパースクリプトをパッケージ化します。`argvus-session` は Hyprland を起動し、セッション互換性設定を適用します。`argvus-config` は Hyprland のテーマ、レイアウト、ボーダーおよび間隔ファイルを `data/generated/hypr/` に投影し、`argvus-appearance` は `hyprctl` 上でコンポジターを調整します。`argvus-display` はモニターの上書きを提供します。

従来のパッケージパスは、現在インストールされているレイアウトの一部ではありません。
