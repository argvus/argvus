---
title: パッケージマトリックス
description: ARGVUSパッケージの責任。
---

| パッケージ | 役割 |
| --- | --- |
| `argvus` | マスターパッケージ：ディスパッチャー、インプロセスコマンド（設定、翻訳、キー、プロジェクト、ターミナル、システムモニター、ネットワーク、Bluetooth、テレメトリ、TUIターミナル、セットアップ）、共有データおよびi18nカタログ。すべてのコンポーネントパッケージがこれに依存しています。 |
| `argvus`（ヘルパー） | 同じパッケージ内のヘルパー：コントロールセンターとディスク使用量アナライザー（ターミナルアプリケーション）、さらにタスクバーカレンダー、リムーバブルデバイス、ローディングオーバーレイ、ARGVUSフォーカスおよびARGVUSクロックのGTK 4アプリケーション。 |
| `argvus`（グリーター） | greetd用ログイングリーター。同じパッケージで提供され、`crates/argvus-greeter`からビルドされます。 |
| `argvus-desktop` | `argvus`マスターパッケージに必要なデスクトップおよび日常アプリケーションプロファイル。`argvus`に依存していないため、依存関係は一方向のままです。 |
| `argvus-devtools`, `argvus-pentest` | 開発ツールおよび認可されたセキュリティ監査用のオプション独立メタパッケージ。マスターパッケージはこれらのプロファイルをインストールしません。 |
| `argvus-session` | セッションのエントリーポイントとユーザーライフサイクル。 |
| `argvus-hyprland` | Hyprland設定。 |
| `argvus-control-panel` | Quickshellコントロールパネル。 |
| `argvus-taskbar`, `argvus-waybar` | Waybar設定およびパッチ済みバイナリ。 |
| `argvus-branding` | ロゴおよびワードマーク資産。 |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | 外観状態および視覚資産。 |
| `argvus-display`, `argvus-power`, `argvus-notifications` | デスクトッププロバイダー。 |
| `argvus-lock`, `argvus-boot-splash` | ロックスクリーンおよびブートスプラッシュ。 |
| `argvus-firewall` | 自身の権限を持つファイアウォール。 |
| `argvus-launcher` | ランチャー設定およびエントリーポイント。 |
| `argvus-app-profiles` | バンドルされた端末アプリケーション用のデータのみのプロファイル。 |
| `argvus-portal` | XDGポータルおよびWayland/DBusのデフォルト設定。 |
| `argvus-theme-*` | 公式テーマファミリーごとのオプションパッケージ。 |
| `argvus-game-snake`, `argvus-games` | オプション。Snakeは独立したプロジェクトです; `argvus-games`はオプションのゲームをまとめています。 |

## 置き換えられたパッケージ

`argvus`は`replaces`、`conflicts`、`provides`を吸収したパッケージとして宣言しているため、アップグレードするとそれらは削除され、ファイルは孤立した状態として残りません:

`argvus-config`、`argvus-i18n`（`argvus-language-en-us`および`argvus-language-pt-br`を含む）、`argvus-keys`、`argvus-projects`、`argvus-terminal`、`argvus-system-monitor`、`argvus-network`、`argvus-widget-telemetry`および`argvus-du`。`argvus`はまた、`argvus-control-center`、`argvus-taskbar-calendar`、`argvus-loading-theme`、`argvus-removable-devices`、および以前の分割パッケージ`argvus-gtk`と`argvus-greeter`を置き換えますが、依存関係のあるパッケージが解決できるよう、これらも引き続き提供されます。アカウントマネージャー（`argvus accounts`）は`crates/argvus-accounts`から構築され、`/usr/lib/argvus/commands/accounts`としてインストールされ、そのpolkitルールが適用されます。ディスク使用量アナライザーは`crates/argvus-du`から構築され、`/usr/lib/argvus/commands/argvus-du`としてインストールされます。

`/usr/bin/argvus-terminal` は、`argvus terminal` の互換性ランチャーとしてインストールされたままです。Hyprland のデフォルト端末、コントロールセンターのカタログ、および保存されたデフォルトアプリケーションは、依然としてそのコマンド名を使用しています。

削除されたパッケージ `argvus-pwvucontrol` および `argvus-snappy-switcher` は、どのパッケージによっても提供されていません。

ARGVUS コンポーネントパッケージ間でのインストール順序は必要ありません：`argvus` がそれらのマスターパッケージであり、各コンポーネントはそれに依存します。`argvus` をインストールすると、`argvus-desktop` プロファイルもインストールされます；`argvus-devtools` および `argvus-pentest` はオプトインのままです。この表にパッケージが存在しているからといって、そのパッケージが機能全体を所有しているわけではありません：例えば、タスクバーは `argvus-waybar`、`argvus-taskbar`、`argvus-session`、およびドメインプロバイダーを組み合わせています。
