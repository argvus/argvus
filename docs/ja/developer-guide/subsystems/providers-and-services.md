---
title: プロバイダーとサービス
description: プロバイダーパッケージとそれらのシステムインターフェース。
---
プロバイダプロジェクトは、UI利用者に対して特化したインターフェースを公開します:

- `argvus network` と `argvus bluetooth` は NetworkManager と Bluetooth ヘルパーを統合します;
- `argvus-display` は Hyprland とモニター状態を統合します;
- `argvus-power` はアイドル、ロックおよび電源アクションを統合します;
- `argvus-firewall` は `argvus-firewall.service` を管理します;
- argvus 内の `argvus removable-devices` ヘルパーは UDisks2 を使用します;
- `argvus` 内の `argvus control-center` `argvus`  ヘルパーはプロバイダ API とシステム D-Bus インターフェースを使用します。

コントロールセンターはこれらプロバイダのクライアントであり、置き換えるものではありません。
