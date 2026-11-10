---
title: 소개
description: ARGVUS가 무엇인지, 데스크톱이 어떻게 작동하는지, 그리고 구성 요소가 어디에 맞는지.
---
ARGVUS는 Arch Linux용 모듈식 Hyprland 및 Wayland 데스크탑 환경입니다. 이는 하나의 통합 애플리케이션이 아니라 패키지들의 통합 컬렉션으로 구성되어 있습니다: 세션, 컴포지터, 쉘 서피스, 설정 애플리케이션, 제공자 및 시각적 자산들이 공유 상태와 명시적 런타임 계약을 통해 함께 작동합니다.

이 페이지는 문서의 지도입니다. 전체 데스크탑을 높은 수준에서 설명하며, 링크된 페이지에는 운영 및 구현 세부 사항이 포함되어 있습니다.

## ARGVUS 작동 방식

일반 세션은 다음 경로를 따릅니다:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```
`argvus-session`는 세션 수명 주기와 환경 설정을 관리합니다. 모든 데스크톱 기능을 구현하지는 않습니다: 각 구성 요소는 자신의 명령, 구성 및 서비스 페이로드에 대한 소유권을 유지합니다.

설치 및 시작에 대해서는 [Installation](/docs/getting-started/installation/)]과 [Sessions](/docs/user-guide/sessions/)]를 참조하세요. 서비스 소유권에 대해서는 개발자 [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/)]를 참조하세요.

## 주요 계층

### 진입점 및 인프라

`argvus` 패키지는 `/usr/bin/argvus` 디스패처와 메인 패키지 진입점을 제공합니다. `argvus-session`는 그래픽 및 TTY 진입점, 사용자 대상 및 `argvus sessionctl`를 제공합니다. 공유 TUI 및 현지화 기능은 `argvus` 작업 공간(`argvus-tui` 및 `argvus-i18n` 크레이트)과 `argvus i18n` 명령에서 제공합니다.

### 컴포지터 및 데스크톱 셸

`argvus-hyprland`는 Hyprland 구성과 키 바인딩을 제공합니다. `argvus-waybar`는 패치된 Waybar 바이너리를 제공하며, `argvus-taskbar`는 작업 표시줄 구성과 동작을 소유합니다. `argvus-control-panel`는 Quickshell 패널을 제공하며, `argvus widget-telemetry`는 선택적 시스템 위젯을 제공하고, 이는 `argvus`에서 프로세스 내에서 실행됩니다.

사용법은 [Desktop](/docs/user-guide/desktop/)]를 참조하고, 구현 소유권은 [shell surfaces](/docs/developer-guide/subsystems/shell-surfaces/)]를 참조하십시오.

### 설정 및 애플리케이션

`argvus control-center`는 키보드 중심 설정 애플리케이션으로, `argvus`의 도우미로 제공됩니다. 캘린더와 이동식 장치 도구는 `argvus`의 GTK 4 도우미이며; 시스템 모니터와 터미널은 `argvus`의 하위 명령어입니다; 런처는 `argvus-launcher`입니다. 이들 모두는 셸과 디스패처와 통합되어 있습니다.

다음 [Applications](/docs/user-guide/applications/) 및 [command reference](/docs/reference/command-line/)를 참조하세요.

### 제공자 및 통합

디스플레이, 네트워크, 블루투스, 전원, 알림, 방화벽 및 이동식 저장소는 특정 프로젝트에서 제공합니다. 제어 센터와 제어판은 이러한 제공자를 사용하며, 이를 대체하지는 않습니다.

로그인, 잠금 및 시작 화면도 별도로 관리됩니다: `argvus-greeter`는 greetd를 처리하고, `argvus-lock`는 Hyprlock을 처리하며, `argvus`에 포함된 핸드오프 도우미는 로그인 스피너를 그리며, 로딩 오버레이는 테마 전환을 처리하고, `argvus-boot-splash`는 부팅 스플래시를 처리합니다.

다음 [Hardware](/docs/user-guide/hardware/), [Privacy and security](/docs/user-guide/privacy-and-security/) 및 개발자 [component reference](/docs/developer-guide/)를 참조하세요.

## 공유된 외형 상태

테마, 액센트, 배경화면, 글꼴 및 효과 설정은 논리적 상태로 `$XDG_CONFIG_HOME/argvus`(보통 `~/.config/argvus`)에 저장됩니다. `argvus-config`는 `data/generated/`를 쓰는 유일한 구성 요소이며, 여기서 Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, 알림, 잠금 화면, Yazi, Superfile 및 터미널의 사용자 파일을 생성합니다. Rofi는 예외입니다: `argvus launcher`는 구성 및 테마를 일반 파일로 패키징하며, 사용자가 편집한 복사본은 `data/rofi/`에 저장됩니다. 외관 도우미는 상태 변경을 커밋한 후 외부 애플리케이션과 조정하며, 생성된 트리를 직접 쓰지 않습니다. 

생성된 디렉터리는 진실의 출처가 아닙니다. [Appearance](/docs/user-guide/appearance/)] 및 [configuration and state](/docs/developer-guide/architecture/configuration-and-state/)]를 참조하세요.

시각 시스템은 레이아웃도 제어합니다. 스티키 모드는 컴팩트하고 정사각형이며, 스페이스트 모드는 더 넓은 창 간격, 둥근 모서리, 작업 표시줄과 셸 주변의 더 넓은 여백을 사용합니다. 작업 표시줄은 기본적으로 상단에 위치하지만, 공유 레이아웃 상태를 통해 상하 배치도 지원합니다. 효과는 합성기와 데스크톱 표면 전반의 투명도, 블러, 그림자 및 애니메이션을 제어합니다.

현재 패밀리와 색상은 [theme table](/docs/user-guide/appearance/)]를, 공유 효과 계약은 [effects](/docs/argvus-appearance/effects/)]를, 작업 표시줄 및 창 배치는 [taskbar](/docs/argvus-taskbar/taskbar/)]를 참조하십시오.

## ARGVUS가 무엇이고 무엇이 아닌가

ARGVUS는 Arch Linux 위의 데스크톱 환경 레이어입니다. Arch Linux는 운영 체제와 패키지를 제공하고, Hyprland는 Wayland 합성을 제공합니다. ARGVUS는 그래픽 세션, 셸 표면, 설정, 외관, 데스크톱 서비스 및 통합 애플리케이션을 조정합니다.

따라서 이것은 단순한 Hyprland 구성이나 개인 도트파일 모음 그 이상입니다. ARGVUS는 지원되는 세션 진입점을 정의하고, 데스크톱 서비스의 수명 주기를 관리하며, 공통 설정 및 외형 모델을 제공하고, 네트워킹, 디스플레이, 전원, 알림, 계정, 이동식 저장 장치와 같은 핵심 구성 요소를 연결합니다.

ARGVUS는 모든 애플리케이션을 대체할 필요가 없습니다. 데스크톱 전체 동작과 구성을 일관되게 유지하면서, 가장 신뢰할 수 있는 선택인 경우에만 선택된 상위 애플리케이션과 통합합니다.

이 이름은 두 가지 아이디어를 결합한 것입니다. *ARGV*는 *argument vector*에서 가져온 것으로, 프로그램이 받는 인수 목록을 의미하며, 명령 및 단축키를 통해 데스크톱을 제어하는 키보드 우선 목표를 반영합니다. *US*는 *User System*에서 가져온 것으로, 사용자를 중심으로 한 데스크톱을 목표로 함을 의미합니다. 전체 설명과 발음 오디오는 [About ARGVUS](/about/)]를 참조하십시오.

구현 경계와 패키지 소유권에 대해서는 개발자 [아키텍처 개요](/docs/developer-guide/architecture/overview/)]와 [저장소 역할](/docs/developer-guide/architecture/repository-roles/)]를 참조하세요.

## 목표별 계속 진행

- [FAQ 읽기](/docs/user-guide/faq/)].
- [ARGVUS 설치](/docs/getting-started/installation/)].
- [그래픽 또는 TTY 세션 시작](/docs/user-guide/sessions/)].
- [테마와 배경화면 설정](/docs/user-guide/appearance/)].
- [작업 표시줄과 제어판 사용](/docs/user-guide/desktop/)].
- [명령어, 경로 및 서비스 이해](/docs/reference/)].
- [아키텍처 학습](/docs/developer-guide/architecture/overview/)].
