---
title: システムモニター
description: ARGVUSシステムモニターを開いてください。
---
設定されたモニターを次の方法で開きます:

```sh
argvus --system-monitor
```
`argvus system-monitor` サブコマンドは、ネイティブのエントリーポイントと btop 統合を提供します。タスクバーに表示されるテレメトリは、`argvus widget-telemetry` が提供する別の機能です。その CPU、メモリ、および GPU モジュールをクリックすると、BRANDED ヘッダーと `Esc Close` フッター付きの ARGVUS TUI パネル内で btop が開きます。GPU モードでは、最大6つの GPU ボックスが表示され、CPU、メモリ、ネットワーク、またはプロセスのボックスは表示されません。パネルを閉じるには `q` または `Esc` を押します。下部の Kitty タブには、ローカライズされた `Press q to close` ヒントが表示されます。独立した `argvus system-monitor --cpu`、`--mem`、および `--gpu` 呼び出しは、選択されたボックスで直接 btop を起動します。

GPUパネルは、インストールされているbtopビルドの機能とシステムのデバイスプロバイダーに依存します。NVIDIAの監視には、btopでのGPUサポート、インストール済みNVIDIAドライバーのNVIDIA管理ライブラリ、デバイスへのアクセスが必要です。AMDの監視にはROCm SMIを使用しますが、これは任意であり、インストールされていない場合があります。IntelのGPUデータはsysfsから読み取られ、ユーザーのこれらのインターフェースへのアクセスに依存します。btop 1.4.7はIntelの使用率、消費電力、クロックを報告しますが、Intelの温度は提供しません。NVIDIAのnouveauおよびその他のドライバーは、デバイスにリンクされたhwmonを通じてタスクバーにGPU温度を表示する場合がありますが、btopのプロバイダーサポートはドライバーと利用可能なライブラリに依存します。ARGVUSはデバイス権限を変更したり、btopを管理者権限で実行したりしません。GPUが表示されない場合、またはサポートやプロバイダーが利用できないためにbtopが3秒以内にGPUボックスを表示できない場合、パネルにはローカライズされた「利用不可」のメッセージが表示されます。

モニターは、まず`$XDG_CONFIG_HOME/argvus/config.json`から選択されたテーマとアクセントを読み取ります。生成されたbtopファイルは引き続き派生キャッシュ状態にあります；`.active-theme`と`.accent-color`は互換性のフォールバックです。btopは独自のテーマファイルを保持しているため、モニターは正規の変更がコミットされた後にテーマトランザクション（`argvus theme-switch`）によって外部アダプタとして調整されます；生成されたツリーの一部ではありません。

## ランタイム契約

- テーマおよびアクセントは、標準の設定 (` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme ) から取得されます。`.accent-color` はフォールバックです。
- 生成された btop の設定とテーマは `$XDG_CACHE_HOME/argvus/argvus-system-monitor` に書き込まれます。それらは派生状態であり、編集しないでください。
- デフォルトは `/usr/share/argvus-system-monitor/btop` にインストールされています。
- モジュールショートカット (`--mem`、`--cpu`、`--net`、`--proc`) は最初の引数として読み取り専用です。btop 自身の `-c`/`--config` または `--themes-dir` を渡すと、それらのパスで btop が実行され、設定は書き込まれません。
- `--panel [--cpu|--mem|--gpu]` は btop を ARGVUS TUI シェルに埋め込みます。このモードはタスクバーで使用されます; `--panel` がない場合、モニターは引き続き btop に置き換わります。
