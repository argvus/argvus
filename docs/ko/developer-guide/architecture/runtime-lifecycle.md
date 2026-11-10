---
title: 런타임 라이프사이클
description: 세션 시작 및 사용자 서비스 소유권.
---
`argvus-session`는 환경을 가져오고, `argvus start`를 통해 Hyprland를 시작하며, 합성기가 준비되면 `argvus-session.target`를 시작합니다. 대상은 셸, 알림, 배경화면, 유휴 및 클립보드 서비스를 가져옵니다. `argvus sessionctl`는 시작, 정지, 다시 불러오기, 재시작, 상태 및 로그를 조정합니다.

세션은 Hyprland가 종료되면 대상을 중지합니다. 개별 프로젝트는 자체 실행 파일과 구성에 대한 책임을 계속 유지합니다.

## 시작

그래픽 진입점은 `/usr/share/wayland-sessions/argvus.desktop`이며, 이는 `argvus session`를 호출합니다. 세션은 ARGVUS와 XDG 구성 환경을 가져와 `systemd --user` 및 D-Bus 활성화와 동기화하고, 합성기 시작을 `argvus start`에 위임합니다.

`argvus start`는 Hyprland 구성을 검증하고, 기본적으로 생성된 파일이 발견되거나 사용자 구성이 유효하지 않을 경우 패키지된 폴백을 적용하며, 가상화 호환성을 감지하고 Hyprland를 실행합니다. 컴포지터의 준비 브리지가 `argvus sessionctl ready`를 호출하면, 현재 Wayland 및 Hyprland 환경에서 ARGVUS 타겟을 시작합니다.

타겟은 장시간 실행되는 구성 요소를 시작하기 전에 일회성 준비 유닛을 사용합니다. 준비 과정에서는 생성된 구성, 테마 및 강조 색상 상태, 모니터 상태, 애플리케이션 통합 및 세션 환경을 적용합니다. 지속적인 서비스는 임시 백그라운드 프로세스가 아니라 `systemd --user`에 의해 관리됩니다.

`argvus start`는 Hyprland를 실행하기 바로 전에 `argvus-config.service`(`--no-block`)를 큐에 넣습니다: 프로젝션은 컴포지터가 필요 없으므로 컴포지터 시작과 겹칩니다. 그런 다음 `argvus sessionctl ready`가 타겟을 시작하며, 타겟은 해당 유닛이 활성 상태인지 확인하거나 그 작업을 기다리므로, 준비 실행 전에 정규 구성은 여전히 프로젝션됩니다. 타겟이 시작되면, `sessionctl ready`는 배경 레이어 표면인 월페이퍼와 작업 표시줄이 매핑될 때까지 최대 1.5초 동안 기다린 후 로딩 오버레이를 제거합니다. 단순 서비스는 그리기 전에 시작된 것으로 간주되므로, 실행되지 않은 유닛의 표면은 기다리지 않으며, 대기는 `session.desktop-surfaces`로 기록되고, 각 표면에 걸린 시간은 `session.wallpaper-surface` 및 `session.taskbar-surface`로 기록됩니다. 치명적인 프로젝션 또는 타겟 시작 실패는 기록되며 독립 로딩 오버레이를 중지하여 끝없어 보이는 스피너를 방지합니다. 선택 사항 시각적 소비자는 세션 대상을 교체하지 않고 폴백을 해결하거나 저하된 실패를 보고해야 합니다.

### 로그인 비용

구성 변경 없이 로그인하면 비용이 많이 드는 작업을 반복하지 않습니다:

- 프로젝션은 외관 또는 효과 섹션이 변경되었거나 실제 출력물(구현된 사용자 복사본 및 `data/state/effects/<theme>.conf`)이 없을 때만 테마 전환과 효과 어댑터를 실행합니다.
- 준비 단계는 `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp`의 스탬프가 패키지된 테마 파일 및 강조 표시와 더 이상 일치하지 않을 때만 시작 테마와 그 위에 강조 색을 적용합니다. 스탬프는 파일 내용을 지문처럼 기록하므로 동일한 파일을 다시 설치해도 무효화되지 않습니다. 테마 입력이 변경되거나 새 강조 색이 적용되면 한 번 무효화됩니다. 이전 타임스탬프 기반 스탬프를 마이그레이션하면 한 번 다시 빌드됩니다. 테마가 다시 적용될 때 준비 단계는 이전 파일에서 시작된 Hyprland도 다시 로드합니다.
- GTK 설정(`gsettings`)은 시작 테마가 다시 적용되었을 때만 작성됩니다. 로그인 사이에는 dconf가 이를 유지하므로, 안정적인 로그인에서는 다시 실행되지 않습니다. 
- `argvus-dunst.service`은 `Type=dbus`입니다: Dunst가 `org.freedesktop.Notifications`를 소유하게 되면 시작된 것으로 간주되므로, 세션 목표는 일시 중지 상태가 복원될 때까지 기다리지 않습니다. 
- 준비 단계에서는 `spaces-switch.sh`와 `borders-switch.sh`(`--apply-static`)을 나란히 실행합니다: 이들은 서로 다른 파일을 작성합니다(바의 JSONC 여백 및 실제 기하 구조, 바의 CSS 및 Rofi 테마). 
- Projection은 내용이 변경된 경우에만 생성된 파일을 다시 씁니다. 변경되지 않은 파일은 그대로 두므로, 안정적인 로그인에서는 다시 fsync되지 않습니다. 
- 준비 단계의 디스플레이 단계(`argvus config display session-prepare`)는 DPMS 및 비활성 출력에만 적용됩니다: 다시 투영하지도 않고 컴포지터를 재로드하지도 않습니다.
- `argvus-session-reload.service`는 `argvus sessionctl reload-desktop --login`를 실행하며, 이는 프로젝션 매니페스트가 영향을 받는 소비자 목록에 `hyprland`를 포함할 때만 Hyprland를 다시 로드합니다. 데스크톱 구성 요소는 이를 기다리지 않습니다.

세션 스피너(Wayland 모드에서의 핸드오프 헬퍼)는 `argvus sessionctl ready`가 대상 시작을 완료할 때까지 컴포지터 핸드오프에서 매핑되므로, 하나의 스피너가 전체 시작을 커버합니다. 새 로그인에서 대상을 다시 생성해야 할 때 종료하면 안 되므로 `PartOf=`는 없습니다; `sessionctl`는 이를 명시적으로 중지하며, `RuntimeMaxSec=30`는 중지가 도착하지 않으면 이를 종료합니다.

각 준비, 재로드 및 프로젝션 어댑터 단계는 세션 로그에 해당 기간을 `<epoch-ms> timing <step> <N>ms`로 추가합니다; [세션 시작 문제 해결](/docs/user-guide/troubleshooting/session-startup/)]를 참조하십시오.

경로 해결, 세션 준비 및 Hyprland 재로드 계획은 Rust가 관리합니다. `argvus config session-prepare`와 `argvus config session-reload-config`는 시작 및 재로드 단계를 실행하며; 기존의 `hypr-init.sh` 스크립트는 제거되었습니다.

## 서비스 및 종료

대상은 활성화 시 일반적으로 `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service`, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, 클립보드 감시자, 키보드 레이아웃 알림 및 PolicyKit 통합을 관리합니다. 세션 로딩 오버레이는 대상이 아닌 `argvus start`에 의해 시작됩니다. 선택적 서비스의 실패는 전체 세션을 대체하지 않습니다.

Control Panel과 idle manager를 포함하여 Wayland 클라이언트 연결이 필요한 서비스는 클라이언트를 실행하기 전에 사용 가능한 Hyprland 런타임을 기다립니다. 이는 컴포지터 전환 중에 Quickshell이나 hypridle을 오래된 소켓에 대해 시작하는 것을 방지합니다. Hyprland가 교체되면, systemd는 새 런타임이 사용 가능해진 후 구성 요소를 다시 시작할 수 있습니다.

테마 전환은 테마 잠금 하에서 하나의 생명주기 작업으로 커밋, 빌드, 프로젝션 및 소비자 재로드를 직렬화합니다. `argvus-config.service`를 재로드하거나 서비스를 재시작하지 않습니다. 실행되는 프로젝션은 단지 파일을 작성하고 잠금이 유지되는 동안 테마 전환을 남겨 두며, 실행 중인 소비자는 제자리에 재로드됩니다([Appearance and themes](/docs/developer-guide/subsystems/appearance-and-themes/) 참고).

Hyprland가 종료되면 `argvus start`는 `argvus-session.target`와 그에 연결된 서비스를 중지합니다. 이는 작업 표시줄, 셸, 알림, 유휴 및 클립보드 프로세스가 로그아웃 후에도 유지되는 것을 방지합니다.

## 다시 불러오기

```sh
argvus sessionctl reload
```

리로드는 그래픽 환경을 다시 가져온 후, 이전 도메인 명령(예: `apply-theme`, `accent`, `set`/`patch`)이나 위젯-원격 측정 블록 변경이 이미 변경 사항을 반영한 경우 **커밋된 세대**를 적용합니다. 이 경우 매니페스트 계획에 나열된 소비자를 복원하며, 동일한 트리를 다시 구축하고 계획을 폐기하게 되므로 `argvus config project`를 다시 호출하지 않습니다. 투영(projection), 마이그레이션, `ensure`는 유효한 대기 중 계획이 존재하지 않을 때만 수행됩니다 — 세션 시작 시 또는 `argvus config recover`가 이전 문서를 복원해야 할 때입니다. 투영과 관리 런타임이 모두 최신 상태일 때만 클린 무작동(clean no-op)이 허용됩니다. 정규 상태는 변경되지 않았지만, 합성기 소켓, 서비스 또는 소비자가 오래된 경우 리로드는 조기에 반환하지 않고 이를 조정합니다. 일반적인 팬아웃(fan-out)은 Hyprland, 작업 표시줄, 위젯 원격 측정, 제어판, 알림, 배경화면 및 유휴 상태를 포함합니다. 투영 매니페스트에 따르면, 변경되지 않은 선택적 소비자는 그대로 유지됩니다. 이 시점에서 `data/generated/` 아래의 모든 파일은 `argvus-config`에 의해 작성되므로, 세션은 스스로 테마를 다시 렌더링하지 않습니다. 그런 다음 재로드는 `theme-adapters.sh`를 실행하며, 이 어댑터는 자체적으로 정식 문서나 게시된 투영을 읽을 수 없는 소비자를 조정합니다. 이 어댑터들은 커밋된 세대만 읽습니다: 그들은 자신의 소비자 파일과 캐시만 작성하며, 정식 문서나 `data/generated/`는 절대 작성하지 않으므로, 단일 사용자 작업이라도 하나의 변이, 하나의 투영, 하나의 게시 및 하나의 재로드가 필요합니다. `argvus sessionctl restart <component>`는 단일 소유 서비스가 명시적으로 격리된 재시작이 필요할 때만 사용하십시오. 정확한 유닛 이름은 [services reference](/docs/reference/services/)]를 확인하십시오.

프로젝션 및 재로드 작업은 사용자 런타임 잠금과 함께 직렬화됩니다. 임시 타이밍 추적의 경우, 재로드를 호출하기 전에 `ARGVUS_SESSION_PERF=1`를 설정하고 `~/.local/state/argvus/session.log`를 확인하십시오; 이 추적은 정상적인 재로드 동작을 변경하지 않고 단계 경계를 기록합니다.
