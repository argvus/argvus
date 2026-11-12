---
title: 最初のセッション
description: インストール後にARGVUSを起動してください。
---
グラフィカルログインの場合は、パッケージ化されたgreetd統合を`argvus greeter-setup --help`で構成し、[Sessions](/docs/user-guide/sessions/)で説明されているディスプレイマネージャーフローを使用します。

TTYセッションの場合は、仮想端末でログインし、次を実行します:

```sh
argvus --start-desktop
```
ログイン後、ARGVUSユーザーターゲットを検査するために`argvus sessionctl status`を使用してください。コンポジターまたはシェルが起動しない場合は、[session troubleshooting](/docs/user-guide/troubleshooting/session-startup/)に従ってください。
