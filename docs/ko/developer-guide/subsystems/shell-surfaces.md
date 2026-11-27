---
title: 조개 껍데기 표면
description: 작업 표시줄, 제어판 및 원격 측정 소유권.
---
`argvus-shell`는 패치된 `Quickshell Taskbar`를 제공합니다. `argvus-shell`는 작업 표시줄 액션과 UX를 소유하며; 제공되는 작업 표시줄 JSONC와 CSS는 `config.json`의 투영이며, `argvus-config`는 `data/generated/taskbar/` 아래에서 활성 레이어를 작성하고 `data/taskbar/` 아래에 복사본을 작성합니다. `argvus-shell`는 Quickshell 카드를 소유합니다. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; 그 JSONC와 CSS는 패키지된 기본값에서 외형 변경 후 `data/taskbar/`에 작성되고, 그 후 `argvus-config`는 구분된 `ARGVUS_TELEMETRY_*`와 폰트 블록만 다시 작성합니다. `argvus-session`는 그들의 사용자 서비스를 소유합니다.

현재 이 런타임 모델에는 `argvus-shell` 패키지나 `argvus-shell.service`가 없습니다.
