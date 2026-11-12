---
title: 저장소 역할
description: 작업 공간 통합 후 ARGVUS 소스 저장소의 책임.
---
저장소는 탐색 페이지별이 아닌 책임별로 그룹화됩니다:

- workspace: `argvus` (디스패처, 인-프로세스 명령, GTK 4 도우미, 공유 크레이트, `crates/argvus-greeter`의 그리터 및 명령 데이터);
- session 및 integration: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, 그리고 그리터 (`argvus-greeter`, `argvus/crates/argvus-greeter`에서 빌드됨);
- workspace 외부 UI: 제어판, 작업 표시줄 (`argvus-taskbar`), 런처 및 Waybar;
- 프로바이더: 디스플레이, 전원, 방화벽, 알림;
- 자산 및 패키징: 외관, 바탕화면, 글꼴, 아이콘, 프로필.

## workspace로 통합됨

이 저장소들은 `de/argvus`로 병합되었습니다. 이들의 문서는 현재 workspace의 `docs/` 아래에 있으며, 저장소들은 역사적 기록을 위해서만 유지됩니다:

| 이전 저장소 | 현재 | 문서 |
| `argvus-config` | `argvus config` | [명령어 줄](/docs/reference/command-line/), [설정 파일](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [국제화 구성 요소](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | 명령어 줄 |
| `argvus-terminal` | `argvus terminal` | [터미널 구성 요소](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [시스템 모니터 구성 요소](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [네트워크 구성 요소](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | 명령어 줄 |
| `argvus-control-center` | `argvus control-center` (터미널 애플리케이션) | [제어 센터 구성 요소](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 도우미) | [작업 표시줄 달력 구성 요소](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 도우미) | [이동식 장치 구성 요소](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` 도우미) | 명령줄 |
| `argvus-tui-terminal` | `argvus tui-terminal` (작업 공간의 crate) | 명령줄 |
| `argvus-tui`, `argvus-theme` | 공유 크레이트 `crates/argvus-tui` 및 `crates/argvus-theme` | 개발자 노트 |
| `argvus-greeter` | `crates/argvus-greeter`에서 소스, `argvus` 패키지에 포함 | [그리터 구성 요소](/docs/components/argvus-greeter/) |
| `argvus-themes` | 은퇴. 각 공식 테마는 자체 `argvus-theme-*` 패키지입니다. | [테마 설치](/docs/user-guide/appearance/) |

통합 구성 요소의 개발자 노트는 `docs/internal/<repository>/`에 있습니다.

언어는 소유권 경계가 아닙니다: 각 구성 요소는 패키지와 함께 실행 시 도우미를 유지합니다. 패키지를 가로지르는 한 가지 확실한 규칙: `argvus-config`만 `data/generated/` 아래에 씁니다. `config.json`를 읽을 수 없는 애플리케이션에 도달해야 하는 구성 요소는 자체 도우미에 어댑터를 추가하고 표준 커밋 후 오케스트레이터가 이를 호출하도록 합니다; 생성된 파일을 절대 쓰지 않습니다. 공개 CLI는 `/usr/bin`에 속하며; 내부 도우미는 `/usr/lib/argvus/<component>` 아래 또는 런타임 시 필요할 때 commands 디렉터리에 위치합니다; 아키텍처에 독립적인 데이터는 `/usr/share/argvus/<component>` 아래에 남습니다.

설치된 패키지 경계에 대한 정보는 [package matrix](/docs/reference/package-matrix/)을 참조하십시오.
