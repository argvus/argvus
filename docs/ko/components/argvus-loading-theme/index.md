---
title: 로딩 오버레이
description: 세션이 다시 로드되는 동안 표시되는 오버레이입니다.
---
로딩 오버레이는 ARGVUS가 세션을 직접 다시 로드할 때(`argvus sessionctl reload`) 표시되는 GTK 4 레이어-셸 표면입니다. 테마 전환은 이를 사용하지 않으며: 데스크탑이 그대로 변경됩니다. 이것은 `argvus` 패키지에 포함된 헬퍼이며 `/usr/lib/argvus/loading-theme/splash`에 있으며, `/usr/bin`에는 배치되지 않습니다.

로그인은 이를 사용하지 않습니다: 그리터 스플래시와 세션 시작 스피너는 핸드오프 헬퍼의 Wayland 모드로, GTK 없이 동일한 스피너를 그리고 수 밀리초 내에 첫 번째 프레임을 표시합니다(참조 [Greeter, lock and splash](/docs/developer-guide/subsystems/greeter-lock-and-splash/)]).
`--spinner-only`에서는 오버레이가 그 스피너만 표시하고 `SIGTERM`에서 종료됩니다. `--detached`에서는 이를 시작한 프로세스가 종료되어도 종료되지 않습니다; 세션 재로드는 두 플래그를 사용하며 pid 파일을 통해 이를 중지합니다.
선택된 테마는 ARGVUS 데이터 디렉토리에서 읽으며, 없을 경우 패키지된 ARGVUS Dark 팔레트로 되돌아갑니다. 준비 상태는 프로세스가 시작될 때가 아니라 매핑된 모든 표면의 첫 번째 프레임 이후에만 보고됩니다.
