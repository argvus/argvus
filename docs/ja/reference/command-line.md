---
title: コマンドライン
description: argvusコマンド、そのサブコマンド、キー割り当てのために保持されるフラグ、およびそれぞれが実行される場所。
---
`/usr/bin/argvus` は ARGVUS の公開エントリーポイントです。これは `argvus-cli` クレートからビルドされた 1 つの Rust プログラムです。各サブコマンドは、プログラム内で実行されるか、commands ディレクトリからヘルパー実行ファイルを実行するか、またはパッケージ化されたスクリプトを実行します。

`argvus --help` を実行すると、あなたのバージョンにインストールされた一覧が表示されます。サブコマンドは先頭の `--` を付けずに書かれます。`--` 形式は、キーバインディングやデスクトップエントリで使用される場合のみ保持されます。

## 各コマンドが実行される場所

- **In-process:** このコマンドはワークスペースのライブラリであり、`argvus` 内で実行されます。別個の実行可能ファイルは存在しません。これには `config`、`theme`、`i18n`、`keys`、`gpg`、`projects`、`terminal`、`system-monitor`、`network`、`bluetooth`、`focus status`、`widget-telemetry`、`tui-terminal`、`sensors`、`setup` が含まれます。
- **ヘルパー:** `/usr/lib/argvus/commands/` 内の別個の実行可能ファイル。`argvus` パッケージによってインストールされます。これらは、`control-center` (ターミナル アプリケーション)、`argvus-du` (`argvus du` とともに起動)、および GTK 4 アプリケーション `taskbar-calendar`、`removable-devices`、`focus`、および `clock` です。
- **プライベート実行可能ファイル:** 別のパッケージが `/usr/lib/argvus/commands/` にインストールする実行可能ファイル。 `argvus` のみが起動します。これらには、`session`、`start`、`tty`、`sessionctl`、`launcher`、`snippets`、`ssh`、`git`、 `app-profiles-yazi-keymap`、`notifications`、`displayctl`、`firewall`、`accounts`、`taskbar-actions`、および `greeter-*` エントリ ポイント。
- **スクリプト:** `/bin/sh` で実行されるパッケージ化されたシェル スクリプト。これらは、以下にリストされている Hyprland スクリプト コマンドです。

`ARGVUS_COMMANDS_DIR` は、テストおよび `DESTDIR` チェックのためにコマンド ディレクトリを変更します。そこに見つからないプライベート実行可能ファイルは `PATH` で検索されます。

## サブコマンド

|コマンド |走る |メモ |
| --- | --- | --- |
| `argvus config ...` | 処理中 | 構成モデルとツール。`theme-switch`および`effects-toggle`からもアクセスされます。 |
| `argvus theme ...` | 処理中 | `apply ID`はテーマスイッチです（`theme-switch`と同じ）。テーマのビルド: `list` はインストール済みテーマ、`check DIRECTORY` はインストール済みテンプレートに対するテーマ、`render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`。詳細は [外観とテーマ](/docs/developer-guide/subsystems/appearance-and-themes/)] を参照してください。 |
| `argvus i18n ...` | 処理中 | 翻訳: `get DOMAIN KEY [name=value...]`、`locale`、`list`、`validate [LOCALE]`。 |
| `argvus keys ...` | 処理中 | SSHキー管理。`gpg` はGPGサブコマンドのエイリアスです。 |
| `argvus gpg ...` | 処理中 | GPGキー: `list`、`status`、`generate`、`export-public`、`export-private`、`import`、`import-private`、`delete`、`edit`、`keyserver`。 |
| `argvus projects ...` | 進行中 | プロジェクトピッカー。 |
| `argvus terminal ...` | 進行中 | ARGVUS端末プロファイル。 |
| `argvus system-monitor ...` | 進行中 | btopシステムモニター；`--gpu`はGPUのみのボックスを選択し、`--panel [--cpu|--mem|--gpu]`はそれをARGVUS TUIシェルに埋め込みます。 |
| `argvus du` | ヘルパー（端末アプリケーション） | 専用のKittyウィンドウでARGVUSディスク使用量アナライザーを開き、ncduを実行します。 |
| `argvus network ...` | 進行中 | NetworkManagerのステータスとコントロール（`networkctl`はエイリアスとして保持）。 |
| `argvus bluetooth ...` | 進行中 | Bluetoothのステータスとコントロール（`bluetoothctl`はエイリアスとして保持）。 |
| `argvus widget-telemetry block ...` | 進行中 | Waybarのテレメトリーブロック。これはバーのホットパスです。 |
| `argvus widget-telemetry toggle ...` | 進行中 | テレメトリーブロックのオン・オフ切り替え、およびその順序。 |
| `argvus game-snake` | プライベート実行可能ファイル | ターミナル用のスネークゲームで、独立した`argvus-game-snake`プロジェクトのオプションパッケージです。ターミナル外で実行するとターミナルを開きます。 |
| `argvus control-center [page]` | ヘルパー（ターミナルアプリケーション） | コントロールセンターを開くか、すでに開いているウィンドウにフォーカスします。 |
| `argvus welcome [--if-needed]` | ディスパッチャ → コントロールセンター | コントロールセンター内の統合ウェルカムページを開きます。`--if-needed`は、ウェルカムがすでに表示、完了、または閉じられたユーザーの進行状況がない場合に一度だけ開きます。 |
| `argvus taskbar-calendar [args]` | GTK 4アプリケーション | デフォルトは`toggle`です。 |
| `argvus removable-devices [args]` | GTK 4アプリケーション | デフォルトは`menu`です。 |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4アプリケーション | ARGVUSフォーカスポップアップ。セッションごとに1つのインスタンス。 |
| `argvus focus status --waybar [--follow]` | 処理中 | FocusタスクバーインジケーターをWaybar JSONとして出力します; `--follow`はGTKを読み込まずに変更をストリームします。 |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4アプリケーション | ARGVUS時計ポップアップ。各セッションにつき1つのインスタンス; `--background`はポップアップなしでアラームを有効にします。 |
| `argvus loading-theme ...` | `splash`を`/usr/lib/argvus/loading-theme/splash`でパッケージ化 | 直接のセッションリロードのオーバーレイ; テーマ切り替えは使用しません。 |
| `argvus session`, `start`, `tty`, `sessionctl` | プライベート実行ファイル | セッションのエントリーポイントおよびセッションコントローラー。 |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | プライベート実行ファイル | グリーターのエントリーポイント。 |
| `argvus accounts`, `firewall` | プライベート実行ファイル | アカウントおよびファイアウォールツール。 |
| `argvus launcher`、`snippets`、`ssh`、`git` | プライベート実行ファイル | ランチャーおよびピッカーツール。 |
| `argvus tui-terminal` | プロセス内 | TUIアプリケーション用ターミナルラッパー。 |
| `argvus sensors` | プロセス内 | `sensors -j`からのライブ温度概要。 |
| `app-profiles-yazi-keymap` | プライベート実行ファイル | キーマップヘルパー。 |
| `argvus notifications`、`displayctl`、`taskbar-actions` | プライベート実行ファイル | 通知、ディスプレイおよびタスクバーの操作。 |
| `argvus setup [options]` | プロセス内 | パッケージされたデフォルトをユーザー設定にコピーします。詳細は下記参照。 |

引数は、スペースや空の値を含めて変更されずに転送されます。標準入力、出力、エラー、シグナル、および終了ステータスは、実行されるコマンドに属します。未知の名前の場合、`argvus --help` を実行するヒントと共に失敗します。プロセス内で実行されるコマンドは、無効な UTF-8 である引数を受け入れず（終了ステータス 2）、変更された値を渡すことはありません。別のプログラムを実行するコマンドは、バイトを変更せずに受け取ります。

## フラグとエイリアス

これらの名前は、キーバインディング、デスクトップエントリ、および古いスクリプトが引き続き動作するように存在します。新しいコードでは、サブコマンド形式を優先してください。

| フラグまたはエイリアス | 同等 |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (`about` および `apps` ページ付き) |
| `--displays`, `--monitors` ページ | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`、`--removable-devices`、`--storage` | `taskbar-calendar`、`removable-devices` |
| `--focus`、`--clock` | `focus`、`clock` |
| `--spf`、`--superfile` |ファイルマネージャー (`spf`) |
| `--yazi`、`--yazy` |ファイル ブラウザ (`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | `tty` を開始します (引数は転送されません)。
| `--version`、`-V`、`version` |プリント `ARGVUS <version>` |
| `help`、`--help`、`-h` |使用法を出力します |
| `networkctl`、`bluetoothctl` | `network`、`bluetooth` |
| `widget-telemetry-toggle`、`widget-telemetry-block` | `widget-telemetry toggle`、`widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

コントロール センターは、従来のページ フラグを最初の引数として受け入れ、それらをページ名にマップします。例: `--apps` は `apps` に、`--keyboard-layout` は `locale` に、`--hostname` は `system` になります。

## Hyprland スクリプト コマンド

これらはパッケージ化されたスクリプトを実行し、Hyprland の設定で使用されます:

`cheatsheets-hypr`、`cheatsheets-launcher`、`scratchpad`、`hypr-screenshot`、`spaces-switch`、`emoji-picker`、`toggle-sidebar`、`weather-location`、`layout-mode-menu`、`brightness-switch`、`toggle-mode`、`idle-timeout`、`keep-awake`、`hypr-power-menu`。

ユーザーの設定にあるスクリプトは、パッケージ化されたものを上書きします。検索の順序は、hyprland.lua 内の `_config_path` と同じです: ユーザーの設定コピー、次に ARGVUS データコピー、次にステートコピー、次に生成されたコピー、そして最後に  `hyprland.lua` `/usr/share/argvus` の下にあるパッケージ化ファイルです。

## 例

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## セットアップ

`argvus setup`は、`/usr/share/argvus/<app>`から`~/.config/argvus/data/<app>`にパッケージ化されたデフォルトをコピーするので、カスタマイズが可能です。パッケージ化されたファイルを編集することは決してありません。

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
すでに存在する宛先は報告され、そのままにされます。ただし、`--force` が指定されている場合は例外です。`--force` が指定されている場合は、コピーの前に既存のディレクトリが `<name>.bak-<timestamp>` に名前変更されます。

`argvus` アプリは特別です：パッケージ化されたディレクトリの内容を設定ルートにコピーします。セットアップの実行時には設定ルートがすでに存在するため、現在のコードでは `--copy argvus` に `--force` が必要です。

セットアップは、何かを変更する前にすべてのオプションを検証します。無効なアプリ名（空、`.` で始まる、または `/` を含む）は拒否されます。

## セッションとリロード

`SUPER + SHIFT + R` は `argvus sessionctl reload` を実行します。セッションコントローラーは生成された設定を適用し、自分が管理するコンポーネントのみを再起動します。ホームディレクトリに `/usr/share/argvus` をコピーすることはなく、破損した JSON ファイルを置き換えることもありません。

## ナイトライト

「デスクトップと入力 > ディスプレイ > ナイトライト」を開くか、`argvus control-center displays night-light`を実行してください。温度、手動起動、および固定ローカルスケジュールは`argvus config display night-light`で設定できます。[Night Light](/docs/user-guide/night-light/)]でコマンド、状態、および実行時要件を参照してください。

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]`は、共有されたARGVUSターミナルフレーム内でインタラクティブなコマンドを実行します。Ctrl+Eでフレームを閉じます。呼び出し元がローカライズされたフッターを提供します。これはターミナル内で使用するか、`argvus tui-terminal --term kitty -- ...`に渡されるコマンドとして使用します。

`argvus tui-terminal --term kitty --profile ai-agents`は、隠しタブとクリップボードショートカットのみを持つ専用のKitty構成を使用します。呼び出し元がコマンドとタイトルを提供し、ウィンドウの配置はクラス`argvus-ai`のHyprlandルールによって管理されます。
