---
title: 서비스
description: ARGVUS systemd 및 사용자 서비스.
---
주요 사용자 대상은 `argvus-session.target`입니다. 일반적인 사용자 서비스에는 `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, 클립보드 서비스 및 `argvus-session-loading.service`가 포함됩니다.

방화벽은 `argvus-firewall.service`라는 시스템 서비스입니다. 캘린더 패키지는 `argvus-taskbar-calendar.service`를 사용자 유닛으로 설치하며, `argvus-clock.service`(기본적으로 비활성화됨)는 **세션과 함께 시작**이 켜져 있을 때 로그인 후 ARGVUS Clock을 숨겨진 상태로 시작합니다.

활성 유닛을 확인하려면:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```
## 세션 소유 유닛

세션 패키지는 `/usr/lib/systemd/user` 아래에 이러한 사용자 유닛을 설치합니다 (소스 페이로드는 세션 패키지의 공유 구성 트리 아래에 있음):

| 유닛 | 역할 |
| --- | --- |
| `argvus-session.target` | ARGVUS 그래픽 세션의 수명 경계. |
| `argvus-session-prepare.service` | 일회성 시작 준비 및 생성된 구성. |
| `argvus-control-panel.service` | 퀵셸 제어판. |
| `argvus-taskbar.service` | 메인 ARGVUS 작업 표시줄. |
| `argvus-widget-telemetry.service` | 선택적 텔레메트리/시스템 정보 표면. |
| `argvus-hyprland.service` | `hyprctl reload`를 실행하는 요청 시 일회성 유닛. |
| `argvus-config.service` | 표준 구성(`argvus config project`)을 프로젝션; 다시 로드 시 `argvus sessionctl reload-config-transaction` 실행.
| `argvus-session-reload.service` | 로그인 리로드(`argvus sessionctl reload-desktop --login`): 프로젝션이 영향을 미쳤을 때만 Hyprland를 리로드합니다. |
| `argvus-ssh-agent.service` | 세션을 위한 SSH 에이전트. |
| `argvus-secret-service.service` | `argvus sessionctl start-secret-service`를 통해 시크릿 서비스를 시작합니다. |
| `argvus-wallpaper.service` | 활성 배경화면 백엔드. |
| `argvus-dunst.service` | Dunst 알림 데몬. |
| `argvus-hypridle.service` | 유휴 및 잠금 정책. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | 클립보드 히스토리 감시기. |
| `argvus-keyboard-layout.service` | 키보드 레이아웃 알림. |
| `argvus-polkit.service` | PolicyKit 에이전트 통합. |
| `argvus-session-loading.service` | 컴포지터 전달에서 데스크탑 구성요소가 시작될 때까지의 세션 로딩 오버레이. |

`argvus-blueman-applet.service`도 세션 패키지에 의해 배송됩니다. 일부 유닛은 선택적이거나 조건부로 시작되며, 모든 세션에서 활성화되는 것은 아닙니다.

유닛의 소스만 확인하는 것이 아니라 실제 정의를 확인하려면:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```
시스템 수준 `argvus-firewall.service`는 `argvus-firewall`에 속합니다. 캘린더 패키지는 `argvus-taskbar-calendar.service`를 사용자 단위로 별도로 제공합니다. `argvus-clock.service`는 `argvus` 패키지의 사용자 단위이며, `WantedBy=graphical-session.target`는 ARGVUS 시계 설정에서 활성화됩니다.

## 세션 관리 명령어

`argvus sessionctl` 명령어는 세션 수명을 관리하며 로그아웃하지 않고 특정 구성 요소를 재시작할 수 있습니다:

### 세션 수명

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```
### 구성 요소별 재시작

전체 세션에 영향을 주지 않고 개별 서비스를 재시작하세요:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```
### 환경 동기화

환경 변수를 세션에 가져오기:

```sh
argvus sessionctl import-environment
```
이것은 Wayland, Hyprland, Qt, 커서 및 DBus 변수를 쉘과 systemd 사용자 서비스 간에 동기화합니다.

## 서비스 유닛 참조

추가 사용자 유닛은 `systemctl --user`를 통해 직접 관리할 수 있습니다:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```
초기 세션 부트스트랩 로그는 다음 위치에 저장됩니다:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
