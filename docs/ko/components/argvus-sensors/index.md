---
title: 온도 센서
description: lm_sensors JSON으로 지원되는 실시간 온도 개요.
---
`argvus sensors`는 작업 표시줄 CPU 온도 팝업에서 사용되는 진행 중인 Ratatui 패널을 엽니다. 이 패널은 `sensors -j`를 실행하고, 하드웨어에서 보고된 칩 이름과 라벨별로 측정값을 그룹화하며, 2초마다 새로 고칩니다. GPU 모니터링은 `argvus system-monitor --gpu`에서 처리됩니다.

측정값은 섭씨를 사용합니다. 패널은 lm_sensors가 제공할 경우 `max`, `crit` 및 경보 상태를 보여줍니다. 눈금계는 양의 한계치가 있을 때만 표시됩니다. 누락된 `lm_sensors`, 명령 실패, 잘못된 JSON 및 빈 측정값은 복구 가능한 상태로 계속 표시되며, 다음 주기에서 폴링이 재시도됩니다.

`q` 또는 `Esc`를 눌러 닫습니다. 작업 표시줄은 `argvus tui-terminal`를 통해 Kitty에서 패널을 엽니다; 로컬화된 하단 탭 제목은 닫기 힌트를 표시합니다. 이 명령은 CPU 온도 세부 정보를 소유하며, GPU 온도 아이콘은 GPU 전용 btop 모니터를 엽니다. 작업 표시줄 패키지는 이미 `lm_sensors`에 의존하므로 별도의 패키지나 의존성은 필요하지 않습니다.
