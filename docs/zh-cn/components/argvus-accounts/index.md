---
title: 账户
description: 通过argvus账户，本地用户配置文件、显示名称、头像和组成员身份。
---
`argvus accounts` 列出并编辑 ARGVUS 桌面的本地用户账户。它是 `argvus` 软件包的私有可执行文件，安装在 `/usr/lib/argvus/commands/accounts`。`argvus` 命令用于启动它，只有更改其他账户或系统数据的操作会以 root 权限运行。

## 命令

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
## 显示名称和头像

显示名称储存在标准的 GECOS 字段中。Unix 用户名从未更改。

头像是 PNG、JPEG 或 WebP 格式的图像，最大 20 MiB。图像被调整为 256×256 像素并以 PNG 格式保存，模式为 `0644`，保存在 `~/.face`，这是 greeter 和大多数显示管理器读取的 freedesktop 位置。您自己的头像写入您的主目录，不需要授权。

## 授权

只有 `list`、`show`、`self`、`self groups` 和您自己的头像可以在无需授权的情况下使用。任何其他更改都需要桌面的 polkit 代理进行授权。ARGVUS 规则（`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`）决定谁必须进行身份验证：

- 属于管理员组的用户（`wheel`，Arch 默认或 `sudo`）使用自己的密码确认；
- 其他任何用户使用 root 密码确认。

| 操作 | 未经授权 | 已授权 |
| --- | --- | --- |
| 列出用户，显示账户，列出组 | ✓ | ✓ |
| 更改自己的头像 | ✓ | ✓ |
| 更改自己的显示名称 | — | ✓ |
| 更改自己的密码 | — | ✓（在验证当前密码后） |
| 更改其他用户的名称、头像、组或密码 | — | ✓ |

更改自己的密码时，系统会先检查当前密码。密码错误将在授权提示出现之前被拒绝。

无需使用 `sudo` 来运行命令。命令在需要时会向 polkit 请求授权。

## 安全限制

- `passwd` 从标准输入读取当前密码、新密码和确认密码，每行一个，且从不将它们作为参数。当管理员重置其他用户的密码时，当前密码行可以为空。例如：

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```
以交互方式，输入三行，每行输入后按回车。在输入时，终端会显示输入内容。
- 拒绝编辑 UID 小于 1000（以及 `nobody`）的账户，也不能锁定、删除或使其密码过期。您自己的账户不能被锁定或删除。
- 拒绝重命名或删除 GID 小于 1000 的组，或管理员组（`wheel`、`sudo`）；其成员仍然可以更改，但不能将自己从管理员组中移除。
- 管理员成员身份为 `wheel` 或 `sudo` 组。当系统存在 `wheel` 时，控制中心会向该组添加新管理员，否则会添加到 `sudo`。通过编辑移除自己的管理员成员身份是被拒绝的。

## 示例

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
## 故障排除

- `pkexec was not found`：安装 `polkit`。`argvus` 包依赖它。
- 授权提示未出现：检查你的会话中是否有 polkit 认证代理正在运行。`argvus` 包依赖 `hyprpolkitagent`，它提供了一个代理。
- 更改密码时出现 `unix_chkpwd` 错误：`pam` 包提供了辅助工具。
- 运行 `argvus accounts --verbose ...` 可将命令的步骤输出到 stderr。

关于权限边界的开发者说明，请参见 `docs/internal/argvus-accounts/`。

当管理员锁定或删除配置为自动登录的账户时，账户提供者首先会禁用其 greetd 自动登录条目。如果该清理操作失败，则拒绝执行账户操作。如果随后锁定或删除操作失败，自动登录仍将被禁用。另一个账户的自动登录将被保留；解锁账户不会启用自动登录。现有会话不会被关闭，greetd 也不会重启。
