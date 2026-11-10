---
title: 계정
description: 로컬 사용자 프로필, 표시 이름, 아바타 및 그룹 구성원 정보는 argvus 계정을 통해 제공됩니다.
---
`argvus accounts`는 ARGVUS 데스크탑의 로컬 사용자 계정을 나열하고 편집합니다. 이는 `argvus` 패키지의 개인 실행 파일로, `/usr/lib/argvus/commands/accounts`에 설치됩니다. `argvus` 명령어가 이를 시작하며, 다른 계정이나 시스템 데이터를 변경하는 작업만 root 권한으로 실행됩니다.

## 명령어

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
## 표시 이름과 아바타

표시 이름은 표준 GECOS 필드에 저장됩니다. 유닉스 사용자 이름은 절대 변경되지 않습니다.

아바타는 PNG, JPEG 또는 WebP 형식의 이미지로, 최대 20 MiB까지 가능합니다. 이미지는 256×256 픽셀로 크기가 조정되고 `0644` 모드로 `~/.face`에 PNG 형식으로 저장됩니다. 이 위치는 그리터와 대부분의 디스플레이 관리자가 읽습니다. 자신의 아바타는 홈 디렉터리에 저장되며 권한이 필요하지 않습니다.

## 권한

`list`, `show`, `self`, `self groups` 및 자신의 아바타만 권한 없이 작동합니다. 다른 모든 변경 사항은 데스크톱의 polkit 에이전트에게 권한을 요청합니다. ARGVUS 규칙(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`)이 인증해야 할 사람을 결정합니다:

- 관리자 그룹(`wheel`, Arch 기본값, 또는 `sudo`)에 속한 사용자는 자신의 비밀번호로 확인;
- 다른 모든 사용자는 루트 비밀번호로 확인.

| 작업 | 인증 없이 | 인증 후 |
| --- | --- | --- |
| 사용자 목록, 계정 보기, 그룹 목록 | ✓ | ✓ |
| 자신의 아바타 변경 | ✓ | ✓ |
| 자신의 표시 이름 변경 | — | ✓ |
| 자신의 비밀번호 변경 | — | ✓ (현재 비밀번호 확인 후) |
| 다른 사용자의 이름, 아바타, 그룹 또는 비밀번호 변경 | — | ✓ |

자신의 비밀번호를 변경할 때 먼저 현재 비밀번호를 확인합니다. 잘못된 비밀번호는 인증 프롬프트가 나타나기 전에 거부됩니다.

`sudo`로 명령을 실행할 필요는 없습니다. 명령은 필요할 때 polkit에 인증을 요청합니다.

## 보안 제한

- `passwd`는 현재, 새 비밀번호 및 확인 비밀번호를 표준 입력에서 한 줄씩 읽으며, 이를 인수로는 절대 받지 않습니다. 현재 비밀번호 줄은 관리자가 다른 사용자의 비밀번호를 재설정할 때 비어 있을 수 있습니다. 예:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```
상호 작용식으로 세 줄을 입력하고 각 줄마다 Enter 키를 누릅니다. 입력한 내용은 터미널에 입력하는 대로 표시됩니다.
- UID 1000 이하(및 `nobody`)의 계정을 편집하는 것은 거부되며, 계정 잠금, 삭제 및 비밀번호 만료도 마찬가지입니다. 본인의 계정은 잠그거나 삭제할 수 없습니다.
- GID 1000 이하의 그룹 또는 관리자 그룹(`wheel`, `sudo`)의 이름 변경이나 삭제는 거부됩니다; 해당 그룹의 구성원은 변경할 수 있지만, 본인을 관리자 그룹에서 제거할 수는 없습니다.
- 관리자 멤버십은 `wheel` 또는 `sudo` 그룹입니다. 시스템에 따라 제어 센터는 새 관리자를 `wheel`에 추가하거나, 없을 경우 `sudo`에 추가합니다. 편집을 통해 본인의 관리자 멤버십을 제거하는 것은 거부됩니다.

## 예시

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
## 문제 해결

- `pkexec was not found`: `polkit`를 설치하세요. `argvus` 패키지는 이를 필요로 합니다.
- 권한 승인 프롬프트가 나타나지 않음: 세션에서 polkit 인증 에이전트가 실행 중인지 확인하세요. `argvus` 패키지는 이를 제공하는 `hyprpolkitagent`에 의존합니다.
- 비밀번호를 변경할 때 `unix_chkpwd` 오류 발생: `pam` 패키지가 도움말을 제공합니다.
- 명령어의 단계들을 stderr로 출력하려면 `argvus accounts --verbose ...`를 실행하세요.

권한 경계에 대한 개발자 노트는 `docs/internal/argvus-accounts/`를 참고하세요.

관리자가 자동 로그인을 위해 구성된 계정을 잠그거나 제거하면 계정 제공자는 먼저 그 계정의 greetd 자동 로그인 항목을 비활성화합니다. 해당 정리가 실패하면 계정 작업은 거부됩니다. 나중에 잠금 또는 제거가 실패하더라도 자동 로그인은 비활성 상태로 유지됩니다. 다른 계정의 자동 로그인은 유지되며, 계정을 잠금 해제한다고 해서 자동 로그인이 활성화되지 않습니다. 기존 세션은 종료되지 않으며 greetd는 재시작되지 않습니다.
