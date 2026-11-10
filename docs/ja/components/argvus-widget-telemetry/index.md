---
title: ウィジェットテレメトリー
description: Waybarのテレメトリーブロックと、それらの切り替えは、argvusのwidget-telemetryを通じて行われます。
---
テレメトリウィジェットは、タスクバーに表示されるシステム情報です（ヘッダー、キー、システム、CPUおよびGPU、メモリ、ストレージ、プロセス、ネットワーク、および開発者ダッシュボード）。これは`argvus`パッケージ内でのプロセス内にあり、シェルコレクターは存在しません。

## コマンド

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block`はバーのホットパスです：これは毎回リフレッシュ時に呼び出されます。`toggle`はバーが表示するブロックを変更し、スクロールウィンドウを移動させます。

ブロックは`header`、`keys`、`system`、`cpu_gpu`、`memory`、`storage`、`processes`、`network`および`dev_dashboard`です。タスクバーは`cpu-temp`および`gpu`プローブも使用します。GPUプローブは`--present`（サポートされているGPUがDRM/sysfsで確認可能）、`--available`（GPUおよびbtopコマンドがパネルで使用可能）、`--available-temp`（読み取り可能な温度を計測可能）、および`--temp`（有効な最高温度、または`N/A`）を受け入れます。

GPUの温度は、GPUのPCIデバイスに対応しています。NVIDIAの独自計測値は`nvidia-smi`/NVMLバスIDを使用します。NVIDIA nouveau、AMD、およびIntelの計測値は、sysfsデバイスリンクがそのGPUに一意に一致する場合のみhwmonを使用します。タスクバーの温度は、検出されたGPUの中で有効な最大値です。欠落、無効、または曖昧な計測値は利用不可のままです。NVIDIA独自の温度はドライバ提供者が必要です。AMDおよびIntelの温度は、カーネルがリンクされたhwmonセンサーを公開しているかに依存します。Intelの温度はbtop 1.4.7のGPUパネルでは提供されません。

## セッション開始時

`argvus-widget-telemetry.service`ユーザーユニットは、ウィジェットが有効になっている場合のみセッション開始時に起動します。マスタースイッチは`effects.widget_telemetry_enabled`で、設定でデフォルトで有効になっているため、新しいアカウントはログイン時にウィジェットを起動します。

トグルが `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state` を書き込むと、そのファイルが代わりに決定します。`enabled` の最初の行だけがウィジェットを開始します。古い `desktop-telemetry-state` および `sysinfo-state` ファイルは、現在のファイルが存在しない場合でも引き続き読み込まれます。

テレメトリラベルは、サポートされている10言語すべてでコンパクトです。CPU、GPU、IP、DE、WM、Kernel のような技術的識別子は、技術的意味を保持します。稼働時間は `/proc/uptime` から読み取られ、略語の単位で表示されます（例えば `1d 12h 49m`）；システムロケールから全文字に展開されることはありません。ハードウェアモデル名やその他のプロバイダーの値は、ウィジェットの幅に影響を与えることがあります。
