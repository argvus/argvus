---
title: UIコンポーネント
description: ARGVUSのユーザーインターフェースパッケージ。
---
UIパッケージには`argvus-control-panel`、`argvus-taskbar`、`argvus-launcher`が含まれており、`argvus`には`argvus control-center`（`argvus`内のヘルパー）、`argvus taskbar-calendar`、`argvus widget-telemetry`、`argvus terminal`、`argvus system-monitor`、`argvus removable-devices`、`argvus focus`、`argvus clock`のサブコマンドがあります。カレンダー、Focus、Clockは、レイヤーシェルの位置決めと解除のために共有`argvus-taskbar-popup`クレートを使用します。Focusはまた、プロセス内の`argvus focus status`ルートを通じてWaybarのステータスリーダーも公開します。

それらのユーザー向けの動作は機能ごとに文書化されています。このページではパッケージの所有権を記録しています。
