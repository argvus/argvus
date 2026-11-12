---
title: 첫 번째 구성
description: ARGVUS를 구성하기 위한 실용적인 첫 부팅 경로.
---
첫 번째 그래픽 세션 후에는 ARGVUS의 모든 부분을 구성할 필요가 없습니다. 데스크탑 환경에 영향을 주는 설정부터 시작한 후 필요에 따라 하드웨어와 서비스를 구성하세요.

## 유용한 순서

1. [desktop overview](/docs/user-guide/desktop/)를 읽고 창, 작업 공간, 작업 표시줄, 제어판 및 런처를 확인합니다.
2. [Control Center](/docs/components/argvus-control-center/)를 열고 **모양(Appearance)**에서 테마, 모드, 강조 색상 및 배경 화면을 선택합니다.
3. 기본 간격, 테두리 또는 작업 표시줄 간격이 화면에 맞지 않으면 [windows and layout](/docs/argvus-hyprland/windows-and-layout/)을 조정합니다.
4. [taskbar](/docs/argvus-taskbar/taskbar/)를 설정하고 어느 제어판 카드를 표시할지 결정합니다.
5. [keyboard shortcuts](/docs/argvus-hyprland/keyboard-shortcuts/)를 검토하고 실제로 사용하는 단축키만 변경합니다.
6. [mouse and touchpad](/docs/argvus-hyprland/input/), 디스플레이, 네트워킹 및 전원 설정을 관련 설정 페이지에서 구성합니다.

처음 네 단계는 선택적 개인화 단계입니다. 나머지 단계는 일반적으로 하드웨어, 언어 또는 작업 흐름에 따라 필요할 때만 수행합니다.

## 설정을 변경할 두 가지 장소

**Control Center**는 ARGVUS의 구성 방법을 설명하는 설정에 사용하세요: 외관, 레이아웃, 글꼴, 입력, 단축키, 언어, 지역 및 기본 애플리케이션. **Control Panel**은 볼륨, 밝기, 네트워크 상태, 알림, 전원 및 세션 제어와 같은 상태 및 자주 사용하는 세션 작업에 사용하세요.

전체 구분은 [Control Center vs Control Panel](/docs/components/argvus-control-center/)를 참고하고, 빠른 색인은 [Where to configure things](/docs/user-guide/where-to-configure/)에서 확인하세요.

## 변경, 지속성 및 복구

대부분의 Control Center 변경 사항은 세션 실행 중에 적용되며 ARGVUS 사용자 상태로 저장됩니다. 외관 변경은 표준 구성에 커밋되며, `argvus-config`는 Hyprland와 셸에서 사용하는 파일을 투영합니다. Control Panel 토글은 직접적인 작업이거나 영구 구성이 아닌 세션 상태 변경일 수 있습니다.

페이지가 **기본값 재설정**을 제공하면, 생성된 파일을 삭제하는 대신 해당 페이지 액션을 사용하세요. Appearance에는 악센트, 배경화면, 레이아웃 값에 대한 별도의 리셋 액션이 있습니다. 키 바인딩 페이지는 자체 복원 액션을 가지고 있습니다. 모든 데스크톱 설정에 대한 단일 보편 리셋 버튼은 없습니다.

## 다음 단계

- [Appearance](/docs/user-guide/appearance/) — 테마, 배경화면, 효과 및 레이아웃.
- [데스크톱 레이아웃](/docs/argvus-hyprland/windows-and-layout/) — 간격, 경계, 패널 공간을 이해하세요.
- [입력 장치](/docs/argvus-hyprland/input/) — 마우스 및 터치패드 제어.
- [테마 및 배경화면 문제 해결 ](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — 시각적 변화가 예상과 다르게 발생할 때 복구.
