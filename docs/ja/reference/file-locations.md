---
title: ファイルの場所
description: 重要なインストール済みのARGVUSファイルとディレクトリ。
---

- `/usr/bin/argvus*` — 公開コマンド。
- `/usr/share/argvus/hyprland/` — Hyprland の設定とヘルパー。
- `/usr/share/argvus/control-panel/` — コントロールパネルの QML とスクリプト。
- `/usr/share/argvus/taskbar/` — タスクバーの設定とスクリプト。
- `/usr/share/argvus/lock/` — Hyprlock の設定。
- `/usr/share/argvus/portal/` — ポータル統合ファイル。
- `/usr/share/backgrounds/argvus/` — 壁紙アセット。
- `/etc/argvus/` — システムプロバイダーの設定。
- `/etc/argvus/greeter.toml` — グリーターの設定。
- `/etc/argvus/removable-devices/config.json` — リムーバブルストレージ用のシステムデフォルト。
- `$XDG_CONFIG_HOME/argvus/config.json` — 標準的なポータブル設定。
- `$XDG_CONFIG_HOME/argvus/data/` — 管理されたコンポーネントデータ、バックアップ、ロック、生成された出力。
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — 任意の完全なネイティブ Waybar タスクバーの上書き。
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — ARGVUSタスクバーのコピーを管理しました。
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — `argvus-taskbar.css`および`argvus-widget-telemetry.{jsonc,css}`を含むWaybarプロファイルを生成しました；これらは上記のコピーより優先されます。
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — また`data/waybar/`の下でもミラーリングされています。こちらは外観変更時にパッケージのデフォルトから置き換えられ、その後`ARGVUS_TELEMETRY_*`およびフォントブロックが書き換えられます。
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — Yazi設定ツリーを投影しました。Yaziフレーバーを提供しないテーマの場合に備えたバリアント対応のフォールバックがあります。
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — Superfileテーマセットを投影しました。
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — ターミナルプロファイルを投影しました。
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — 予測された Qt6ct パレット。
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — 予測された GTK テーマファイル。
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — Rofi の設定、テーマ、モードのユーザーコピー。ARGVUS はファイルを最初に変更するときに各コピーを作成する；変更されていないファイルは `/usr/share/argvus/launcher/config/` から読み込まれる。
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — 予測されたリムーバブルデバイス用スタイルシート。
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`、`$XDG_DATA_HOME/argvus/focus/history.json`、`$XDG_STATE_HOME/argvus/focus/session.json` — ARGVUS の Focus 設定、セッション履歴、現在のセッションチェックポイント。
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — タスクバーで消費される一時的な ARGVUS Focus ステータス；Focus が終了すると削除される。
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`、`$XDG_DATA_HOME/argvus/clock/alarms.json`、`$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUSの時計設定と世界時計、アラーム、タイマー/ストップウォッチのチェックポイント。
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — ARGVUS FocusとARGVUS Clockの色（テーマ消費者`time`）。
- `/usr/share/argvus/sounds/` — ARGVUS FocusとARGVUS Clockの音（`argvus-chime.oga`、`argvus-beacon.oga`、`argvus-bell.oga`）。
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — テーマ構築（`builds/<id>-<variant>-<hash>/`）、`argvus theme render`によって書かれ、すべてのテーマ切り替えやアクセント変更によって行われ、アクティブテーマのビルドへの`current`シンボリックリンク。
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — ユーザーHyprland Luaの上書きとネイティブHyprlandプロジェクション。
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — オプションの完全なネイティブHyprland構成。`argvus start`によって選択されます。これはインクリメンタルオーバーレイではありません。  
- `$XDG_STATE_HOME/argvus/config-projection.json` — プロジェクションマニフェスト：生成、実効構成ハッシュ、変更されたセクション、および保留中のランタイム作業。`argvus sessionctl reload`によって読み取られます；決定的な情報源にはなりません。  
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — レガシーテレメトリーブロックの設定。移行入力専用；正規値は`config.json`にあり、このファイルはもはや書き込まれません。  
- `$XDG_STATE_HOME/argvus/session.log` — グラフィカルセッションログ（通常は`~/.local/state/argvus/session.log`）。  
- `/usr/share/wayland-sessions/argvus.desktop` — グラフィカルセッションエントリ。  
- `/usr/share/plymouth/themes/argvus/` — パッケージ化されたブートスプラッシュアセット。  

正確なペイロードはパッケージ固有です；開発者向けコンポーネントリファレンスおよび関連するパッケージマニフェストを参照してください。
