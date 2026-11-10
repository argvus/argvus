---
title: ネットワーキングとBluetooth
description: ARGVUSを通じてNetworkManagerとBluetoothを管理する。
---
`argvus network` サブコマンドは NetworkManager のレポートと制御を行い、`argvus bluetooth` は Bluetooth アダプターの状態と利用可能性のレポートと制御を行います。両方とも `argvus` 内でプロセス内で実行されます；`networkctl` と `bluetoothctl` はエイリアスとして残ります。コントロールパネルと Waybar はこれらのコマンドを呼び出します。

いずれかのコマンドで `--help` を使用すると、インストールされているサブコマンドを確認できます。ネットワークおよび Bluetooth の UI 統合は、Blueman のような任意の高度なマネージャーとは別です。

コントロールセンターはネットワークの状態を設定ページから分離しています：**ネットワーク → 状態** は現在の状態を要約しますが、インターフェース、Wi-Fi、イーサネット、VPN、DNS、プロキシおよびファイアウォールは、これらのプロバイダーが利用可能な場合に個別のルートとなります。**Bluetooth → 状態**、**デバイス**および**ペアリング**は、アダプターの状態とデバイス操作をカバーします。これらのページはプロバイダーに支えられたシステムアクションを実行します；タスクバーとコントロールパネルは、迅速な状態確認/アクションのためのインターフェースを提供します。

ネットワークおよびBluetoothの変更には認証が必要な場合があり、NetworkManagerやBluetoothサービスを通じて即座に適用されることがあります。これらはARGVUSタスクバーのインジケーターを変更するのとは異なります。

コントロールセンターは、`--ask`を使用して標準入力を通じてnmcliにWi-Fiパスワードを送信します；パスワードはプロセスの引数には含まれず、返されるコマンドエラーからも赤塗りされます。

## コマンドラインでのネットワーク管理

グラフィカルインターフェースを使わずにネットワークを照会および制御するには、`argvus networkctl`を使用してください:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## コマンドラインによるBluetooth管理

Bluetoothアダプターには`argvus bluetoothctl`を使用してください:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
カーネルが`/sys/class/bluetooth`にコントローラーをリストしない場合、`status`はbluetoothctlに確認せずにすぐに`status=no-controller`に応答します（サービスが停止している場合、通常はBlueZを待機します）。タスクバーのモジュールとコントロールパネルのカードは、同じステータスを読み取ります。

インストールされているバージョン固有の追加オプションやサブコマンドを確認するには、どちらのコマンドでも `bluetoothctl` `--help`を使用してください。

Wi-Fiパスワードのダイアログが開いている間、スキャンの更新では選択したSSIDとセキュリティタイプが保持されます。そのネットワークが消えたり、セキュリティが変更された場合は、確認がエラーで停止し、新しい選択が必要になります。

手動DNSは、リゾルバーファイルを更新する前にIPv4およびIPv6のサーバーをそれぞれのNetworkManager設定に送信します。サービス管理のリゾルバーのシンボリックリンクは、ファイルの置き換えを拒否することで保持されます。自動DNSへの正常な復帰はバックアップを復元および削除するため、後で手動でサイクルする際に現在のリゾルバー状態が保存されます。

ネットワークプロファイルエディターを閉じると、現在のジョブが終了するまで待機するため、プロファイルの保存や削除はエディターを破棄しても中断されません。バックグラウンドの更新は引き続きキャンセル可能です。

Wi-Fi接続の要求はネットワークアクションガードを共有します：実行中のアクションやプロファイルエディタージョブが完了してからでないと、別の変更を開始できません。進行状況や完了メッセージは選択された言語に従います。

Bluetoothの削除確認では、デバイスリストが更新されても元のデバイスアドレスが保持されます。bluetoothctlからのバッテリー残量は認識され、デバイス詳細のクエリは制限付きの同時実行と個別のタイムアウトで実行されます。

Wi-Fiのパスワード、手動DNS、およびネットワーク検索フィールドは、括弧付きの貼り付けを受け付けます。貼り付けられたコントロールや改行は省略されます。CtrlのみまたはAltのみのサポートされていない文字ショートカットは文字を挿入しません。エディターは、Ctrl+Altとしてエンコードされた印刷可能なAltGrイベントを受け付けます；Ctrl+SはAltなしでのみ保存します。共有テキストエディターは、視覚的な列で垂直ナビゲーションを維持し、幅の広い文字に対して水平スクロールを整列させます。
