---
title: 설정 파일
description: 사용자, 시스템, 패키지 기본값 및 생성된 ARGVUS 구성.
---

| 계층 | 위치 | 의미 |
| --- | --- | --- |
| 시스템 | `/etc/argvus/` | 서비스 및 공급자에 대한 관리자 구성. |
| 패키지 기본값 | `/usr/share/argvus/` | 읽기 전용 구성요소 기본값 및 런타임 자산. |
| 사용자 | `$XDG_CONFIG_HOME/argvus/config.json` | 표준 휴대용 환경 설정. |
| 관리 데이터 | `$XDG_CONFIG_HOME/argvus/data/` | 구성요소 데이터, 호환성 파일 및 내부 메타데이터. |
| 생성됨 | `$XDG_CONFIG_HOME/argvus/data/generated/` | `config.json`의 투영, `argvus-config`에 의해서만 작성됨. 절대 편집하지 말고, 다른 곳에 구성으로 복사하지 말며, 출처로 다시 읽지 말 것. |
| 런타임 | `$XDG_RUNTIME_DIR/` 및 `$XDG_CACHE_HOME/` | 소켓, 잠금 및 일시적 상태. |

활성 테마, 강조, 사용자 지정 배경화면 및 효과 상태는 논리적 사용자 상태입니다. 관리되는 논리 모델은 [Canonical configuration](/docs/reference/argvus-config/)에 문서화되어 있습니다. 생성된 Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, 터미널, 글꼴 및 이동식 장치 파일은 독립적인 진실 소스로 취급해서는 안 됩니다.

`argvus-config`는 또한 테마 레이어를 `data/waybar/argvus-taskbar.{jsonc,css}` 및 `data/waybar/argvus-widget-telemetry.{jsonc,css}`로 복제합니다. 이러한 복사본은 외관 변경 시 패키지 기본값으로 대체되며, 구분된 관리 블록이 다시 작성되므로 수동으로 관리되는 파일이 아니라 파생된 것으로 취급해야 합니다. `$XDG_CONFIG_HOME/<app>` 아래의 네이티브 애플리케이션 구성은 다릅니다. 이는 외관 오케스트레이터가 작성한 어댑터 대상이며, 프로젝터에 의해 작성되지 않습니다.

## 단일 작성자 규칙에 대한 예외

`argvus config`는 `data/generated/`의 유일한 작성자입니다. 마이그레이션에 대한 한 가지 예외가 존재합니다: 사용자의 패키지 파일 복사본이 생성될 때, 이전 버전이 `data/generated/` 아래에 남긴 파일이 사용자의 위치(`data/`)로 이동되어 사용자가 수정한 내용을 유지합니다. 이 이동은 현재 버전에서 생성하지 않는 파일에만 발생합니다. 새로운 파일의 경우 이 경로에 의존하지 마십시오: 생성된 파일은 여기에서가 아니라 프로젝션에 추가하십시오.
