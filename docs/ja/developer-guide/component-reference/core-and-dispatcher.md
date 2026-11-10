---
title: コアとディスパッチャ
description: argvusワークスペース、そのディスパッチャー、および共有のRustクレート。
---
`argvus` は `de/argvus` の Cargo ワークスペースであり、`/usr/bin/argvus` を提供するパッケージです。`argvus-cli` クレートはディスパッチャーです：最初の引数を読み取り、ライブラリコマンドをプロセス内で実行するか、GTK 4 ヘルパーまたはプライベート実行可能ファイルを実行するか、あるいはパッケージ化されたスクリプトを実行します。サブコマンドの完全なリストは [コマンドラインリファレンス](/docs/reference/command-line/)] にあります。

このワークスペースには、コマンド間で共有されるクレートも含まれています：

- `argvus-config`：標準的な設定ストアであり、生成されたツリー (`data/generated/`) の唯一の書き込み先です。また、`argvus config`、`theme-switch`、`effects-toggle` がプロセス内で利用するライブラリでもあります。
- `argvus-i18n` と `argvus-i18n-core`：`resources/locales/` のカタログ、そのバリデーション、および `org.argvus.i18n` QML モジュール。
- `argvus-tui` と `argvus-theme`：共有のターミナル UI とテーマ読み込み。
- `argvus-runtime`：翻訳されたメッセージと、コマンド間で共有されるプロセスヘルパー。

`argvus-session` は別のリポジトリとして維持されます。これはセッションエントリポイント（`session`、`start`、`tty`）およびライフサイクルユニットを提供し、`argvus sessionctl` はそのコントローラーに到達します。
