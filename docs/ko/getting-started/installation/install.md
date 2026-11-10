---
title: ARGVUS 설치
description: ARGVUS를 설치, 업데이트, 제거 및 구성하십시오.
---
ARGVUS는 공식 ARGVUS 패키지 저장소를 통해 Arch Linux 패키지로 배포됩니다.

## 설치 전

ARGVUS를 설치하기 전에 공식 Arch Linux `multilib` 저장소를 활성화하십시오. `/etc/pacman.conf`를 편집하고 `[multilib]` 섹션과 그 `Include` 줄의 주석 처리를 해제하십시오:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```
그런 다음 패키지 데이터베이스를 동기화하고 전체 시스템 업그레이드를 수행하십시오:

```sh
sudo pacman -Syu
```

빠른 설치 프로그램은 ARGVUS 저장소를 구성하지만, `multilib`를 활성화하지는 않습니다.

## 빠른 설치

새로운 Arch Linux 설치에서, 다른 데스크톱 환경을 먼저 설치하지 않고도 바로 TTY에서 ARGVUS를 설치할 수 있습니다.

실행:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```
설치 프로그램:

1. ARGVUS 저장소 서명 키를 가져오고 로컬에서 서명합니다.
2. ARGVUS 패키지 저장소를 구성합니다.
3. 패키지 데이터베이스를 업데이트합니다.
4. `argvus` 패키지와 그 종속 항목을 설치합니다.

설치 후, 로그아웃하고 디스플레이 관리자에서 **ARGVUS**를 선택하십시오.

> **참고:** 빠른 설치 프로그램은 `curl`, `sudo` 및 표준 Arch Linux 패키지 관리 도구가 필요합니다.

## 저장소만 구성하기

데스크톱을 설치하지 않고 ARGVUS 패키지 저장소를 구성하려면, 다음을 실행하십시오:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```
그 후에 ARGVUS를 수동으로 설치하세요:

```sh
sudo pacman -Syu argvus
```
## 수동 설치

저장소를 단계별로 구성하려는 경우, 다음 명령어를 사용하십시오.

### 1. 저장소 서명 키 다운로드

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```
### 2. 서명 키 가져오기

```sh
sudo pacman-key --add /tmp/argvus.gpg
```
### 3. 키를 로컬에서 서명하기

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```
### 4. ARGVUS 저장소 구성하기

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```
`/etc/pacman.conf`에 저장소 구성을 추가하십시오:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```
저장소 구성은 SigLevel = Required TrustedOnly과 함께 `argvus` 및 `argvus-extras`를 포함한 ARGVUS 저장소를 활성화합니다. Pacman은 패키지의 서명이 유효하고 로컬 키링에 의해 신뢰될 때만 패키지를 수락합니다.

### 5. ARGVUS 설치

```sh
sudo pacman -Syu argvus
```
설치 후, 로그아웃하고 디스플레이 관리자에서 **ARGVUS**를 선택하십시오.

## 업데이트

다른 Arch Linux와 함께 ARGVUS를 업데이트하십시오:

```sh
sudo pacman -Syu
```
ARGVUS 패키지만 업데이트하려면:

```sh
sudo pacman -S argvus
```

> **권장 사항:** Arch Linux는 일반적으로 개별 패키지를 업데이트하는 대신  `SigLevel = Required TrustedOnly` `sudo pacman -Syu`으로 전체 시스템 업그레이드를 권장합니다.

## ARGVUS 제거

다른 설치된 패키지에서 여전히 필요한 의존성은 유지하면서 ARGVUS를 제거합니다:

```sh
sudo pacman -R argvus
```
더 이상 설치된 패키지에서 필요하지 않은 ARGVUS 및 종속 항목을 제거합니다:

```sh
sudo pacman -Rns argvus
```
### ARGVUS 저장소 제거

패키지 저장소도 제거하려면, 해당 설정을 삭제하세요:

```sh
sudo rm /etc/pacman.d/argvus.conf
```
그런 다음 `/etc/pacman.conf`에서 해당 줄을 제거하십시오:

```text
Include = /etc/pacman.d/argvus.conf
```
다음으로 파일을 편집할 수 있습니다:

```sh
sudo nano /etc/pacman.conf
```
### 서명 키 제거

pacman 키링에서 ARGVUS 저장소 서명 키를 제거하려면 먼저 키를 나열하십시오:

```sh
sudo pacman-key --list
```
그런 다음 ARGVUS 키와 해당 키 ID를 제거하십시오:

```sh
sudo pacman-key --delete KEY_ID
```
`KEY_ID`를 ARGVUS 서명 키 ID로 교체하세요.

> **경고:** ARGVUS 저장소를 제거할 때만 이 키를 제거하세요. 다른 저장소에서 여전히 필요한 키는 제거하지 마세요.

## 사용자 구성

패키지 설치는 `$HOME`에 구성을 기록하지 않습니다. ARGVUS는 `/usr/share/argvus`에서 패키지 기본값을 읽으며, 홈 디렉토리에 복사하지 않고도 작동합니다.

런타임 구성은 일반적으로 다음 우선순위를 따릅니다:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```
`argvus-config`는 `~/.config/argvus/data/generated/`를 작성하는 유일한 구성 요소이며, 그 투영은 활성 레이어입니다: 관리되거나 네이티브 복사는 해당 생성 파일이 없을 때만 유효합니다.

ARGVUS 관리 구성은 `~/.config/argvus/` 아래에 존재합니다. 선택적 `$XDG_CONFIG_HOME/<app>` 오버라이드는 네이티브 사용자 구성을 지원하는 애플리케이션에서 사용할 수 있습니다.

`argvus --setup` 디스패처는 선택 사항이며 명시적 수동 오버라이드를 위한 것이지, 복구를 위한 것은 아닙니다. `~/.config/argvus`가 삭제되면, `SUPER + SHIFT + R`는 정상적인 재로드 파이프라인을 통해 정규 프로필과 생성된 런타임 상태를 재생성합니다.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```
`--copy-all`은 지원되는 TUI 프로필 파일의 명시적 허용 목록입니다; 이는 `/usr/share/argvus`를 복제하지 않으며 스크립트, 서비스, 자산, 생성된 파일 또는 `config.json`을 복사하지 않습니다. 현재 옵션을 보려면 `argvus --setup --help`를 실행하십시오. 복사되면 파일은 사용자 소유의 재정의가 되며 패키지 업그레이드가 이를 대체하지 않습니다.

기존에 복사된 구성을 교체하려면 `--force`를 사용하십시오. ARGVUS는 교체하기 전에 타임스탬프가 있는 백업을 만듭니다:

```sh
argvus --setup --copy <app> --force
```
## Hyprland 사용자 재정의

전체 Hyprland 재정의 가이드와 전체 네이티브 구성과 점진적 ARGVUS 조각의 차이를 포함하여, [Hyprland overrides](/docs/argvus-hyprland/hyprland-overrides/)]를 참조하세요.

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
### 가상 머신 실패 대체

일부 가상 머신은 Hyprland, Kitty 또는 Quickshell과 함께 렌더링 문제를 경험할 수 있습니다. ARGVUS는 `vmwgfx` 그래픽 드라이버를 사용하는 가상 머신을 감지하고 필요할 경우 소프트웨어 렌더링을 활성화합니다.

가상 머신에서 감지가 작동하지 않는 경우 `user.lua`에서 수동으로 실패 대체를 활성화하세요:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
`user.lua`를 생성하거나 수정한 후, Hyprland에서 로그아웃하고 다시 로그인하여 세션에서 실행되는 애플리케이션에 변수가 적용되도록 합니다.

다음으로 확인합니다:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```
예상 출력은:

```text
1
```
> **참고:** `LIBGL_ALWAYS_SOFTWARE=1`은 Mesa/OpenGL 애플리케이션이 보통 `llvmpipe`를 통해 소프트웨어 렌더링을 사용하도록 강제합니다. 이는 그래픽 성능을 저하시킬 수 있으며, 영향을 받는 가상 머신에서만 대체 방법으로 사용해야 합니다.
