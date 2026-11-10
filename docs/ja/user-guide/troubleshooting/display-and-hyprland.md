---
title: ディスプレイとHyprlandのトラブルシューティング
description: モニターのレイアウトとコンポジタの設定を診断する。
---
保存された表示状態とアクティブなHyprlandレイアウトを区別するには、`argvus displayctl --status`と`argvus displayctl --apply`を使用してください。パッケージ化されたHyprland設定は`/usr/share/argvus/hyprland/`の下にあり、ユーザーによる上書き設定はARGVUS設定ディレクトリの下にあります。
