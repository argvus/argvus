---
title: はじめに
description: ARGVUSが何であるか、そのデスクトップがどのように機能するか、そしてそのコンポーネントがどこに配置されるか。
---
ARGVUSは、Arch Linux向けのモジュラー型HyprlandおよびWaylandデスクトップ環境です。これは一つのモノリシックなアプリケーションではなく、統合されたパッケージのコレクションとして構成されています：セッション、コンポジター、シェルサーフェス、設定アプリケーション、プロバイダー、ビジュアルアセットは、共有状態と明示的なランタイム契約を通じて協調して動作します。

このページはドキュメントのマップです。デスクトップ全体を高レベルで説明しており、リンクされたページには運用および実装の詳細が含まれています。

## ARGVUSの動作方法

通常のセッションは次の経路に従います:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session` はセッションのライフサイクルと環境設定を管理します。すべてのデスクトップ機能を実装するわけではありません：各コンポーネントは自身のコマンド、設定、サービスペイロードの管理を保持します。

インストールと起動については [Installation](/docs/getting-started/installation/) と [Sessions](/docs/user-guide/sessions/) を参照してください。サービスの所有権については、開発者向け [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/) を参照してください。

## 主要レイヤー

### エントリーポイントとインフラ

`argvus` パッケージは `/usr/bin/argvus` ディスパッチャとメインパッケージのエントリーポイントを提供します。`argvus-session` はグラフィカルおよび TTY エントリーポイント、ユーザーターゲット、および `argvus sessionctl` を提供します。共有 TUI およびローカリゼーション機能は `argvus` ワークスペース（`argvus-tui` および `argvus-i18n` クレート）および `argvus i18n` コマンドから提供されます。

### コンポジターとデスクトップシェル

`argvus-hyprland` は Hyprland の設定とキーバインディングを提供します。`argvus-waybar` はパッチ済みの Waybar バイナリを提供し、`argvus-taskbar` はタスクバーの設定と操作を所有します。`argvus-control-panel` は Quickshell パネルを提供し、`argvus widget-telemetry` はオプションのシステムウィジェットを提供します（`argvus` 内でプロセス中）。

使用方法については [Desktop](/docs/user-guide/desktop/) を参照し、実装の所有については [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/) を参照してください。

### 設定とアプリケーション

`argvus control-center` はキーボード優先の設定アプリケーションで、`argvus` にヘルパーとして同梱されています。カレンダーとリムーバブルデバイスツールは `argvus` 内の GTK 4 ヘルパーで、システムモニターとターミナルは `argvus` のサブコマンドです。ランチャーは `argvus-launcher` です。これらはすべてシェルとディスパッチャーと統合されています。

「[Applications](/docs/user-guide/applications/)」と[command reference](/docs/reference/command-line/)を参照してください。

### プロバイダーと統合

ディスプレイ、ネットワーク、Bluetooth、電源、通知、ファイアウォール、リムーバブルストレージは、それぞれ専用のプロジェクトによって提供されています。コントロールセンターとコントロールパネルはこれらのプロバイダーを利用しますが、置き換えるものではありません。

ログイン、ロック、および起動時のビジュアルも別々です：`argvus-greeter`がgreetdを処理し、`argvus-lock`がHyprlockを処理し、`argvus`に搭載されているハンドオフヘルパーがログインスピナーを描画し、読み込みオーバーレイがテーマの切り替えを処理し、`argvus-boot-splash`がブートスプラッシュを処理します。

[Hardware](/docs/user-guide/hardware/)、[Privacy and security](/docs/user-guide/privacy-and-security/)、および開発者向けの[component reference](/docs/developer-guide/)を参照してください。

## 共有表示状態

テーマ、アクセント、壁紙、フォント、およびエフェクトの設定は、`$XDG_CONFIG_HOME/argvus`（通常は`~/.config/argvus`）の下に論理状態として保存されます。`argvus-config`は`data/generated/`に書き込む唯一のコンポーネントであり、Hyprland、GTK、Qt6ct、Waybar、Quickshell、Dunst、通知、ロック画面、Yazi、Superfile、ターミナル用のユーザーファイルを投影します。例外はRofiで、`argvus launcher`は設定やテーマをプレーンファイルとしてパッケージ化し、ユーザーが編集したコピーは`data/rofi/`の下に保存されます。外観ヘルパーは状態の変更をコミットした後に外部アプリケーションを調整しますが、生成されたツリーに書き込むことはありません。

生成されたディレクトリは事実のソースではありません。[Appearance](/docs/user-guide/appearance/)および[configuration and state](/docs/developer-guide/architecture/configuration-and-state/)を参照してください。

視覚システムはレイアウトも制御します。Stickyモードはコンパクトで四角形、Spacedモードはより大きなウィンドウ間隔、丸みを帯びた角、タスクバーやシェル周囲の広いマージンを使用します。タスクバーはデフォルトで上端にありますが、共有レイアウト状態を通じて上下への配置をサポートします。エフェクトはコンポジタとデスクトップ表面全体での透明度、ぼかし、影、アニメーションを制御します。

現在のファミリーと色については [theme table](/docs/user-guide/appearance/) を、共有エフェクト契約については [effects](/docs/argvus-appearance/effects/) を、バーとウィンドウの配置については [taskbar](/docs/argvus-taskbar/taskbar/) を参照してください。

## ARGVUSとは何か、そして何ではないか

ARGVUSはArch Linuxの上に構築されたデスクトップ環境レイヤーです。Arch Linuxはオペレーティングシステムとパッケージを提供し、HyprlandはWaylandのコンポジションを提供します。ARGVUSはグラフィカルセッション、シェルサーフェス、設定、外観、デスクトップサービス、および統合アプリケーションを調整します。

したがって、これは単なるHyprlandの設定や個人的なドットファイルの集まり以上のものです。ARGVUSはサポートされるセッションのエントリーポイントを定義し、デスクトップサービスのライフサイクルを管理し、共通の設定および外観モデルを提供し、ネットワーク、ディスプレイ、電源、通知、アカウント、取り外し可能ストレージなどの特定のコンポーネントを接続します。

ARGVUSはすべてのアプリケーションを置き換える必要はありません。最も信頼性の高い選択肢である場合に選定された上流アプリケーションを統合しつつ、デスクトップ全体の動作と設定の一貫性を維持します。

名前は二つの概念を組み合わせています。*ARGV*は「引数ベクタ（argument vector）」に由来し、プログラムが受け取る引数のリストであり、コマンドやショートカットを通じてデスクトップを操作するキーボード優先の目標を反映しています。*US*は「ユーザーシステム（User System）」に由来し、ユーザー中心のデスクトップという目標を反映しています。詳細な説明と発音の音声については、[About ARGVUS](/about/)をご覧ください。

実装の境界とパッケージ所有権については、開発者向けの[アーキテクチャ概要](/docs/developer-guide/architecture/overview/)および[リポジトリの役割](/docs/developer-guide/architecture/repository-roles/)を参照してください。

## 目標別に進める

- [FAQを読む](/docs/user-guide/faq/)
- [ARGVUSをインストールする](/docs/getting-started/installation/)
- [グラフィカルまたはTTYセッションを開始する](/docs/user-guide/sessions/)
- [テーマと壁紙を設定する](/docs/user-guide/appearance/)
- [タスクバーとコントロールパネルを使用する](/docs/user-guide/desktop/)
- [コマンド、パス、およびサービスを理解する](/docs/reference/)
- [アーキテクチャを学ぶ](/docs/developer-guide/architecture/overview/)
