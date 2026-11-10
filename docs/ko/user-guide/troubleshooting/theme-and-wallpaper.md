---
title: 테마 및 배경화면 문제 해결
description: 테마 전파 및 생성된 외관 파일을 진단하십시오.
---
$XDG_CONFIG_HOME/argvus/config.json에서 정식 `appearance` 섹션을 확인한 후,  `$XDG_CONFIG_HOME/argvus/config.json` `$XDG_CONFIG_HOME/argvus/data/generated` 아래에서 생성된 파일을 검사하십시오. 기존 도트 파일은 마이그레이션 입력 전용입니다.

data/generated/ 아래에서 유일한 작성자는 `argvus-config`입니다. 손상된 테마나 배경화면을 복구하려면, 정식 문서에서 전체 생성을 다시 빌드하고 생성된 소비자를 직접 편집하는 대신 그 결과를 적용하십시오:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```
 `data/generated/` `argvus theme-switch` (`argvus config theme-apply`를 실행함) 및 `argvus config accent`는 오케스트레이터입니다: 이들은 유효성을 검사하고, 잠금을 수행하며, 스플래시를 표시하고, `argvus-config`를 통해 커밋한 다음 `config.json`를 읽을 수 없는 외부 소비자를 조정합니다. 이들은 `data/generated/`를 절대 작성하지 않으며, 테마 트랜잭션은 별도의 스크립트를 통해 GTK, 스토리지 또는 `mode.css` 파일을 작성하지 않습니다. `argvus config accent --apply`를 사용하여 액센트 조정만 다시 실행하십시오.

공식 테마 변경은 저장된 외관 값을 적용하고 영향을 받는 런타임 소비자들을 업데이트합니다. 사용자 지정 테마를 적용하면 작업 표시줄, Hyprland 재로드, 바탕 화면, 제어판 서비스가 재시작됩니다. Sticky 또는 Spaced를 전환하면 해당 모드의 간격, 테두리 및 작업 표시줄 여백이 다시 적용되고, 그 후 작업 표시줄, 텔레메트리 및 Hyprland 재로드 서비스가 재시작됩니다. DND(방해 금지) 모드가 활성화된 동안 Dunst의 알림 색상은 예외입니다. 이 경우 색상 재로드는 DND가 꺼질 때까지 연기됩니다. 다른 표면에서 이전 색상이 유지된다면, 테마 빌드가 활성 상태인지, 선택한 테마인지 확인한 후 재로드를 다시 실행하십시오:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```
모든 테마 파일은 빌드에서 나옵니다: 설치되지 않은 테마( `theme.toml` `/usr/share/argvus/themes/` 또는 `~/.local/share/argvus/themes/` 아래에 theme.toml이 없음) 또는 렌더링되지 않는 테마는 거부되며, 이전 테마가 유지됩니다. `argvus theme check DIRECTORY`는 각 렌더 오류의 템플릿과 줄을 보여줍니다. Waybar 바는 JSONC의 `reload_style_on_change`이 있어야만 색상을 다시 불러옵니다. 스위치가 멈춘 것처럼 보이면, 여전히 실행 중인지 확인하세요; 자체적으로 `theme-switch.active`에 기록되며, 잠금 옆에 있습니다:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```
마커에 아직 살아 있는 pid가 있다면 스위치가 완료되지 않은 것입니다. 스위치는 테마 잠금과 프로젝션을 위해 최대 60초, 리로드 명령을 위해 5초 동안 대기하며; 프로세스가 종료된 마커는 오래된 것으로 간주되어 다음 스위치가 이를 무시합니다. 문제를 보고할 때 위 명령어들의 출력 결과를 포함하세요.

사전 로그인 그리터가 흰색이거나 대체 테마를 사용하는 경우에는, 개인 소스와 공개 프로젝션을 비교하세요:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```
인사 설정을 변경하지 않고 투영을 수리하십시오:

```sh
sudo argvus greeter-setup --sync-themes
```
그 파일은 외형 조정자에 의해 사전 인증을 위해 게시되며 `data/generated/` 외부에 존재하므로 일반 `argvus config apply`로는 갱신되지 않습니다.

만약 인사 담당자가 Kitty 캐시에서 `Invalid color name: 'null'`를 보고하면, `argvus terminal` 패키지를 업데이트하고 인사 담당자를 다시 열어 캐시를 재생성하십시오. 존재하지 않는 표준 악센트는 Kitty 색상이 아니라 기본값이며, 현재 터미널 실행기는 JSON `null`와 잘못된 이전 값을 쓰기 전에 거부합니다 `active_tab_foreground`.

여러 테마 변경 후 기본 터미널이 열리지 않으면, 수동으로 편집하지 말고 파생된 Kitty 캐시를 검사하십시오:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```
위 명령은 정식 환경 설정(preference)을 읽습니다. `.active-theme`는 단지 호환성용 투영(projection)일 뿐입니다. 캐시는 `argvus terminal`에 의해 재생성되며 테마 적용과 동기화됩니다. `/usr/lib/argvus/loading-theme/splash`가 없거나 오래된 경우, 패키지 업데이트가 불완전함을 나타냅니다; 라이브 세션 결과를 판단하기 전에 관련 패키지를 재빌드하고 설치하십시오.

창 테두리가 이전 테마 색상을 유지한다면, Hyprland의 Lua 파일을 수동으로 수정하지 마십시오. 테마 및 강조(accent) 변경 사항은 `config.json`에서 투영된 후, `argvus sessionctl reload`가 Hyprland를 다시 로드하고 세션 생명주기 동안 활성, 비활성 및 그룹 테두리 색상을 다시 적용합니다:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```
테두리 *환경 설정*은 컨트롤 센터 및 제어판에서 `argvus config borders`(또는 그 `borders-switch.sh` 대리자)를 통해 변경됩니다; 다시 불러오기는 세션 라이프사이클 동안 커밋된 색상을 다시 적용하며, 이를 재도출하지는 않습니다.

계획과 유효한 컴포지터 값을 검사하세요:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```
만약 `hyprctl`가 현재 Hyprland 소켓에 연결할 수 없다면, 경계 투사는 실패로 보고되며 의존 컴포넌트는 성공적으로 다시 로드된 것으로 간주되지 않습니다. 다시 시도하기 전에 사용자 세션 저널을 확인하세요.
