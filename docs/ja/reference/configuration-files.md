---
title: 設定ファイル
description: ユーザー、システム、パッケージデフォルトおよび生成されたARGVUS構成。
---

| レイヤー | 場所 | 意味 |
| --- | --- | --- |
| システム | `/etc/argvus/` | サービスおよびプロバイダーの管理者設定。 |
| パッケージのデフォルト | `/usr/share/argvus/` | 読み取り専用のコンポーネントデフォルトおよびランタイムアセット。 |
| ユーザー | `$XDG_CONFIG_HOME/argvus/config.json` | 標準的なポータブル設定。 |
| 管理データ | `$XDG_CONFIG_HOME/argvus/data/` | コンポーネントデータ、互換性ファイルおよび内部メタデータ。 |
| 生成済み | `$XDG_CONFIG_HOME/argvus/data/generated/` | `config.json` の投影で、`argvus-config` のみが書き込みます。編集せず、他の場所に設定としてコピーせず、ソースとして読み戻すこともありません。 |
| ランタイム | `$XDG_RUNTIME_DIR/` および `$XDG_CACHE_HOME/` | ソケット、ロック、そして一時的な状態。 |

アクティブなテーマ、アクセント、カスタム壁紙、およびエフェクトの状態は、論理的なユーザー状態です。管理された論理モデルは[Canonical configuration](/docs/reference/argvus-config/)]に文書化されています。生成されたHyprland、GTK、Qt6ct、Waybar、Quickshell、Dunst、Hyprlock、Yazi、Superfile、ターミナル、フォント、リムーバブルデバイスのファイルは、独立した信頼できる情報源として扱うべきではありません。

`argvus-config`はまた、テーマレイヤーを`data/waybar/argvus-taskbar.{jsonc,css}`および`data/waybar/argvus-widget-telemetry.{jsonc,css}`にミラーリングします。外観が変更された際、これらのコピーはパッケージ化されたデフォルトから置き換えられ、その後区切られた管理ブロックが書き直されるため、手動で維持されるファイルではなく派生物として扱ってください。`$XDG_CONFIG_HOME/<app>`の下にあるネイティブアプリケーション構成は異なります：これは外観オーケストレーターによって書かれるアダプターターゲットであり、プロジェクターによって書かれることはありません。

## 単一作成者ルールの例外

`argvus config` は `data/generated/` の唯一の作成者です。移行に関しての例外が一つあります：  
パッケージ化されたファイルのユーザーコピーが作成される場合、古いバージョンが `data/generated/` の下に残したファイルがユーザーの場所（`data/`）に移動され、ユーザーは編集内容を保持します。この移動は、現在のバージョンが生成しないファイルに対してのみ発生します。新しいファイルに関してはこのパスに依存しないでください：ここではなく、プロジェクションに生成されたファイルを追加してください。
