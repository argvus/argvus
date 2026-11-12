---
title: 코어 및 디스패처
description: argvus 작업 공간, 그 디스패처 및 공유 Rust 크레이트.
---
`argvus`은 `de/argvus`의 Cargo 작업 공간이며 `/usr/bin/argvus`를 제공하는 패키지입니다. `argvus-cli` 크레이트는 디스패처로 첫 번째 인수를 읽고 라이브러리 명령을 프로세스 내에서 실행하거나, GTK 4 도우미 또는 개인 실행 파일을 실행하거나, 패키지된 스크립트를 실행합니다. 전체 하위 명령 목록은 [command line reference](/docs/reference/command-line/)에 있습니다.

이 작업 공간에는 명령들이 공유하는 크레이트도 포함되어 있습니다:

- `argvus-config`: 표준 구성 저장소이자 생성된 트리(`data/generated/`)의 유일한 작성자입니다. 또한 `argvus config`, `theme-switch` 및 `effects-toggle`에서 프로세스 내에서 사용되는 라이브러리이기도 합니다.
- `argvus-i18n` 및 `argvus-i18n-core`: `resources/locales/`의 카탈로그, 그 검증 및 `org.argvus.i18n` QML 모듈입니다.
- `argvus-tui` 및 `argvus-theme`: 공유 터미널 UI 및 테마 로딩입니다.
- `argvus-runtime`: 번역된 메시지와 명령들이 공유하는 프로세스 도우미입니다.

`argvus-session`는 별도의 저장소로 유지됩니다. 이 저장소는 세션 진입점(`session`, `start`, `tty`)과 라이프사이클 유닛을 제공하며, `argvus sessionctl`은 해당 컨트롤러에 도달합니다.
