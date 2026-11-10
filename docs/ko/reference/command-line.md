---
title: 명령 줄
description: argvus 명령어, 그 하위 명령어들, 키 바인딩을 위해 유지되는 플래그, 그리고 각각이 실행되는 위치.
---
`/usr/bin/argvus`는 ARGVUS의 공용 진입점입니다. 이것은 `argvus-cli` 크레이트로 빌드된 하나의 Rust 프로그램입니다. 각 하위 명령어는 해당 프로그램 내에서 실행되거나, commands 디렉토리의 도우미 실행 파일을 실행하거나, 패키지화된 스크립트를 실행합니다.

사용 중인 버전과 함께 설치된 목록은 `argvus --help`를 실행하세요. 하위 명령어는 선행 `--` 없이 작성됩니다. `--` 형식은 키 바인딩이나 데스크탑 항목에서 사용하는 경우에만 유지됩니다.

## 각 명령어의 실행 위치

- **프로세스 내:** 명령어는 워크스페이스의 라이브러리이며 `argvus` 내에서 실행됩니다. 별도의 실행 파일은 존재하지 않습니다. 이들은 `config`, `theme`, `i18n`, `keys`, `gpg`, `projects`, `terminal`, `system-monitor`, `network`, `bluetooth`, `focus status`, `widget-telemetry`, `tui-terminal`, `sensors` 및 `setup`입니다.
- **도우미:** argvus 패키지에 의해 설치되는 `/usr/lib/argvus/commands/`의 별도 실행 파일입니다. 이는  `argvus` `control-center`(터미널 애플리케이션), `argvus-du`(`argvus du`로 시작) 및 GTK 4 애플리케이션 `taskbar-calendar`, `removable-devices`, `focus` 및 `clock`입니다.
- **전용 실행 파일:** `/usr/lib/argvus/commands/`에 다른 패키지가 설치하는 실행 파일입니다. `argvus`만 시작합니다. 여기에는 `session`, `start`, `tty`, `sessionctl`, `launcher`, `snippets`, `ssh`, `git`, `app-profiles-yazi-keymap`, `notifications`, `displayctl`, `firewall`, `accounts`, `taskbar-actions` 및 `greeter-*` 진입점.
- **스크립트:** `/bin/sh`로 실행되는 패키지된 셸 스크립트입니다. 아래 나열된 Hyprland 스크립트 명령은 다음과 같습니다.

`ARGVUS_COMMANDS_DIR`는 테스트 및 `DESTDIR` 검사를 위해 명령 디렉터리를 변경합니다. 찾을 수 없는 개인 실행 파일은 `PATH`에서 조회됩니다.

## 하위 명령

| 명령 | 실행 | 메모 |
| --- | --- | --- |
| `argvus config ...` | 진행 중 | 구성 모델 및 도구. `theme-switch` 및 `effects-toggle`에서도 접근 가능. |
| `argvus theme ...` | 진행 중 | `apply ID`는 테마 전환기(`theme-switch`와 동일). 테마 빌드: 설치된 테마 `list`, 설치된 템플릿에 대한 테마 `check DIRECTORY`, `render ID [--variant sticky\|spaced] [--accent #RRGGBB] [--output DIRECTORY]`. [외관 및 테마](/docs/developer-guide/subsystems/appearance-and-themes/)] 참조. |
| `argvus i18n ...` | 진행 중 | 번역: `get DOMAIN KEY [name=value...]`, `locale`, `list`, `validate [LOCALE]`. |
| `argvus keys ...` | 진행 중 | SSH 키 관리자. `gpg`는 GPG 하위 명령어의 별칭. |
| `argvus gpg ...` | 진행 중 | GPG 키: `list`, `status`, `generate`, `export-public`, `export-private`, `import`, `import-private`, `delete`, `edit`, `keyserver`. |
| `argvus projects ...` | 진행 중 | 프로젝트 선택기. |
| `argvus terminal ...` | 진행 중 | ARGVUS 터미널 프로파일. |
| `argvus system-monitor ...` | 진행 중 | btop 시스템 모니터; `--gpu`은 GPU 전용 박스를 선택하고, `--panel [--cpu|--mem|--gpu]`은 이를 ARGVUS TUI 셸에 내장합니다. |
| `argvus du` | 도우미(터미널 애플리케이션) | 전용 Kitty 창에서 ARGVUS 디스크 사용량 분석기를 열고 ncdu를 실행합니다. |
| `argvus network ...` | 진행 중 | NetworkManager 상태 및 제어(`networkctl`는 별칭으로 유지). |
| `argvus bluetooth ...` | 진행 중 | Bluetooth 상태 및 제어(`bluetoothctl`는 별칭으로 유지). |
| `argvus widget-telemetry block ...` | 진행 중 | Waybar 텔레메트리 블록. 이는 바(bar)의 주요 경로입니다. |
| `argvus widget-telemetry toggle ...` | 진행 중 | 텔레메트리 블록을 켜고 끄며, 순서를 변경합니다. |
| `argvus game-snake` | 개인 실행 파일 | 독립 프로젝트 `argvus-game-snake`의 터미널 스네이크 게임으로, 선택적 패키지입니다. 터미널 외부에서 실행하면 하나가 열립니다. |
| `argvus control-center [page]` | 도우미 (터미널 응용 프로그램) | 제어 센터를 열거나 이미 열려 있는 창에 포커스를 맞춥니다. |
| `argvus welcome [--if-needed]` | 디스패처 → 제어 센터 | 제어 센터 내 통합된 환영 페이지를 엽니다. `--if-needed`는 사용자별 진행 기록에서 환영 화면이 이미 표시되었거나 완료되었거나 닫힌 경우를 제외하고 한 번 열립니다. |
| `argvus taskbar-calendar [args]` | GTK 4 응용 프로그램 | 기본값은 `toggle`입니다. |
| `argvus removable-devices [args]` | GTK 4 응용 프로그램 | 기본값은 `menu`입니다. |
| `argvus focus [--toggle] [--start\|--pause] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 응용 프로그램 | ARGVUS 포커스 팝업. 세션당 한 인스턴스. |
| `argvus focus status --waybar [--follow]` | 진행 중 | Focus 작업 표시기(태스크바)를 Waybar JSON으로 내보냅니다; `--follow`는 GTK를 로드하지 않고 변경 사항을 스트리밍합니다. |
| `argvus clock [--toggle] [--background] [--page PAGE] [--x X --y Y] [--bar-bottom PX --bar-right PX]` | GTK 4 애플리케이션 | ARGVUS 시계 팝업. 세션당 한 인스턴스; `--background`는 팝업 없이 알람을 활성 상태로 유지합니다. |
| `argvus loading-theme ...` | `splash`를 `/usr/lib/argvus/loading-theme/splash`로 패키징 | 직접 세션 다시 로드의 오버레이; 테마 전환에는 사용하지 않습니다. |
| `argvus session`, `start`, `tty`, `sessionctl` | 개인 실행 파일 | 세션 진입점 및 세션 컨트롤러. |
| `argvus greeter`, `greeter-session`, `greeter-tui`, `greeter-setup` | 개인 실행 파일 | 그리터 진입점. |
| `argvus accounts`, `firewall` | 개인 실행 파일 | 계정 및 방화벽 도구. |
| `argvus launcher`, `snippets`, `ssh`, `git` | 비공개 실행 파일 | 실행기 및 선택 도구. |
| `argvus tui-terminal` | 프로세스 내 | TUI 애플리케이션용 터미널 래퍼. |
| `argvus sensors` | 프로세스 내 | `sensors -j`에서 실시간 온도 개요. |
| `app-profiles-yazi-keymap` | 비공개 실행 파일 | 키맵 도우미. |
| `argvus notifications`, `displayctl`, `taskbar-actions` | 비공개 실행 파일 | 알림, 디스플레이 및 작업 표시줄 동작. |
| `argvus setup [options]` | 프로세스 내 | 패키지된 기본값을 사용자 구성에 복사합니다. 아래 참조. |

인수는 공백과 빈 값을 포함하여 변경 없이 전달됩니다. 표준 입력, 출력, 오류, 신호 및 종료 상태는 실행되는 명령에 속합니다. 알 수 없는 이름은 `argvus --help`를 실행하라는 힌트와 함께 실패합니다. 프로세스 내에서 실행되는 명령은 유효하지 않은 UTF-8 인수를 전달된 값으로 바꾸지 않고 거부하며(종료 상태 2), 별도의 프로그램을 실행하는 명령은 바이트를 변경 없이 받습니다.

## 플래그 및 별칭

이 이름들은 키바인딩, 데스크톱 항목 및 이전 스크립트가 계속 작동하도록 존재합니다. 새 코드에서는 서브커맨드 형태를 사용하는 것이 좋습니다.

| 플래그 또는 별칭 | 동일한 의미 |
| --- | --- |
| `--control-center`, `--about`, `--default-apps` | `control-center` (`about` 및 `apps` 페이지 포함) |
| `--displays`, `--monitors` 페이지 | `control-center displays` |
| `--network-interfaces` | `control-center network interfaces` |
| `--system-monitor`, `--terminal`, `--game-snake` | `system-monitor`, `terminal`, `game-snake` |
| `--calendar`, `--removable-devices`, `--storage` | `taskbar-calendar`, `removable-devices` |
| `--focus`, `--clock` | `focus`, `clock` |
| `--spf`, `--superfile` | 파일 관리자(`spf`) |
| `--yazi`, `--yazy` | 파일 브라우저(`yazi`) |
| `--setup` | `setup` |
| `--start-desktop` | `tty` 시작(인수가 전달되지 않음) |
| `--version`, `-V`, `version` | `ARGVUS <version>`를 인쇄합니다 |
| `help`, `--help`, `-h` | 사용법을 인쇄합니다 |
| `networkctl`, `bluetoothctl` | `network`, `bluetooth` |
| `widget-telemetry-toggle`, `widget-telemetry-block` | `widget-telemetry toggle`, `widget-telemetry block` |
| `theme-switch` | `config theme-apply` |
| `effects-toggle` | `config effects` |

제어 센터는 레거시 페이지 플래그를 첫 번째 인수로 받아들이고 이를 페이지 이름에 매핑합니다. 예: `--apps`는 `apps`가 되고, `--keyboard-layout`는 `locale`가 되고, `--hostname`는 `system`가 됩니다.

## Hyprland 스크립트 명령어

이들은 패키지로 제공되는 스크립트를 실행하며 Hyprland 구성에서 사용됩니다:

`cheatsheets-hypr`, `cheatsheets-launcher`, `scratchpad`, `hypr-screenshot`, `spaces-switch`, `emoji-picker`, `toggle-sidebar`, `weather-location`, `layout-mode-menu`, `brightness-switch`, `toggle-mode`, `idle-timeout`, `keep-awake`, `hypr-power-menu`.

사용자의 구성에 있는 스크립트는 패키지화된 스크립트를 덮어씁니다. 조회 순서는 hyprland.lua의 `_config_path`와 동일합니다: 사용자의 구성 복사본, 그 다음 ARGVUS 데이터 복사본, 그 다음 상태 복사본, 그 다음 생성된 복사본, 그 이후에야  `hyprland.lua` `/usr/share/argvus` 아래의 패키지 파일이 적용됩니다.

## 예시

```sh
argvus control-center displays
argvus taskbar-calendar toggle
argvus widget-telemetry block header system
argvus widget-telemetry toggle status
argvus i18n get widget-telemetry label.device
argvus config get /appearance/theme
argvus network status
argvus bluetooth status
argvus gpg list
```
## 설정

`argvus setup`는 `/usr/share/argvus/<app>`에서 `~/.config/argvus/data/<app>`로 패키지된 기본값을 복사하여 사용자가 이를 커스터마이즈할 수 있도록 합니다. 패키지된 파일을 편집하지는 않습니다.

```sh
argvus setup --copy <app>          # copy one packaged app
argvus setup --copy-all            # copy every packaged app directory
argvus setup --copy <app> --force  # back up the existing destination first
argvus setup --copy <app> --dry-run
```
이미 존재하는 대상은 `--force`가 제공되지 않는 한 보고되고 그대로 둡니다. `--force`가 주어지면, 기존 디렉터리는 복사 전에 `<name>.bak-<timestamp>`로 이름이 바뀝니다.

`argvus` 앱은 특별합니다: 패키지된 디렉터리의 내용을 구성 루트에 복사합니다. 구성 루트는 설정이 실행될 때 이미 존재하기 때문에, 현재 코드에서는 `--copy argvus`가 `--force`를 필요로 합니다.

설정은 어떤 것도 변경하기 전에 모든 옵션을 검증합니다. 잘못된 앱 이름(비어있거나, `.`로 시작되거나, `/`를 포함하는 경우)은 거부됩니다.

## 세션 및 재로드

`SUPER + SHIFT + R`는 `argvus sessionctl reload`를 실행합니다. 세션 컨트롤러는 생성된 구성을 적용하고 자신이 소유한 구성 요소만 재시작합니다. 홈 디렉터리에 `/usr/share/argvus`를 복사하지 않으며, 잘못된 JSON 파일을 교체하지도 않습니다.

## 야간 조명

데스크탑 및 입력 > 디스플레이 > 나이트 라이트를 열거나 `argvus control-center displays night-light`를 실행하십시오. `argvus config display night-light`를 통해 색온도, 수동 활성화 및 고정 로컬 일정을 구성할 수 있습니다. 명령, 상태 및 실행 요구 사항에 대해서는 [나이트 라이트](/docs/user-guide/night-light/)]를 참조하십시오.

`argvus tui-terminal --frame TITLE FOOTER COMMAND [ARG...]`는 공유된 ARGVUS 터미널 프레임 안에서 인터랙티브 명령을 실행합니다. Ctrl+E를 눌러 프레임을 닫을 수 있으며, 호출자가 현지화된 푸터를 제공합니다. 이를 터미널 안에서 사용하거나 `argvus tui-terminal --term kitty -- ...`에 전달되는 명령으로 사용하십시오.

`argvus tui-terminal --term kitty --profile ai-agents`는 숨겨진 탭과 클립보드 단축키만 있는 전용 Kitty 구성을 사용합니다. 호출자가 명령과 제목을 제공하며, 창 배치는 `argvus-ai` 클래스에 대한 Hyprland 규칙에 의해 처리됩니다.
