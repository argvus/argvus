---
title: Chìa khóa
description: Khóa SSH và GPG được quản lý thông qua khóa argvus và argvus gpg.
---

`argvus keys` quản lý các cặp khóa SSH và `argvus gpg` quản lý khóa GPG. Cả hai đều là
các lệnh trong quá trình của gói `argvus` và các trang SSH và GPG của Trung tâm điều khiển sử dụng chúng.

## Lệnh

```text
argvus keys                              show the help
argvus keys list                         list the key pairs in ~/.ssh and whether they are loaded
argvus keys generate NAME [--type ed25519|rsa] [--comment C] [--passphrase]
argvus keys load NAME                    add the key to the agent
argvus keys unload NAME                  remove the key from the agent
argvus keys unload-all                   remove every key from the agent
argvus keys delete NAME                  delete the key pair from ~/.ssh
```

Chìa khóa không bao giờ được in. Cụm mật khẩu được nhập vào thiết bị đầu cuối chạy `ssh-keygen` hoặc
`ssh-add`; nó không bao giờ được thông qua như một đối số. Đặt `ARGVUS_SSH_DIR` để sử dụng thư mục khác
thay vì `~/.ssh`.

##GPG

```text
argvus gpg list                          keys as TSV
argvus gpg status                        API version, gnupg and pinentry
argvus gpg import FILE                   public keys only
argvus gpg import-private FILE           a private key, kept protected
argvus gpg delete FINGERPRINT --yes
argvus gpg keyserver send|recv FINGERPRINT [--yes]
```

Các lệnh hủy và xuất bản cần `--yes`; Trung tâm điều khiển sẽ vượt qua nó sau
xác nhận. Pinentry ARGVUS được ghi vào `gpg-agent.conf` bên trong một khối mà ARGVUS
sở hữu; dòng riêng của bạn được giữ.
