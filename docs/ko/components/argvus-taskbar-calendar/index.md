---
title: 달력
description: ARGVUS 작업 표시줄 달력을 사용하세요.
---
`argvus-taskbar-calendar`는 Waybar 날씨 모듈에서 열리거나 다음을 통해 열리는 GTK4 캘린더 팝업입니다:

```sh
argvus --calendar
argvus taskbar-calendar toggle
argvus taskbar-calendar status
```
로컬 이벤트, 알림 및 ICS 가져오기/내보내기를 지원합니다. 사용자의 서비스는 `argvus-taskbar-calendar.service`입니다. 설치된 버전에서 지원되는 옵션에는 `argvus taskbar-calendar --help`을 사용하십시오.

캘린더는 Sticky 및 Spaced 변형을 포함하여 활성 ARGVUS 테마를 따릅니다. 테마 변경은 생성된 사용자 캐시를 통해 적용됩니다. 실행 중인 팝업이 명시적인 새로 고침이 필요한 경우 `argvus taskbar-calendar reload`을 사용하십시오.

작업 표시줄 날짜 모듈에서 열면 팝업이 표시줄 가장자리에 고정됩니다. 위쪽 작업 표시줄 아래 또는 아래쪽 작업 표시줄 위에 열리며 날짜 모듈의 오른쪽 가장자리에 맞춰집니다. 작업 표시줄 지오메트리가 없는 직접 실행은 포인터 또는 모니터 대체를 사용합니다.

## 명령어

```bash
argvus taskbar-calendar                 open the calendar
argvus taskbar-calendar toggle          show or hide it (the Waybar date module uses this)
argvus taskbar-calendar show | hide
argvus taskbar-calendar config          open the system configuration for editing (sudo)
argvus taskbar-calendar config-path     print the configuration path
argvus taskbar-calendar reload
argvus taskbar-calendar import file.ics
argvus taskbar-calendar export --output calendar.ics
argvus taskbar-calendar sync
argvus taskbar-calendar status
argvus taskbar-calendar service
```
## 이벤트 및 알림

이벤트는 기본적으로 꺼져 있습니다. 팝업에서 **설정 > 이벤트**에서 켜세요; 선택 내용은 상태 디렉토리에 저장되며 알림을 보여주는 `argvus-taskbar-calendar.service`를 활성화하고 시작합니다. 끄면 서비스가 중단됩니다.

- **+ 이벤트** (또는 `n`)는 선택한 날짜의 편집기를 엽니다. 오늘부터 이벤트를 생성하고 변경할 수 있으며, 과거 날짜의 이벤트도 열어 삭제할 수 있습니다.
- 새로운 이벤트는 09:00에 시작하며 설정에서 지정된 기본 길이만큼 지속됩니다. 시작 시간을 옮기면 길이가 유지됩니다; 종료 시간이 시작 시간보다 빠르면 이벤트는 다음 날에 종료됩니다.
- 각 이벤트는 시작 시 알림과 소리를 표시하며, **알림** 및 **반복** 상자 설정과 관계없이 알림이 울립니다. 시작 후에 저장된 이벤트(예: 당일 생성된 종일 이벤트)는 30초 내에 알림을 보냅니다.
- **알림**은 시작 전에 선택한 시간에 이전 알림을 추가합니다(그 시간이 이미 지난 경우 시작 시 알림). 
- 종일 이벤트에서는 시간 필드가 **알림 시간**으로 변합니다: 이벤트가 하루 종일 계속되며 해당 시간에 알림이 울립니다(00:00은 하루가 시작될 때 알림). 이벤트를 다시 열면 시간이 유지됩니다.
- 종일 이벤트의 경우, **반복**은 그 시간 이후 선택한 간격에 따라 다시 알림을 보냅니다.
- 시작 시간을 변경하지 않고 이벤트를 편집해도 이미 표시된 알림은 반복되지 않습니다.
- 알림은 캘린더 언어(Settings > 언어)를 따릅니다.
- 이벤트가 있는 날에는 점(`•`)이 표시되며, 하룻밤 이벤트나 다일 이벤트의 모든 날이 포함됩니다. 날짜는 현지 시간대를 따릅니다.
- 로컬 이벤트는 종료 후 10분 후에 자동으로 제거됩니다. 이벤트가 저장 시 이미 종료된 경우, 저장 후 최소 10분 동안은 계속 사용할 수 있습니다.

가져온 `.ics` 파일은 날짜, 시간대(`TZID`) 및 알람(`VALARM`)을 유지합니다. 가져온 이벤트는 자체 알람 외에도 시작 시 알림을 보냅니다. 반복 이벤트(`RRULE`)는 저장되고 내보내지지만 첫 번째 발생만 표시되고 알림됩니다. 패키지 업데이트 후 알림 서비스가 종료되고 systemd가 30초 이내에 새 버전을 시작합니다. `sync`는 구성된 원격 캘린더만 보고하며, CalDAV 동기화는 아직 구현되지 않았습니다.

## 경로

- 구성: `/etc/argvus/taskbar/calendar/config.toml` (시스템 전체; 기본값은 패키지와 함께 제공됩니다).
- 스타일: `/etc/argvus/taskbar/calendar/style.css` 및 `theme.css`; 활성 ARGVUS 테마의 색상은 아래 캐시를 통해 빌드된 테마에서 가져옵니다.
- 데이터: `$XDG_DATA_HOME/argvus-taskbar-calendar/` (또는 `~/.local/share/argvus-taskbar-calendar/`).
- 상태: `$XDG_STATE_HOME/argvus/taskbar/calendar/` (또는 `~/.local/state/...`); 이벤트 토글이 여기에 저장됩니다.
- 캐시: `$XDG_CACHE_HOME/argvus-taskbar-calendar/` (또는 `~/.cache/argvus-taskbar-calendar/`).

하이라이트 색상은 `~/.config/argvus/data/.accent-color`에서 읽어 테마 색상 위에 실행 시 적용됩니다.

자동 언어 선택은 사용자 `locale.conf`를 따르며, 기본값은 영어입니다. 명시적인 달력 언어 설정은 여전히 자동 선택을 무시합니다.

작업 표시줄 캘린더, ARGVUS Focus, ARGVUS Clock 및 제어판은 상호 배타적입니다: 하나를 열면 다른 항목은 닫힙니다.

캘린더 읽기 실패는 비어 있는 일정과 별도로 표시됩니다. 데이터베이스 열기, 일일 이벤트 불러오기 및 월별 지표 불러기는 각각 다른 메시지를 가집니다. 동일한 요일을 다시 선택하여 재시도하십시오. 새로 고침 실패 시 이전에 불러온 데이터는 동일한 날 또는 달에 대해서만 유지되며, 다른 쿼리에서 유효한 결과는 계속 사용할 수 있습니다.

## 공유 언어 카탈로그

인터페이스는 10개 언어 모두에 대해 공유 `argvus-i18n` 카탈로그를 사용합니다. 텍스트는 마스터 작업 공간의 `resources/locales/<locale>/taskbar-calendar.json`에 저장되며 `argvus`에 의해 설치됩니다. 영어가 기본값이며, 새로운 언어는 더 이상 로컬 이중 언어 사전을 요구하지 않습니다.
