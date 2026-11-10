---
title: 요구 사항
description: ARGVUS의 시스템 요구 사항 및 전제 조건.
---
ARGVUS는 Arch Linux를 대상으로 하며 Wayland를 지원하는 그래픽 스택, Hyprland 및 패키지에서 가져오는 런타임 종속성을 필요로 합니다. 디스플레이 관리자는 선택 사항이며, ARGVUS는 greetd를 통한 그래픽 로그인 흐름과 수동 TTY 흐름을 지원합니다.

설치 전에 다음을 확인하십시오:

- 시스템이 `pacman`로 완전히 업데이트되었는지;
- 작동하는 커널과 그래픽 드라이버가 설치되었는지;
- 사용자가 Wayland 세션을 실행할 수 있는지;
- 패키지 저장소가 구성될 때 네트워크가 사용 가능한지.

가상 머신은 [Virtual machines](/docs/user-guide/virtual-machines/)에 설명된 호환성 설정이 필요할 수 있습니다.
