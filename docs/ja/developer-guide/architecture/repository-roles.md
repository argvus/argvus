---
title: リポジトリの役割
description: ワークスペース統合後のARGVUSソースリポジトリの責任。
---
リポジトリはナビゲーションページごとではなく、責任ごとにグループ化されています：

- workspace: `argvus`（ディスパッチャー、インプロセスコマンド、GTK 4ヘルパー、共有クレート、`crates/argvus-greeter`のグリーターおよびコマンドのデータ）;
- sessionおよびintegration: `argvus-session`、`argvus-hyprland`、`argvus-portal`、`argvus-lock`、`argvus-boot-splash`、およびグリーター（`argvus-greeter`、`argvus/crates/argvus-greeter`からビルド）;
- workspace外のUI: コントロールパネル、タスクバー（`argvus-taskbar`）、ランチャーおよびWaybar;
- プロバイダー: ディスプレイ、電源、ファイアウォール、通知;
- アセットおよびパッケージング: 外観、壁紙、フォント、アイコン、プロファイル。

## workspaceへの統合

これらのリポジトリは`de/argvus`に統合されました。ドキュメントは現在workspaceの`docs/`に存在し、リポジトリ自体は履歴のためのみ保持されています：

| 以前のリポジトリ | 現在 | ドキュメント |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [コマンドライン](/docs/reference/command-line/)、[設定ファイル](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [国際化コンポーネント](/docs/components/argvus-i18n/) |
| `argvus-keys`、`argvus-projects` | `argvus keys`、`argvus projects` | コマンドライン |
| `argvus-terminal` | `argvus terminal` | [端末コンポーネント](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [システムモニターコンポーネント](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`、`argvus bluetooth` | [ネットワークコンポーネント](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | コマンドライン |
| `argvus-control-center` | `argvus control-center`（端末アプリケーション） | [コントロールセンターコンポーネント](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar`（GTK 4 ヘルパー） | [タスクバー カレンダー コンポーネント](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices`（GTK 4 ヘルパー） | [取り外し可能デバイス コンポーネント](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme`（`splash` ヘルパー） | コマンドライン |
| `argvus-tui-terminal` | `argvus tui-terminal`（作業スペース内のクレート） | コマンドライン |
| `argvus-tui`、`argvus-theme` | 共有クレート `crates/argvus-tui` および `crates/argvus-theme` | 開発者ノート |
| `argvus-greeter` | `crates/argvus-greeter` 内のソース、`argvus` パッケージに同梱 | [グリーター コンポーネント](/docs/components/argvus-greeter/) |
| `argvus-themes` | 廃止。公式テーマごとに独自の `argvus-theme-*` パッケージがあります。 | [テーマのインストール](/docs/user-guide/appearance/) |

統合されたコンポーネントの開発者ノートは `docs/internal/<repository>/` にあります。

言語は所有権の境界ではありません：各コンポーネントは自分のパッケージ内にランタイムヘルパーを保持します。パッケージを跨ぐ唯一の厳格なルールは次の通りです：data/generated/ 以下に書き込めるのは `argvus-config` のみです。 `data/generated/` `config.json` を読めないアプリケーションに到達する必要があるコンポーネントは、自身のヘルパーにアダプターを追加し、カノニカルコミット後にオーケストレーターがそれを呼び出すようにします；生成されたファイルに書き込むことは決してありません。公開CLIは`/usr/bin`に属します；内部ヘルパーは`/usr/lib/argvus/<component>`の下、またはランタイム時に必要な場合はcommandsディレクトリ内に置かれます；アーキテクチャに依存しないデータは`/usr/share/argvus/<component>`の下に残ります。

インストールされたパッケージ境界については、[package matrix](/docs/reference/package-matrix/)] を参照してください。
