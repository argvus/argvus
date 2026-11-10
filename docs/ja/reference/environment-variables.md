---
title: 環境変数
description: ARGVUSセッションコンポーネントで使用される環境契約。
---
セッションは XDG および Wayland の環境統合を確立し、検出された場合は仮想化互換性の変数をエクスポートすることがあります。`ARGVUS_CONFIG_HOME`、`ARGVUS_SYSTEM_CONFIG`、および `ARGVUS_BOOTSTRAP` は、セッションおよびディスパッチャーヘルパーによって構成ルートを解決するために使用されます。

環境変数はランタイム契約として扱います: 永続的なオーバーライドを追加する前に、インストールされているヘルパーの `--help` またはソースを確認してください。

派生状態は構成ルートではなく状態ルートの下に存在します。`XDG_STATE_HOME`（通常は `~/.local/state`）は `argvus/config-projection.json` を受け取り、投影マニフェスト `argvus-config` は再構築され `argvus sessionctl reload` が適用され、さらに `argvus/session.log` も適用されます。どちらも真実の情報源ではありません。

## 言語環境

唯一の一般的な言語の優先設定は`LANG`であり、`$XDG_CONFIG_HOME/locale.conf`（デフォルトは`~/.config/locale.conf`）で使用されます。有効な値がない場合、ARGVUSはポルトガル語が/etc/locale.conf、PAM、またはログインマネージャによって提供されても、英語（`en-US`）を選択します。 `/etc/locale.conf` `pt_BR.UTF-8`のような値は、カタログでは`pt-BR`に正規化されます。

セッションは、デスクトップコンポーネントの起動やsystemdユーザーおよびD-Busアクティベーション環境のインポートの前にこの優先設定を適用します。継承された`LANG`、`LC_MESSAGES`、`LANGUAGE`を置き換え、`LC_ALL`をクリアします。同じファイル内の地域ごとの`LC_*`設定は保持されます。英語は生成時に`en_US.UTF-8`を使用し、それ以外の場合は`C.UTF-8`を使用します。優先設定ファイルは自動的には作成されません。言語を変更した後は、ログアウトして再度ログインしてください。

`session.language`、`argvus/data/language`、および`argvus/data/internal/language`は無視されるレガシーデータです。これらはインポートも再生成もされず、既存のファイルは保持されます。`argvus config set`および`patch`は`session.language`への新しい書き込みを拒否し、呼び出し元を`locale.conf`に誘導します。

診断のために、`argvus i18n locale`、`cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"`、および`systemctl --user show-environment`を比較してください。`localectl status`はシステムを説明しており、ユーザーのARGVUS言語ではありません。`ARGVUS_I18N_DIR`は言語ではなく、開発用のカタログディレクトリを選択します。
