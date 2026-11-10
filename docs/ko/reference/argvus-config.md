---
title: ARGVUS 구성
description: ARGVUS에서 사용하는 논리적 JSON 구성 계약.
---
ARGVUS는 관리되는 사용자 환경설정을 `$XDG_CONFIG_HOME/argvus/config/`(보통은 `~/.config/argvus/config/`) 아래의 모듈식 JSON 파일로 유지합니다. 이 파일들은 ARGVUS에서 관리하는 환경설정의 표준 논리 모델을 형성하며, `argvus-config`만이 `data/generated/` 아래의 파일을 작성하는 유일한 구성 요소입니다. 네이티브 파일과 생성된 파일은 호환성 투영이거나 애플리케이션별 재정의 상태로 남아 있습니다.

## 사용자 안내서: 빠른 시작

대부분의 사용자에게는 구성 설정이 ARGVUS 제어 센터 또는 이 명령줄 도구를 통해 이루어집니다:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```
관리자 및 고급 구성을 위해, 아래 내용을 계속 읽으세요.

사용자 root에는 오직 표준 문서와 관리되는 데이터만 포함되어 있습니다:

```text
~/.config/argvus/
├── config.json
└── data/
```
파일은 원자적으로 작성되며 사용자가 개인적으로 보관하고, 투영 중에 교체될 수 있는 data/ 트리 외부에서 `.config.lock`에 의해 보호됩니다. 이전 버전은  `data/` `data/backups/config.json.bak`에 저장됩니다.

## 명령줄

`argvus config` 하위 명령어는 `argvus` 패키지에 의해 프로세스 내에서 제공됩니다:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```
`ensure`는 스키마 기본값으로 새 `config.json`를 생성하거나, 명시적 값을 대체하지 않고 누락된 필드를 채웁니다. 잘못된 JSON에서는 실패합니다.
`migrate`는 기존 레거시 외형 상태를 가져오되 이미 `config.json`에 있는 값을 대체하지 않습니다. `argvus sessionctl`는 구성 변경 후 재로드를 적용하기 전에 `recover`, `ensure`, `migrate` 및 `project`를 실행합니다:

```sh
argvus sessionctl apply-config
```
`apply`와 `rebuild`는 `config.json`에서 모든 프로젝션을 재빌드하기 위한 별칭입니다; `project`는 기존 철자를 사용하며 `--force`를 허용합니다. `apply`는 또한 커밋 후 런타임 재로드를 `argvus sessionctl`에 위임합니다. 준비나 독립 테스트를 위해 `ARGVUS_NO_RUNTIME=1`를 설정합니다. 생성은 스테이징에서 이루어지며, 모든 프로젝터가 성공한 후에만 `data/` 트리가 교체됩니다. 따라서 `data/generated/`를 삭제하고 `argvus config apply`를 실행하는 것은 Hyprland 또는 Taskbar 출력의 부분 시퀀스를 게시하지 않고 재현 가능한 재빌드입니다. 명령은 파생 섹션 해시와 런타임 결정을 `$XDG_STATE_HOME/argvus/config-projection.json`(보통 `~/.local/state/argvus/config-projection.json`)에 기록합니다. 매니페스트나 생성된 파일은 진위 여부의 근원이 아닙니다. `apply`는 `argvus sessionctl`를 통해 커밋 후 재로드를 수행합니다; `project`는 호환성을 위해 프로젝션 전용으로 유지됩니다.

모든 지속적인 변이는 동일한 규칙을 사용합니다: 표준 문서가 변경된 경우, CLI는 커밋 후 하나의 중앙화된 `argvus sessionctl reload`를 호출합니다. 재로드 결과는 `data/internal/last-reload.json`에 기록됩니다.

따라서 `argvus sessionctl reload`는 *적용* 단계이지 투영 단계가 아닙니다. 들어오는 매니페스트가 보류 중인 작업을 보고하면 계획에 설명된 대상 소비자를 복원하며 `argvus config project`를 다시 호출하지 않습니다; 재투영은 동일한 트리를 재구축하고 변이가 방금 생성한 계획을 버리게 됩니다. 커밋된 계획이 존재하지 않을 때, 예를 들어 세션 시작 시,에만 마이그레이션과 투영으로 돌아갑니다.

### 재로드 대상

`argvus config session-plan`는 변경된 섹션의 대상과 함께 리로드에 응답하고, `argvus sessionctl`는 그것들만 실행합니다. 스플래시는 대상이 표면을 재생성할 때만 리로드를 포함합니다: Waybar 바 또는 화면으로 다시 이동하는 제어판은 모든 창을 이동시킵니다. 나머지는 모두 제자리에서 리로드됩니다.

| 변경 | 대상 | 스플래시 |
| --- | --- | --- |
| 테마, 강조 색상 또는 GTK 모드 | `hyprland`, `consumers`, `wallpaper`, `polkit` | 아니요 |
| 배경화면만 | `wallpaper` (hyprpaper IPC; swaybg만 서비스를 재시작) | 아니요 |
| `effects` (블러, 애니메이션, 투명도, 그림자) | `hyprland`, `consumers` | 아니요 |
| `layout`의 창들 (간격, 테두리) | `hyprland` | 아니요 |
| `layout.taskbar` (위치, 여백) | `hyprland`, `taskbar`, `widget_telemetry` | 예 |
| `fonts` | `consumers` | 아니요 |
| `control_panel` (전환, 카드) | `control_panel` | 예 |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | 아니요 |
| `power` | `hypridle` | 아니요 |

`consumers`는 `argvus config theme-runtime consumers`를 실행합니다 (바 스타일시트, IPC를 통한 제어판, Hyprland 색상, Dunst 테마 재로드, btop 및 kitty). Dunst는 방해 금지 모드가 활성화되어 있는 동안 테마 재로드를 연기합니다. `wallpaper`는 `argvus config theme-runtime wallpaper`를 실행하며, 이로써 유효한 `/appearance/wallpaper`를 표시합니다. `layout.taskbar` 해시는 프로젝션 매니페스트에서 창 레이아웃과 별도로 유지되므로, 간격 변경이 바를 재시작하지 않습니다. 커밋된 계획이 없을 때 사용되는 대체 `full` 계획은 스플래시 아래의 모든 표면을 재시작합니다.

각 유효한 구성에는 운영 세대가 있습니다. 이는 `data/internal/generation.json` 및 `$XDG_STATE_HOME/argvus/config-projection.json`에 있는 프로젝션 매니페스트에 저장되며, 도트파일(dotfiles) 구성의 일부는 아닙니다. 변경되지 않은 유효한 구성을 다시 적용하면 세대가 유지됩니다. `status`는 매니페스트 해시, 세대 메타데이터 및 모든 생성된 파일이 일치하는지 여부를 보고하며, `apply`는 오래되었거나 불완전한 프로젝션을 복구합니다.

`explain /path`는 리졸버(resolver)를 직접 읽고 유효한 값, 소스, 테마 및 변형을 출력합니다. 이는 생성된 파일을 파싱하거나 두 번째 우선 규칙을 생성하지 않습니다.

테마와 효과 투영은 `argvus-config` 자체에서 구현됩니다.  
테마 거래(`argvus theme-switch`, `theme-apply`를 실행), `argvus config accent` 및 `argvus effects-toggle`는 투영하지 않습니다: 이들은 이 CLI를 통해 유효성을 검사하고, 잠그고, 커밋한 후 `config.json`를 읽을 수 없는 외부 소비자를 조정합니다. `argvus effects-toggle`는 자체 출력이 없는 얇은 대리자입니다. 표현 도우미는 투영기에서 결코 호출되지 않으며, `data/generated/` 아래에 절대 작성해서는 안 됩니다.  

### 투영 표면  

`project_effective`는 게시 전에 모든 ARGVUS 소유 소비자를 하나의 스테이징 트리에서 렌더링합니다:  

| 표면 | 출력 |  
| --- | --- |  
| Hyprland | 테마 프로필을 포함한 `generated/hypr/`, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` 및 `monitors.lua` |  
| 레이아웃/효과 | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Waybar | `generated/waybar/`, `argvus-taskbar.css` 및 `argvus-widget-telemetry.{jsonc,css}` 포함 |
| Widget 텔레메트리 테마 | `generated/widget-telemetry/` 및 `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| 알림 | `generated/notifications/` 및 `dunstrc` |
| 잠금 화면 | `generated/hypr/hyprlock.conf` 및 Hyprtoolkit 테마 프로필 |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Superfile | `generated/superfile/` |
| 터미널 및 Qt 프로필 | `generated/terminal/`, `generated/qt6ct/` |
| 글꼴 | `generated/fonts.conf` 및 Waybar 스타일시트에서 관리되는 글꼴 블록 |
| 이동식 장치 | `generated/removable-devices/theme.css` |
| 배경화면 | `generated/hypr/hyprpaper.conf` 및 `data/hypr/hyprpaper.conf` |
| 제어판 및 기본값 | `data/control-panel/`, `data/control-center/defaults.json` |
| 캘린더, 입력, 키보드, 전원 | 캘린더 TOML, 입력 프래그먼트, 키보드 상태, 깨어있기 표시기 |

테마 레이어는 생성된 루트 외부에서도 반영됩니다:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` 및
`data/hypr/application-style.conf`. 외관 섹션이 변경되면 이러한 복사본은 패키지된 기본값으로 대체되며, 프로젝터는 그 후에 자신이 관리하는 블록만 다시 작성합니다 — Waybar 스타일시트의 폰트 블록과 
텔레메트리 프로파일의 `ARGVUS_TELEMETRY_*` 블록 — 따라서 표시기 밖의 텍스트는 테마 전환 후 유지되지 않습니다. `fonts.targets`와
`control_panel.widget_telemetry_blocks`는 표준으로 유지됩니다: 제어 센터는
`argvus-config`를 통해 이를 지속하며 이러한 파일을 스스로 작성하지 않습니다.

위젯 원격 측정 블록 선택은 `/control_panel/widget_telemetry_blocks`에서 표준이며, 7개의 블록 이름 배열(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`)로 구성됩니다. 이전의 `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` 파일은 마이그레이션 백업 용도로 읽기 전용이며, 절대 쓰이지 않습니다. ` and ` and blocks all 명령은 단일 원자적 쓰기로 표준 배열을 수정하며, `apply-state`는 선택이 이미 호출자에 의해 저장되었다고 가정합니다.

Hyprland 간격 및 테두리 설정도 동일한 경계를 따릅니다. argvus config spaces 및 argvus config borders 명령(및 그 과거 호환성 래퍼)은 /layout을 업데이트하며, Rust는 data/generated/spaces-effective.conf와 data/generated/borders-effective.conf를 생성하고 검증과 실시간 hyprctl 적용을 담당합니다. .spaces와 .borders는 마이그레이션 전용 상태이며, 절대 진실의 출처가 아닙니다.

악센트와 GTK 모드 상태는 동일한 Rust 경계를 사용합니다. 악센트 소유권에는 ` or ` or argvus config accent --theme-default를 사용하고, GTK 모드에는 `argvus config mode toggle|set dark|light`를 사용하세요. 기존 악센트 및 모드 스크립트는 호환성 래퍼입니다; `.accent-color`, `.accent-custom` 및 `.gtk-mode`는 마이그레이션/프로젝션 파일일 뿐입니다.

논리적 ARGVUS 경로는 `argvus config paths`에 의해서도 해결됩니다. 레거시 `paths.sh` 파일은 호환성 함수만 노출하며, 네이티브 오버라이드, 사용자 데이터, 생성된 데이터 및 시스템 폴백은 Rust 경로 모델에서 해결됩니다.

`audio`는 기본 출력 싱크를 변경하고 결과를 `/audio`에 한 단계로 기록합니다: `argvus config audio volume <N|+N|-N> [--limit PERCENT]`와 `argvus config audio mute <on|off|toggle>`. PipeWire가 이미 값을 가지고 있기 때문에 구성을 다시 로드하지 않고 스토어를 씁니다. Waybar 볼륨 모듈, Hyprland 미디어 키 및 제어판에서 이를 사용하므로 `argvus config project`, 싱크에 `/audio`를 투영하는 기능은 이전 볼륨을 복원하지 않습니다. 100%를 초과하는 볼륨(Hyprland 키는 최대 150%를 허용)은 100으로 저장됩니다.

`patch`는 키가 JSON 포인터인 JSON 객체를 하나의 잠긴 읽기-수정-쓰기 작업으로 적용합니다. `apply-theme`는 동등한 원자적 테마 트랜잭션을 수행하고 테마의 선택적 모양 프로필이나 패키지된 기본값에서 관리되는 모양 섹션을 교체합니다. 변형 명령은 `NextUserConfig`를 구축하고, `EffectiveConfig`를 해결하며, 모든 프로젝터를 준비하고, 그 다음에만 정규 JSON을 게시합니다. 운영 `.config-transaction.json` 마커는 데이터와 JSON 게시 사이에 충돌이 발생한 후 이전 구성을 `argvus config recover`가 다시 투사할 수 있게 합니다. 테마 기본값은 각 테마의 매니페스트(`themes/<theme>/theme.toml`)에서 선언됩니다: 기본 강조(`palette.accent`), GTK 색 구성표, 그리고 배경화면. CSS, Lua, Rasi 및 Qt 파일은 테마 빌드로 렌더링됩니다.

라이트/다크를 포함한 테마를 변경하면 사용자가 사용자 지정 색상을 선택했더라도 하이라이트 색상이 테마의 기본값으로 재설정됩니다. 사용자 지정 색상은 다음 테마 변경 전까지 유지됩니다. 테마 변경 후 `argvus config accent`를 사용하여 다시 설정하십시오. `argvus config theme-apply`로 테마를 적용하면 매니페스트에서 `/appearance/accent` 및 `/appearance/gtk_mode`가 기록되며, `argvus config accent`는 `accent_custom`가 설정된 상태에서 선택한 색상을 한 명령으로 기록합니다.

`apply-theme`는 변형을 명시적으로 `--variant sticky|spaced`로 가져오고, 요청된 이름을 매니페스트를 통해 해결하므로 요청이 다른 변형의 에셋으로 해결되는 일은 없습니다. `theme-manifest THEME`은 동일한 해결을 읽기 전용으로 노출합니다: 매니페스트 항목을 `asset_id`와 `layout_variant`와 함께 출력하며, 이는 호출자가 패키지된 에셋을 처리하는 데 필요한 것입니다. Sticky는 접미사가 없는 ID이고 Spaced만 `-spaced` 접미사를 가지므로, `dracula-sticky`와 `argvus-dark-dracula-sticky`가 허용되며 `dracula`로 정규화됩니다.

`--raw`을 사용하면 문자열 값은 JSON 따옴표 없이 출력되는 반면, 객체와 배열은 JSON 값으로 유지됩니다. 테마 및 레이아웃 도트 파일은 더 이상 프로젝션 헬퍼에 의해 작성되지 않으며, `config.json`만 해당 설정의 유일한 지속적 소스로 남습니다. 프로젝션에 실패할 경우, 세션 재로드는 데스크톱 서페이스를 재시작하기 전에 중지되므로 이전의 부분적인 테마가 현재 테마로 표시되지 않습니다.

## 내보내기 및 가져오기

프로필 교환에는 동일한 논리 모델이 사용됩니다:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```
전체 관리 데스크탑 모델을 위해 `--scope desktop`을 사용하십시오. 누락된 필드는 ARGVUS 기본값을 사용하며, 생성된 출력은 직접 편집하지 마십시오.

컨트롤 센터 테마 내보내기 및 가져오기는 테마와 그 시각적 프로필을 교환합니다: 아카이브는 $XDG_DATA_HOME/argvus/themes/ 아래에 설치되는 스키마 2 테마 디렉토리(`<id>/theme.toml`, 선택적 `appearance.json` 및 `effects.conf`, `overrides/`, 배경화면)를 포함합니다. 프로필은 외관, 레이아웃 지오메트리, 시각 효과, 글꼴 및 패널 표시를 보관합니다. 오디오, 키보드, 네트워크 및 기타 비시각적 환경설정은 테마 외부에 남아 있습니다. 전체 외관 구성을 이동하려면 위의  `$XDG_DATA_HOME/argvus/themes/` `appearance` 범위를 사용하고, 전체 구성 백업을 위해 `desktop` 범위를 사용하십시오.

현재 스키마는 `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` 및 `removable_devices` 아래에 환경설정을 그룹화합니다. `default_apps`는 선호 애플리케이션의 유일한 지속적인 소스이고, `keyboard_shortcuts`는 문자열 값이나 비활성 바인딩의 경우 `null`와 함께 안정적인 영어 및 로케일 독립 키를 저장합니다.

컨트롤 센터는 Appearance 페이지를 열 때 표준 외관 값을 읽고 `argvus-config`를 통해 변경 사항을 씁니다. 기존의 네이티브 상태 파일은 외관 스크립트와 이전 설치를 위한 투영으로 남아 있습니다.

제어판은 카드 순서와 가시성을 위해 `control_panel.cards`를 사용하고, 패널 마스터 스위치에는 `control_panel.enabled`를 사용합니다. `control-panel/cards.json` 및 `state/control-panel` 파일은 호환성 투영으로 보관됩니다. 따라서 제어판에서 이루어진 변경 사항은 `desktop` 내보내기 범위의 일부가 됩니다.

글꼴 환경 설정은 작업 표시줄, 원격 측정, 제어판, 시스템, 애플리케이션, 터미널 및 브라우저 대상의 글꼴 패밀리, 스타일 및 크기에 대해 `fonts.targets`를 사용합니다. `fonts.rendering`는 안티앨리어싱, 힌팅, 서브픽셀 순서 및 DPI 환경 설정을 저장합니다. `fonts.conf`, 즉 fontconfig 규칙, 데스크톱 설정 및 Waybar 스타일시트 내 관리되는 글꼴 블록은 투영입니다: 제어 센터는 `/fonts/targets/*`에서 `argvus-config`까지 유지하며, 프로젝터는 자신의 구분된 블록만 재작성합니다.

Hyprland 환경설정은 단축키에 `keyboard_shortcuts`, 마우스 및 터치패드 설정에 `hyprland.input`, XKB 레이아웃, 변형 및 옵션에 `hyprland.keyboard`을 사용합니다. 매니페스트 바인딩 ID는 안정적인 `config_key`에 매핑됩니다(예: `window.close` → `close_window` 및 `window.drag_mouse` → `drag_window__floating_window_only`). 디스플레이 환경설정은 `displays.monitors`, `displays.workspaces`, `displays.primary_monitor`을 사용합니다. `data/generated/` 아래의 네이티브 TOML 파일과 생성된 Lua 파일은 Hyprland에서 사용하는 투영으로 남아 있습니다.

모든 `effects` 스위치는 가상 머신과 저전력 세션에서도 기본적으로 활성화되어 있습니다(애니메이션, 블러, 위젯 텔레메트리 및 각 투명 표면). 창 그림자(`shadow_enabled`)와 흔들림(`wobble_enabled`)은 기본적으로 비활성화되어 있습니다. 효과 환경설정은 애니메이션 마스터 스위치(`effects`, `animations`, 투영된 hypr/state.lua에서 `hyprland.lua`에 적용, 효과 스위치와 독립적이며, 효과 스위치는  `hypr/state.lua` `blur_global_enabled` 및  `transparency_<surface>_enabled` `taskbar`, `control-panel`, `widget-telemetry`, `terminal`, `launchers`의 transparency_<surface>_enabled와 함께 구성되어 있으며, 이 중 하나라도 활성화되면 활성으로 간주; `argvus effects-toggle animations|effects` 스위치로 변경), 글로벌 블러 스위치(`blur_global_enabled`), Hyprland 블러 매개변수(`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), 창 그림자(`shadow_enabled`, `shadow_sharp`, `shadow_color`) `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), 애니메이션 값 (각 애니메이션에 대해 `animation_<leaf>_speed`, `_curve`, `_style`, `_popin` 및 `_enabled`, 커스텀 곡선에 대해 `animation_bezier_x1`에서 `animation_bezier_y2` 및 `animation_spring_mass`, `animation_spring_stiffness` 및 `animation_spring_dampening`) 및 표면별 투명도 값. 효과는 테마 소유이며, 변형 소유가 아님: Sticky와 Spaced는 동일한 `<theme>.conf` 효과 투영을 사용하지만 레이아웃 투영은 다름. 현재 표면에는 `taskbar`, `control-panel`, `widget-telemetry`이 포함됨; 투명도 값은 `0`에서 `100`까지의 백분율 (컨트롤 센터와 런처 표면은 기본적으로 `15`, 나머지는 `50`), 블러 매개변수는 `schema.json`에서 선언된 범위 내의 숫자이며, 활성 상태는 불리언. 애니메이션 값은 `argvus-config`에 의해 검증됨: 속도는 `1`부터 `10`까지의 숫자(소수 허용), 팝인 백분율은 `1`부터 `100`까지의 정수이고, 곡선은 애니메이션 페이지의 이름이어야 하며, 스타일은 애니메이션 페이지의 이름이어야 하며 애니메이션 자체 리프에서 허용되어야 합니다(예: `popin`는 `animation_workspace_in_style`에 대해 거부되고, `gnomed`는 창 열기 및 닫기에만 허용되며 슬라이드 변형은 창 및 레이어 리프에서 거부됩니다. 베지어 Y 점은 `-1`와 `2`, 범위 Hyprland 0.56.2 사이에 유지됩니다. 구문 분석되고 다른 곡선 매개변수는 문서화된 범위 내에 유지됩니다. `layers` 및 `switching` 잎은 더 이상 존재하지 않으므로 제거하기 전에 저장된 구성은 로드 시 `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` 및 `animation_switching_*` 키를 잃게 됩니다. 현재 세트 외부의 스타일은 삭제되고 `-1`~`2` 외부의 베지어 Y 지점은 고정되므로 패키지된 기본값이 다음에 다시 적용됩니다. 제거된 모든 것.

전원 환경 설정은 `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` 및 덮개/전원 버튼 정책을 사용합니다. 예상되는 형식은 `data/generated/hypr/hypridle.conf`이며 마커는 `data/power/keep-awake`입니다. `.keep-awake`는 마이그레이션 입력 전용입니다. `session.language`는 사용 중지되었습니다: 기존 값은 무시되며, 새 쓰기는 거부되고, 언어 호환성 파일이 예상되지 않습니다. 대신 사용자 locale.conf에서 `LANG`를 설정하십시오. 달력 환경 설정은  `locale.conf` `calendar`를 사용하며 작업 표시줄 달력 사용자 TOML로 예상됩니다. 이동식 장치 환경 설정은 `removable_devices`를 사용하며, 이동식 장치 응용 프로그램은 이 섹션을 패키지 기본값보다 사용자가 재정의한 것으로 처리합니다.

`migrate`은 기존 표준 값을 보존하면서 이전 루트 파일과 섹션을 한 번 가져옵니다. 레거시 `defaults.json`, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, 점 파일(dot-files), `generated/` 및 구성 요소 디렉터리는 `data/` 아래로 이동됩니다. 반복된 재로드는 이전 루트 레이아웃을 다시 생성하지 않습니다. 현재 테마를 다시 선택하는 것은 멱등성입니다: 표준 상태와 투영이 이미 일치하면, 투영 계획은 변경된 섹션이 없음을 보고하며 세션은 시작 화면과 서비스 재시작을 건너뜁니다.

전력은 동일한 투영 계약을 따릅니다: 제어 센터는 `/power`을 `config.json`에 기록하고, `argvus config project`은 `data/hypr/hypridle.conf`을 원자적으로 렌더링하며, 그 다음에만 `argvus sessionctl`이 `argvus-hypridle.service`을 재시작할 수 있습니다. 해당 파일을 제거하는 것은 `argvus config project`으로 복구할 수 있습니다; `0` 값은 해당 타임아웃을 생략하며 Never를 의미합니다.

잠금 화면 외관은 Rust에 의해 정규 테마, 강조색, 효과, 배경화면 및 글꼴 상태에서 `data/hypr/hyprlock.conf`로 투영됩니다. 과거의 `hyprlock-theme.sh` 진입점은 호환성 전용이며 `argvus config lock apply`로 위임됩니다; `--invalidate`는 파생된 흐림 처리된 배경화면 캐시만 무효화합니다.

전원 타임아웃 변경은 `/power/lock_minutes`와 `/power/screen_off_minutes`를 진실 소스로 사용합니다. `argvus config project`는 기존 Hypridle 렌더러를 재사용하며, `idle-timeout.sh` 진입점은 단지 호환성 어댑터일 뿐입니다; 서비스 재시작은 계속해서 `argvus sessionctl`에 위임됩니다.

알림 외관은 정규 테마, 강조색 및 효과에서 `data/notifications/dunstrc`로 파생됩니다. 과거 알림 `theme.sh` 파일은 `argvus config notifications`를 호출하는 호환성 기능만 노출하며, Dunst 구성을 분석하거나 변형하지 않습니다.

디스플레이 탐색 및 블루투스 제어와 같은 하드웨어 중심 어댑터는 이제 Rust 디스플레이 도메인에서 가져옵니다: `nwg-displays`는 외부 가져오기이며, `/displays`와 `data/generated/hypr/monitors.lua`는 각각 정규 및 파생 상태로 유지됩니다. 디스플레이 도구는 `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces`, `monitor-remove`와 함께 레이아웃을 변경합니다; 디스플레이 명령은 `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`입니다. Hyprland 파일은 `/displays`에서 투사되며, 더 이상 어떤 셸 스크립트도 이를 작성하지 않습니다. 배경화면 선택, 작업 표시줄 유틸리티 그룹 모드, 기본 앱 조회 또한 `argvus-config` API를 사용하며; 이들의 과거 Shell 진입점은 UI 또는 호환성 래퍼입니다. Keep Awake는 `/power/keep_awake` 지속 설정으로 `data/power/keep-awake` 마커에 투사되며, lock-DPMS는 세션 런타임 상태입니다.
`$XDG_CACHE_HOME/argvus/lock-dpms`. 키보드 레이아웃은 이제 Rust에서 소유합니다.  
`argvus-keyboard-layout` 헬퍼: 구성된 순서는 생성된 Hyprland 입력 프로젝션이나 로케일 폴백에서 나오며, 활성 레이아웃은 Hyprland에서 발견된 세션 런타임 상태로 유지됩니다. 사이클 진입점과 키보드 레이아웃 데몬은 호환성 래퍼일 뿐이며; 사용자 서비스는 Rust 데몬을 직접 실행하고 `argvus sessionctl keyboard-layout cycle`가 호출자를 위한 안정적인 명령입니다.

블루투스 런타임 상태는 외부 어댑터 경계를 따르지만 `argvus-config`의 일부는 아닙니다: `argvus bluetooth`는 `bluetoothctl` 주위의 타입화된 파싱 및 검증, 제한된 재시도, 시스템 서비스 확인 및 RFKill 상태를 담당합니다. `argvus bluetoothctl`가 진입점입니다. 어댑터 전원, 페어링된 장치 및 연결된 장치는 표준 JSON 상태보다는 BlueZ 런타임 상태로 남아 있으며, 하드웨어 기반 블루투스 검증은 환경에 따라 달라집니다.  

## 런타임 식별 및 조정

정확한 바이너리, 빌드 식별, 구성 루트, 생성된 루트, 자산 루트, 매니페스트 소스 및 활성 세대를 검사하려면 `argvus config status --verbose`를 사용하십시오. 이는 `synchronized`, `stale`, `incomplete`, `legacy` 및 `pending-transaction` 투영 상태를 보고합니다.

`argvus config --version`는 빌드 환경에서 제공될 때 패키지 버전, 빌드 수정 사항 및 프로파일을 보고합니다. 개발 테스트는 바이너리와 에셋을 명시적으로 주입해야 하며, `argvus config`나 `~/.config/argvus`를 조용히 사용해서는 안 됩니다. `apply`는 `argvus-dark-spaced`와 같은 기존 결합 식별자를 재구성하기 전에 테마와 변형 값을 안전하게 분리하여 정규화합니다. 이 릴리스에서는 패키지된 에셋의 아이덴티티가 매니페스트에 기록되지 않습니다.

## 나이트 라이트

**Desktop & Input > Displays > Night Light**를 열거나 `argvus control-center displays night-light`를 실행하세요. `argvus config display night-light`를 통해 온도, 수동 활성화 및 고정된 지역 스케줄을 설정할 수 있습니다. 명령, 상태 및 런타임 요구사항에 대해서는 [Night Light](/docs/user-guide/night-light/)]를 참조하세요.

### 흔들림 설정 (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`, `wobble_intensity` 및 `wobble_mesh` 프로젝트는 `hypr/state.lua`로 직접 연결됩니다. `argvus effects-toggle wobble-settings get`를 사용하여 이를 읽거나, `argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`로 저장하고 재로드를 요청하세요. `enabled=disabled`는 효과를 끕니다.

컨트롤 센터는 애니메이션 초안을 공유합니다: `animations-settings get` 또한 `wobble_*` 항목을 반환하며, `animations-settings set`는 기존 애니메이션 인수와 함께 이 전체 이름을 수락합니다. 이는 전체 초안을 하나의 검증된 패치로 유지하고 한 번의 재로드를 요청합니다. 예를 들어:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```
전역 `enabled`는 애니메이션을 제어합니다; `wobble_enabled`는 그 독립적인 환경설정입니다. 전역 애니메이션을 비활성화하면 Lua에서 흔들림이 억제되지만, 해당 환경설정은 삭제되지 않습니다. 잘못된 값은 전체 패치를 거부합니다. 누락된 키는 기존 선택을 대체하지 않고 ARGVUS 기본값을 받습니다. 범위와 기본값은 [Appearance](/docs/user-guide/appearance/#window-wobble)]을 참조하세요.

### 사용자가 만든 키보드 단축키

`/keyboard_shortcuts`는 공식 매니페스트 작업을 위한 기존 문자열 또는 널 값을 유지합니다. 사용자 정의 전역 키보드 단축키는 별도의 `/hyprland/custom_shortcuts` 객체를 사용하며, 이는 `config/hyprland.json`에 저장됩니다. 안정적인 `custom_` ID는 설명과 로케일과 독립적입니다. 각 레코드에는 `description`, `keys`, `enabled` 및 유형화된 `action`가 있습니다:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```
예시는 `/hyprland/custom_shortcuts`의 값일 뿐, 완전한 구성 문서는 아닙니다. 디스패처 이름은 지원되는 Lua 디스패처 팩토리에서 가져오며; 인수는 선택적인 JSON 객체 또는 `null`로, 절대 평가되는 코드는 아닙니다. 커스텀 바로가기는 하위맵, 마우스 바인딩 또는 고급 플래그가 없습니다. 투영은 마지막 레코드가 제거될 때 빈 테이블을 포함하여 `data/generated/hypr/custom-shortcuts.lua`를 이스케이프된 데이터 테이블로 작성합니다. 전체 투영과 `project --only keyboard_shortcuts` 모두 이를 생성합니다. Hyprland는 활성화된 항목을 `hl.bind`로 등록하며; 선택적 파일 및 개별 등록 실패는 포함되어 기록됩니다.

컨트롤 센터는 하나의 `argvus config patch` 트랜잭션을 사용하여 레코드를 생성/편집하고 확인된 충돌을 비활성화합니다. 기존 구성 서비스를 통해 적용하기 전에 저장합니다. 적용 실패는 성공적인 저장을 되돌리지 않습니다. 공식 기본값을 복원하면 커스텀 레코드는 유지되며, 활성 커스텀 단축키와 충돌할 경우 거부됩니다. 사용자 설명은 모든 로케일과 유효 생성된 치트시트에서 그대로 텍스트로 유지됩니다.

## 전역 창 모드

**Control Center → Desktop & Input → Hyprland → Windows → Mode**를 열고 **Tile Windows (Default)** 또는 **Floating Windows**를 선택합니다. 동일한 선택기는 작업 표시줄 유틸리티 그룹의 창 모드 아이콘에서도 사용할 수 있습니다.

이 설정은 모든 모니터와 작업 공간에 적용되며 로그아웃 후에도 기억됩니다. 플로팅(Floating)은 기존 애플리케이션 창과 새로 열린 창을 변환합니다. 타일(Tile)은 이 정책으로 변환된 창만 복원하며, 이미 플로팅 중인 대화 상자나 다른 창은 그대로 플로팅 상태를 유지합니다. 플로팅이 활성화된 동안 명시적으로 고정된 창도 고정을 유지하기 위해 동일하게 플로팅 상태로 유지됩니다. 타일 레이아웃은 다시 계산되며, 이전의 정확한 분할 트리는 복원되지 않습니다.

`SUPER + SHIFT + Space`는 타일 내에서 일반적인 창별 동작을 유지합니다. 플로팅에서는 창을 다시 타일로 전환하지 않습니다. 패널, 작업 표시줄 및 알림은 레이어 셸(surface)이며 영향을 받지 않습니다. 전체 화면 및 최대화는 계속해서 Hyprland의 일반 동작을 사용합니다.

표준 기본 설정은 `/hyprland/window_mode`이며, 위치는 `$XDG_CONFIG_HOME/argvus/config/hyprland.json`에 있습니다. 생성된 투영은 `data/generated/hypr/window-mode.lua`이며, 수정하지 마십시오. 창별 복원 레지스트리는 `$XDG_RUNTIME_DIR/argvus` 아래에 있으며, 컴포지터 인스턴스에 범위가 지정되고 구성 다시 로드 시에도 유지됩니다. 새 세션에서는 재사용되지 않습니다.

애플리케이션이 대기 중이거나 실패하면, 인터페이스는 이를 저장된 기본 설정과 구별합니다. 옵션을 다시 선택하여 재시도하십시오. 컴포지터가 없으면 기본 설정은 다음 로그인에 저장됩니다.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```
## 창 버튼과 애플리케이션 바

플로팅 모드는 일반 애플리케이션 창에 **최소화**, **최대화 / 복원**, **닫기**가 있는 ARGVUS 제목 표시줄을 추가합니다. 제목 표시줄을 더블 클릭하면 최대화가 전환됩니다. 추가 장식은 타일 모드와 전체 화면 모드에서는 숨겨집니다. 자체 컨트롤을 그리는 애플리케이션은 두 개의 바를 모두 표시할 수 있으며, 통일된 ARGVUS 컨트롤은 의도된 것입니다.

두 번째 Waybar는 모든 모니터에서 나타나며, 기본 작업 표시줄과 반대쪽 가장자리에 위치합니다. 이 바에는 해당 모니터의 현재 작업 공간에 속한 창만 나열되며, 그 작업 공간에서 최소화된 창도 포함됩니다. 열려 있는 특별 작업 공간이 우선합니다. 고정된 창은 계속 표시되며, 숨겨진 그룹 탭은 추가 항목을 만들지 않고, 최소화된 그룹은 하나의 복원 항목을 갖습니다. 애플리케이션 내 탭은 별도로 나열되지 않습니다. 최소화된 항목은 원래 모니터/작업 공간 연결을 유지합니다. 비활성 창을 클릭하면 활성화되고, 활성 창을 클릭하면 최소화되며, 최소화된 창을 클릭하면 복원됩니다; 해당 컨텍스트 메뉴에는 복원, 최소화, 최대화/복원 및 닫기가 제공됩니다. 알 수 없는 애플리케이션 아이콘은 창 기호로 대체됩니다. 항목을 활성화하거나 복원하면 포인터 위치가 유지됩니다. 보조 바는 기본 CSS(색상, 글꼴, 테두리 및 둥글기)를 적용하며, 높이와 반대쪽 가장자리에 대칭된 여백을 가집니다.

`SUPER + Alt + Up`는 메인 바를 상단에, 창 바를 하단에 배치합니다. `SUPER + Alt + Down`는 그 반대로 배치합니다. 방향을 반복하면 위치가 변경되지 않습니다. 타일에는 메인 바만 있습니다. 타일로 돌아가면 창 바를 제거하기 전에 최소화된 모든 창이 복원되고, 이후 원래의 타일/플로팅 정책이 유지됩니다.

최소화는 예약된 `special:argvus-minimized` 작업 공간을 사용합니다. 해당 작업 공간을 수동으로 사용하지 마십시오. `argvus windows service` 프로세스는 컴포지터 인스턴스별 런타임 레지스트리의 `$XDG_RUNTIME_DIR/argvus` 아래에 신원, 원래 작업 공간/모니터, 기하학, 전체 화면/최대화 상태 및 고정 상태를 기록합니다. 기록은 서비스 재시작이나 설정 다시 불러오기 후에도 남아 있지만, 컴포지터 세션 간에는 공유되지 않습니다. 창을 닫으면 해당 기록이 삭제됩니다. 모니터가 사라지면 복원은 사용 가능한 출력으로 수행됩니다. 복구 바가 없거나 실패하면 새로운 최소화가 방지됩니다; 복원 실패 시 기록과 복구 바는 유지됩니다.

세션은 `argvus-windows.service`와 `argvus-window-bar.service`를 소유합니다. 후자는 그 Waybar 자식을 Tile 상태에서 비활성으로 유지합니다. hyprbars 플러그인은 `argvus-hyprland`에 의해 번들로 제공되며, x86_64 아키텍처에서 Hyprland 0.57.0 ABI용으로 빌드되었습니다; 사용자의 홈 디렉터리에서는 컴파일되지 않았습니다. 컴포지터 ABI 업데이트 시 플러그인을 재빌드하십시오.명령어: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all`, `minimize|restore|activate|maximize|close ADDRESS`. 창 주소는 리스트에서 가져옵니다. `position top --persist`는 메인 바를 재시작하지 않고도 환경설정을 저장/프로젝트합니다. 구성 및 런타임 오류 세부 사항은 저널에 기록되며, 눈에 보이는 동작 실패는 번역된 알림으로 표시됩니다.

**Spaced + Floating** 모드에서, 창은 외부 간격과 두 바가 예약한 영역을 존중합니다. 경계에는 테두리와 제목이 포함되며, 제목은 같은 창 테두리 안에 위치합니다. ARGVUS 제어를 통해 최대화하면 이 영역이 여백과 함께 채워지고, 복원하면 이전 지오메트리를 복구하며, 서비스 재시작 이후에도 마찬가지입니다. 전체 화면은 여전히 전체 출력 영역을 사용합니다. 플로팅 창은 겹칠 수 있으며, 타일링 내부 간격은 창을 분리하지 않습니다.

창 막대에 의해 예약된 내부 여백은 창 외부 간격에서 두 번 계산되지 않습니다. 대칭 간격에서 눈에 보이는 창과 막대 사이의 거리는 위쪽과 아래쪽이 동일하며, 막대를 교체한 후에도 마찬가지입니다.

`SUPER + Alt + Space`는 작업 표시줄과 제어 센터와 동일한 환경 설정을 사용하여 전역 타일/플로팅 모드를 전환합니다. 동등한 명령은 `argvus config window-mode toggle`입니다. 동시 호출은 직렬화되며; 전환은 애플리케이션이 대기 중일 때에도 저장된 환경 설정을 사용합니다. 단축키는 키보드 단축키에서 변경하거나 비활성화할 수 있으며; `SUPER + Shift + Space`는 개별 창 동작을 유지합니다.
