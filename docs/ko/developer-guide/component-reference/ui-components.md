---
title: UI 구성 요소
description: ARGVUS 사용자 인터페이스 패키지.
---
UI 패키지에는 `argvus-control-panel`, `argvus-taskbar`, `argvus-launcher`가 포함되며, `argvus` 하위 명령어에는 `argvus control-center`(`argvus`의 헬퍼), `argvus taskbar-calendar`, `argvus widget-telemetry`, `argvus terminal`, `argvus system-monitor`, `argvus removable-devices`, `argvus focus`, `argvus clock`가 포함됩니다. 캘린더, Focus, 시계는 레이어-셸 위치 지정 및 해제를 위해 공유 `argvus-taskbar-popup` 크레이트를 사용합니다. Focus는 또한 프로세스 내 `argvus focus status` 경로를 통해 Waybar 상태 리더를 노출합니다.

사용자 대상 행동은 기능별로 문서화되어 있으며, 이 페이지는 패키지 소유권을 기록합니다.
