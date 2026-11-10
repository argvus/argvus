---
title: 钥匙
description: 通过 argvus keys 和 argvus gpg 管理的 SSH 和 GPG 密钥。
---
`argvus keys` 管理 SSH 密钥对，`argvus gpg` 管理 GPG 密钥环。两者都是 `argvus` 软件包的进程内命令，控制中心的 SSH 和 GPG 页面使用它们。

## 命令

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
密钥从不打印。访问密码是在运行 `ssh-keygen` 或 `ssh-add` 的终端中输入的；它从不作为参数传递。设置 `ARGVUS_SSH_DIR` 以使用其他目录代替 `~/.ssh`。

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
破坏性和发布命令需要 `--yes`；控制中心在自身确认后传递它。ARGVUS pinentry 被写入 `gpg-agent.conf` 中，一个 ARGVUS 拥有的块；你自己的行被保留。
