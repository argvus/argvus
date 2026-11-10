---
title: 패키지 매트릭스
description: ARGVUS 패키지 책임.
---

| 패키지 | 책임 |
| --- | --- |
| `argvus` | 마스터 패키지: 디스패처, 프로세스 내 명령(구성, 번역, 키, 프로젝트, 터미널, 시스템 모니터, 네트워크, 블루투스, 원격 측정, TUI 터미널, 설정), 공유 데이터 및 i18n 카탈로그. 모든 구성 요소 패키지가 이 패키지에 의존합니다. |
| `argvus` (헬퍼) | 동일 패키지 내의 헬퍼: 제어 센터 및 디스크 사용 분석기(터미널 애플리케이션), 그리고 작업 표시줄 달력, 이동식 장치, 로딩 오버레이, ARGVUS Focus 및 ARGVUS Clock용 GTK 4 애플리케이션. |
| `argvus` (그리터) | greetd용 로그인 그리터, 동일 패키지로 제공되며 `crates/argvus-greeter`에서 빌드됨. |
| `argvus-desktop` | `argvus` 마스터 패키지에 필요한 데스크톱 및 일상용 애플리케이션 프로필. `argvus`에 의존하지 않으므로 의존성은 단방향으로 유지됩니다. |
| `argvus-devtools`, `argvus-pentest` | 개발 도구 및 인증된 보안 감사용 선택적 독립 메타패키지입니다. 마스터 패키지는 이러한 프로필을 설치하지 않습니다. |
| `argvus-session` | 세션 진입점 및 사용자 수명 주기. |
| `argvus-hyprland` | Hyprland 구성. |
| `argvus-control-panel` | Quickshell 제어판. |
| `argvus-taskbar`, `argvus-waybar` | Waybar 구성 및 패치된 바이너리. |
| `argvus-branding` | 로고 및 워드마크 자산. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | 외관 상태 및 시각적 자산. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | 데스크탑 제공자. |
| `argvus-lock`, `argvus-boot-splash` | 잠금 화면 및 부팅 스플래시. |
| `argvus-firewall` | 자체 권한이 있는 방화벽. |
| `argvus-launcher` | 런처 구성 및 진입점. |
| `argvus-app-profiles` | 번들 터미널 애플리케이션용 데이터 전용 프로필입니다. |
| `argvus-portal` | XDG 포털 및 Wayland/DBus 기본값입니다. |
| `argvus-theme-*` | 공식 테마 계열별 선택형 패키지 하나입니다. |
| `argvus-game-snake`, `argvus-games` | 선택 사항입니다. Snake는 독립 프로젝트이며; `argvus-games`는 선택 게임을 그룹화합니다. |

## 교체된 패키지

`argvus`는 흡수한 패키지인 `replaces`, `conflicts` 및 `provides`를 선언하므로, 업그레이드하면 해당 패키지가 제거되고 파일이 고아로 남지 않습니다:

`argvus-config`, `argvus-i18n` (`argvus-language-en-us` 및 `argvus-language-pt-br` 포함), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` 및 `argvus-du`. `argvus`는 또한 `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` 및 이전 분할 패키지인 `argvus-gtk`와 `argvus-greeter`를 대체하며, 여전히 이들을 제공하여 해당 패키지에 의존하는 패키지가 계속 해결되도록 합니다. 계정 관리자는 (`argvus accounts`) `crates/argvus-accounts`로부터 빌드되어 `/usr/lib/argvus/commands/accounts`로 설치되며, polkit 규칙이 포함됩니다. 디스크 사용 분석기는 `crates/argvus-du`로부터 빌드되어 `/usr/lib/argvus/commands/argvus-du`로 설치됩니다.

`/usr/bin/argvus-terminal`는 `argvus terminal`와의 호환성 실행기로 설치된 상태로 유지됩니다. Hyprland의 기본 터미널, 제어 센터 카탈로그 및 저장된 기본 애플리케이션은 여전히 해당 명령어를 나타냅니다.

제거된 패키지 `argvus-pwvucontrol` 및 `argvus-snappy-switcher`는 어떤 패키지에도 제공되지 않습니다.

ARGVUS 구성 요소 패키지 간에는 설치 순서가 필요하지 않습니다: `argvus`가 이들의 마스터 패키지이며, 각 구성 요소는 이에 의존합니다. `argvus`를 설치하면 `argvus-desktop` 프로필도 함께 설치됩니다; `argvus-devtools` 및 `argvus-pentest`는 선택적으로 설치할 수 있습니다. 이 표에 패키지가 존재한다고 해서 해당 기능 전체를 소유한다는 의미는 아닙니다: 예를 들어, 작업 표시줄은 `argvus-waybar`, `argvus-taskbar`, `argvus-session` 및 도메인 제공자를 결합합니다.
