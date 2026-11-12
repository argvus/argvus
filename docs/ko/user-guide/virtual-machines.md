---
title: 가상 머신
description: 가상 그래픽 하드웨어로 ARGVUS를 실행하십시오.
---
`argvus start`은 가상화를 감지하고, 감지된 환경에서 필요할 경우 소프트웨어 렌더링 및 커서/수정자 대체 등을 포함한 영향을 받는 세션에 대한 호환성 설정을 내보냅니다.

가상 머신이 아직도 렌더링하지 못하는 경우, 재정의(overrides)를 추가하기 전에 세션 환경과 로그를 검사하십시오. [graphics troubleshooting](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/)을 참조하십시오.

가상화가 감지되면, `argvus start`은 소프트웨어 렌더링, 소프트웨어 커서 및 수정자 대체 등을 포함한 세션용 호환성 변수를 내보냅니다. 이로 인해 일반 로그인 경로를 먼저 시도하게 되며, 데스크탑이 VM에서 실행되고 있다는 이유만으로 영구 재정의를 추가하지 마십시오.

감지가 충분하지 않은 경우, 진단 대체 수단으로 사용자 Hyprland 재정의를 사용하십시오:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```
VM 그래픽스 스택이 필요할 때만 이것을 추가하십시오:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```
세션 환경을 변경한 후에는 로그아웃했다가 다시 로그인하십시오. `echo $LIBGL_ALWAYS_SOFTWARE`로 활성 값을 확인하세요. 소프트웨어 렌더링은 데스크톱을 사용할 수 있게 만들 수 있지만 성능을 저하시킬 수 있습니다.
