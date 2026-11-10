---
title: ナイトライト
description: Hyprlandで色温度を調整し、固定のローカルスケジュールを設定する。
---
コントロールセンター > デスクトップと入力 > ディスプレイ > ナイトライト を開くか、ナイトライトを検索するか、`argvus control-center displays night-light` を実行します。

ナイトライトとそのスケジュールは最初はオフになっています。1000 〜 20000 ケルビンの温度を選択できます（デフォルトは 4500 K）。値が低いほど暖かい色になります。ページには手動での有効化状況、実際の状態、次の遷移が表示されます。入力すると値が編集でき、Esc キーでキャンセルまたはディスプレイに戻ります。無効な値は保存前に拒否されます。変更は即座に保存され、適用されます。保存後に適用に失敗した場合でも、保存された設定は実際の状態とエラー情報とともに表示されます。

## スケジュールとクイックアクション

固定スケジュールは現在のローカルタイムゾーンを使用します。開始時刻は含まれ、終了時刻は含まれません。**20:00–07:00** は日付をまたぎます。開始と終了は異なる必要があり、`HH:MM` を使用します。位置情報や太陽計算は使用されません。

コントロールパネルの**ディスプレイ**カードを使用するか、**SUPER + Shift + B**でクイックトグルを行います。明るさセレクターは、すべての明るさ操作を保持し、バックエンドが利用可能な場合はナイトライトも追加されます。明るさバックエンドがないディスプレイでも使用可能です。常駐タスクバーのモジュールはありません。

スケジュールがオフの場合は、手動での有効化は持続します。スケジュールがオンの場合は、クイックトグルにより次のスケジュールされた切り替えまでランタイムでのオーバーライドが作成されます。スケジュールや温度を変更すると、そのオーバーライドはクリアされます。ログアウト、新しいコンポジターのインスタンス、または次の切り替えの変更（タイムゾーンの変更を含む）も無効にします。

## コマンドと状態

すべてのインターフェイスは同じ公開コマンドを使用します:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status` は JSON スキーマ バージョン 1 を返します: `available`、`unavailable_reason`、`settings`（enabled、temperature、schedule_enabled、start、end）、`active`、`effective_temperature`、`mode`（`manual` または `scheduled`）、`manual_override`、`next_transition`（オフセット付き ISO 8601）、`minimum_temperature` および `maximum_temperature`。利用可能でない場合、実効フィールドは null です。identity が有効な場合、実効温度は null です。利用可否の失敗は status で報告されます; アクションの失敗は診断とともに非ゼロを返します。

正式な優先設定は `/displays/night_light` であり、`argvus config` が `$XDG_CONFIG_HOME/argvus/config/displays.json` 内で原子操作として保存します。揮発性の上書きとアクションロックは `$XDG_RUNTIME_DIR/argvus/night-light/` の下に存在します。生成されたモニターファイルは独立しています。この機能を設定するために生成されたファイルを編集しないでください。

## ランタイム

Hyprland と **hyprsunset >= 0.4.0** が必要です。セッションはアップストリームの `hyprsunset.service` を一度開始し、`argvus-night-light-init.service` を通じて保存された状態を適用し、`argvus-night-light.timer` および `argvus-night-light.service` を通じて毎分スケジュールを評価します。時計やタイムゾーンの変更でも評価がトリガーされます。スケジューリングがオフの場合、定期的な操作は手動で設定された状態を変更しません。繰り返しの適用では、有効なアイデンティティ/温度を問い合わせ、変更のない書き込みはスキップします。コンポジタのリロードは不要です。

ARGVUSは、上流サービス向けにsystemdのドロップインを提供します：`--identity`から始まり、設定の検出を`/usr/share/argvus/session/config/night-light`に隔離します。これにより、競合するネイティブプロファイルを避け、個人の`hyprsunset.conf`ファイルには影響を与えません。このドロップインは、ARGVUSセッションパッケージがインストールされている間、上流サービスに適用され、ARGVUS外での起動も含まれます。既存のユーザーによるsystemdオーバーライドが優先される場合があります；その状況を診断する際には`systemctl --user cat hyprsunset.service`を確認してください。Hyprsunset 0.4.0では`--config`の別パス引数が拒否されるため、サービスローカルのXDG変数で設定の検出が隔離されます。

バックエンドは公式のHyprsunset IPC（`temperature`、`identity`、`identity get`）を使用しており、入出力は制限され、変更後に読み戻しが行われます。シェーダー、代替コンポジタバックエンド、システム設定の書き込み、または特権昇格は使用されません。利用できないコントロールはコントロールセンターで無効化され、ディスプレイカードでは非表示にされます。
