---
title: 鍵
description: SSHおよびGPGキーは、argvusキーおよびargvus gpgを通じて管理されます。
---
`argvus keys` は SSH 鍵ペアを管理し、`argvus gpg` は GPG 鍵リングを管理します。どちらも `argvus` パッケージのプロセス内コマンドであり、コントロールセンターの SSH および GPG ページで使用されます。

## コマンド

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
キーは決して印刷されません。パスフレーズは`ssh-keygen`または`ssh-add`を実行する端末に入力され、引数として渡されることはありません。`ARGVUS_SSH_DIR`を設定して、`~/.ssh`の代わりに別のディレクトリを使用してください。

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
破壊的および公開コマンドには`--yes`が必要です。コントロールセンターは自己確認の後にこれを渡します。ARGVUSのピンエントリは、ARGVUSが所有するブロック内の`gpg-agent.conf`に書き込まれます。あなた自身の行は保持されます。
