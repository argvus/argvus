---
title: 열쇠
description: SSH 및 GPG 키는 argvus keys 및 argvus gpg를 통해 관리됩니다.
---
`argvus keys`는 SSH 키 쌍을 관리하고 `argvus gpg`는 GPG 키링을 관리합니다. 두 명령어 모두 `argvus` 패키지의 인프로세스 명령어이며, 컨트롤 센터의 SSH 및 GPG 페이지에서 사용됩니다.

## 명령어

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```
키는 절대 출력되지 않습니다. `ssh-keygen` 또는 `ssh-add`를 실행하는 터미널에 암호 구문을 입력하며, 절대 인수로 전달되지 않습니다. `ARGVUS_SSH_DIR`를 설정하여 `~/.ssh` 대신 다른 디렉토리를 사용할 수 있습니다.

## GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```
파괴적 및 게시 명령에는 `--yes`가 필요하며, 제어 센터는 자체 확인 후 이를 전달합니다. ARGVUS 핀엔트리는 ARGVUS가 소유한 블록 안의 `gpg-agent.conf`에 기록됩니다; 귀하의 자체 라인은 보관됩니다.
