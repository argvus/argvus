---
title: セッションとsystemd
description: ARGVUSセッションターゲットとそのユーザーサービス。
---
ユニット定義は `argvus-session` によって `/usr/share/argvus/session/config/systemd/user/` のもとでパッケージ化されています。セッションマネージャーはライフサイクルと環境のインポートを管理します。コンポーネントリポジトリは、そのユニットによって実行されるコマンドを管理します。

統合セッションを診断する際には、コンポーネントを手動で起動する代わりに `argvus sessionctl` を使用してください。
