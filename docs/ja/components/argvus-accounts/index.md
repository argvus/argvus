---
title: アカウント
description: ローカルユーザープロファイル、表示名、アバター、およびグループメンバーシップを、argvusアカウントを通じて。
---
`argvus accounts`は、ARGVUSデスクトップのローカルユーザーアカウントを一覧表示および編集します。これは`argvus`パッケージのプライベート実行ファイルで、`/usr/lib/argvus/commands/accounts`にインストールされています。`argvus`コマンドで起動され、他のアカウントやシステムデータを変更する操作のみがrootとして実行されます。

## コマンド

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```
## 表示名とアバター

表示名は標準の GECOS フィールドに保存されます。Unix のユーザー名は変更されません。

アバターは PNG、JPEG または WebP 形式の画像で、最大 20 MiB です。画像は 256×256 ピクセルにリサイズされ、PNG 形式で `0644` モードで `~/.face` に保存されます。これは greeter やほとんどのディスプレイマネージャによって読み取られる freedesktop の場所です。自分自身のアバターはホームディレクトリ内に書き込まれ、認証は必要ありません。

## 認証

`list`、`show`、`self`、`self groups`、そして自分自身のアバターのみが認証なしで動作します。それ以外の変更はすべてデスクトップの polkit エージェントに認証を要求します。ARGVUS ルール (`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) が誰が認証する必要があるかを決定します:

- 管理者グループ (`wheel`、Arch のデフォルト、または `sudo`) のユーザーは自分のパスワードで確認します;
- その他のユーザーは root のパスワードで確認します。

| 操作 | 許可なし | 許可あり |
| --- | --- | --- |
| ユーザーの一覧表示、アカウントの表示、グループの一覧表示 | ✓ | ✓ |
| 自分のアバターを変更する | ✓ | ✓ |
| 自分の表示名を変更する | — | ✓ |
| 自分のパスワードを変更する | — | ✓（現在のパスワードを証明した後） |
| 他のユーザーの名前、アバター、グループ、またはパスワードを変更する | — | ✓ |

自分のパスワードを変更する場合、最初に現在のパスワードを確認します。パスワードが間違っている場合、認証プロンプトが表示される前に拒否されます。

`sudo` を使ってコマンドを実行する必要はありません。コマンドは必要に応じて polkit に認可を要求します。

## セキュリティ制限

- `passwd` は標準入力から現在のパスワード、新しいパスワード、確認用パスワードをそれぞれ1行ずつ読み込み、引数として受け取ることはありません。管理者が他のユーザーのパスワードをリセットする場合、現在のパスワード行は空にすることができます。例:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  対話形式で、3行を入力し、それぞれの後にEnterキーを押します。入力された内容は
  ターミナルにタイプした通りに表示されます。
- UID 1000 未満（および `nobody`）のアカウントを編集することは拒否され、ロック、削除、
  パスワードの有効期限切れもできません。自分自身のアカウントはロックしたり削除したりできません。
- GID 1000 未満のグループ、または管理者グループ（`wheel`、`sudo`）の
  名前変更や削除は拒否されます。メンバーの変更は可能ですが、自分を管理者グループから
  削除することはできません。
- 管理者メンバーシップは `wheel` または `sudo` グループにあります。 
  コントロールセンターは、システムに存在する場合は新しい管理者を `wheel` に、
  存在しない場合は `sudo` に追加します。編集によって自分自身の管理者メンバーシップを
  削除することは拒否されます。

## 例

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```
## トラブルシューティング

- `pkexec was not found`: `polkit` をインストールしてください。`argvus` パッケージはそれに依存しています。
- 認証プロンプトが表示されない場合: セッションでポルキット認証エージェントが実行されていることを確認してください。`argvus` パッケージは `hyprpolkitagent` に依存しており、それが認証エージェントを提供します。
- パスワードを変更するときに `unix_chkpwd` エラーが発生する場合: `pam` パッケージがヘルパーを提供します。
- `argvus accounts --verbose ...` を実行して、コマンドの手順を stderr に出力してください。

特権境界に関する開発者向けノートは `docs/internal/argvus-accounts/` を参照してください。

管理者が自動ログイン用に設定されたアカウントをロックまたは削除すると、アカウントプロバイダーは最初にその greetd の自動ログインエントリを無効にします。そのクリーンアップが失敗した場合、アカウント操作は拒否されます。後でロックまたは削除が失敗した場合でも、自動ログインは無効のままです。別のアカウントの自動ログインは維持されます；アカウントのロック解除は自動ログインを有効にしません。既存のセッションは閉じられず、greetd は再起動されません。
