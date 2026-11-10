---
title: 시스템 모니터
description: ARGVUS 시스템 모니터를 열어라.
---
다음으로 설정된 모니터를 엽니다:

```sh
argvus --system-monitor
```
`argvus system-monitor` 하위 명령어는 네이티브 진입점과 btop 통합을 제공합니다. 작업 표시줄에 표시되는 원격 측정은 `argvus widget-telemetry`에서 제공하는 별도의 기능입니다. CPU, 메모리 및 GPU 모듈을 클릭하면 브랜드 헤더와 `Esc Close` 푸터가 있는 ARGVUS TUI 패널 안에서 btop이 열립니다. GPU 모드는 최대 여섯 개의 GPU 상자를 표시하며 CPU, 메모리, 네트워크 또는 프로세스 상자는 표시하지 않습니다. 패널을 닫으려면 `q` 또는 `Esc`를 누르세요. 하단 Kitty 탭에는 현지화된 `Press q to close` 힌트가 표시됩니다. 독립 실행형 `argvus system-monitor --cpu`, `--mem` 및 `--gpu` 호출은 선택된 상자와 함께 btop을 직접 실행합니다.

GPU 패널은 설치된 btop 빌드의 기능과 시스템의 장치 제공업체에 따라 달라집니다. NVIDIA 모니터링은 btop에서 GPU 지원, 설치된 NVIDIA 드라이버의 NVIDIA Management Library, 그리고 장치 접근을 필요로 합니다. AMD 모니터링은 선택 사항이며 설치되지 않았을 수도 있는 ROCm SMI를 사용합니다. Intel GPU 데이터는 sysfs에서 읽으며 사용자의 해당 인터페이스 접근 권한에 따라 달라집니다; btop 1.4.7은 Intel 사용률, 전력, 클럭을 보고하지만 Intel 온도는 제공하지 않습니다. NVIDIA nouveau 및 기타 드라이버는 장치 연결 hwmon을 통해 GPU 온도를 작업 표시줄에 노출할 수 있으며, btop 제공업체 지원은 사용 가능한 드라이버 및 라이브러리에 따라 달라집니다. ARGVUS는 장치 권한을 변경하거나 btop을 관리자 권한으로 실행하지 않습니다. GPU가 노출되지 않거나 지원 또는 제공업체가 없어서 btop이 3초 이내에 GPU 박스를 렌더링하지 못하면 패널에 지역화된 사용 불가 메시지가 표시됩니다.

모니터는 먼저 `$XDG_CONFIG_HOME/argvus/config.json`에서 선택된 테마와 강조색을 읽습니다. 생성된 btop 파일은 여전히 파생된 캐시 상태를 유지하며, `.active-theme`와 `.accent-color`는 호환성 대체용입니다. btop이 자체 테마 파일을 유지하기 때문에, 모니터는 표준 변경이 커밋된 후 외부 어댑터로서 테마 트랜잭션(`argvus theme-switch`)에 의해 조정됩니다; 이것은 생성된 트리의 일부가 아닙니다.  

## 런타임 계약

- 테마와 강조 색상은 표준 구성(` and ` and `). The files `). The files ~/.config/argvus/data/.active-theme 및 `.accent-color`는 대체값)에서 가져옵니다.  
- 생성된 btop 구성과 테마는 `$XDG_CACHE_HOME/argvus/argvus-system-monitor`에 작성됩니다. 이는 도출된 상태이므로 편집하지 마십시오.  
- 기본값은 `/usr/share/argvus-system-monitor/btop`에 설치됩니다.  
- 모듈 단축키(`--mem`, `--cpu`, `--net`, `--proc`)는 첫 번째 인수로 읽기 전용입니다.  
  btop의 자체 `-c`/`--config` 또는 `--themes-dir`를 전달하면 해당 경로로 btop이 실행되며 구성은 작성되지 않습니다.  
- `--panel [--cpu|--mem|--gpu]`는 ARGVUS TUI 셸에 btop을 포함시킵니다. 이 모드는 작업 표시줄에서 사용됩니다; `--panel`가 없으면 모니터는 계속해서 btop으로 자신을 대체합니다.
