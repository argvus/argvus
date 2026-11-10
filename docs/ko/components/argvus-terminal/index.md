---
title: 터미널
description: ARGVUS 터미널과 터미널 애플리케이션 프로필을 엽니다.
---
다음으로 구성된 터미널을 엽니다:

```sh
argvus --terminal
```
`argvus terminal` (`argvus`의 하위 명령어)는 터미널 통합과 Kitty 구성을 제공합니다. `argvus-app-profiles`는 Superfile 및 Yazi와 같은 터미널 애플리케이션용 프로필을 제공하며, `argvus` 디스패처는 제공하지 않습니다.

`argvus-config`는 테마를 `data/generated/terminal/`로 투영하며, `argvus --yazi`와 `argvus --spf`는 자신들의 구성을 홈 디렉터리에 복사된 파일이 아닌 투영된 `data/generated/yazi/` 및 `data/generated/superfile/` 트리에서 읽습니다. 애플리케이션 자체 구성 디렉터리에서의 완전한 네이티브 오버라이드는 여전히 우선하므로, 투영된 트리를 자신의 구성으로 교체할 수 있습니다.

테마 변경 시 `$XDG_CACHE_HOME/argvus/argvus-terminal` 아래에서 Kitty의 파생 파일이 다시 생성됩니다. 생성기는 업데이트를 직렬화하고 각 파일을 원자적으로 교체합니다. 터미널이 여전히 열리지 않는 경우, 활성 테마를 확인하고 캐시를 다음과 같이 다시 생성하십시오:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```
터미널은 먼저 `$XDG_CONFIG_HOME/argvus/config.json`에서 관리되는 테마와 강조색을 읽습니다. 이전 버전 설치를 위한 레거시 `.active-theme` 및 `.accent-color` 파일은 여전히 백업으로 남아 있습니다.

## 투명도 및 블러

**제어 센터 → 모양 → 터미널**에서 `0%`에서 `100%`까지와는 별도로 **투명도**와 **블러**를 설정할 수 있습니다. 변경 사항은 **적용** 버튼을 통해 적용되며 `$XDG_CACHE_HOME/argvus/argvus-terminal` 아래에서 Kitty 프로필을 다시 생성합니다. 기존 Kitty 창은 재로드를 통해 업데이트된 설정을 받습니다.

`0%` 투명도는 배경을 불투명하게 유지하며, `100%`는 완전히 투명하게 만듭니다. 블러는 Kitty에서 지원하는 배경 블러 반경을 사용하며, 창이 투명할 때만 시각적인 효과가 있습니다.

## 장시간 실행되는 명령 알림

30초 이상 실행되는 명령은 터미널 창이 비활성 상태일 경우 완료 시 알림을 보냅니다. 알림은 이미 활성화된 셸 통합을 사용하는 Kitty의 `notify_on_cmd_finish`를 사용하므로 `~/.zshrc` 및 `~/.bashrc`는 변경할 필요가 없습니다. 30초 임계값은 Kitty 구성에서 고정되어 있습니다.

## TUI 앱 모드

ARGVUS 컨트롤 센터와 작업 표시줄 TUI 팝업은 전용 Kitty 앱 프로필과 함께 `argvus tui-terminal`를 사용합니다. 이는 활성 ARGVUS 테마와 글꼴을 상속하고, 안정적인 창 클래스를 할당하며, 셸을 열지 않고 자식과 함께 종료됩니다. 컨트롤 센터는 탭을 숨기고 독립적인 투명도와 블러 값을 가지며, 작업 표시줄 모니터와 온도 팝업은 지역화된 `Press q to close` 제목을 위해 하단 탭 바를 사용합니다. 런처는 `--title TITLE`를 받아 고정된 Kitty 창과 탭 제목을 설정합니다. 일반 터미널은 탭과 설정을 유지합니다.

런처 환경에서 LANG이 없을 경우, `argvus terminal`는 Kitty가 키보드 조합을 초기화하기 전에 `$XDG_CONFIG_HOME/locale.conf`에서 개인 로케일을 제공합니다(기본값은 영어). 기존 로케일 환경 오버라이드는 유지됩니다. 업데이트 후 Kitty를 재시작하세요: 기존 셸에서 LANG을 내보내도 Kitty의 조합 테이블이 다시 초기화되지 않습니다.

세션 초기화에서 상속된 비어 있는 LC_ALL 및 LC_CTYPE는 Kitty 자식 환경에서 제거됩니다. 그렇지 않으면 Kitty는 비어 있는 로케일을 명시적인 것으로 간주하여 유효한 LANG이 있어도 조합 테이블을 로드할 수 없습니다. 비어 있지 않은 오버라이드는 유지됩니다. 이는 새 프로파일에도 적용되며, 런처 업데이트 후 Kitty를 다시 열어야 합니다.

전용 제어 센터 창(`argvus-control-center`)은 실행별 Kitty 오버라이드 `confirm_os_window_close=0`를 수신하므로, 합성기 종료 작업은 확인을 요청하지 않습니다. 이 오버라이드는 공유 터미널 프로필에 기록되지 않으며, 다른 TUI와 일반 터미널은 기존 정책을 유지합니다.
