---
title: 위젯 원격 측정
description: Waybar 텔레메트리 블록과 argvus widget-telemetry를 통한 토글.
---
텔레메트리 위젯은 작업 표시줄에 표시되는 시스템 정보입니다(헤더, 키, 시스템, CPU 및 GPU, 메모리, 저장소, 프로세스, 네트워크 및 개발자 대시보드). 이것은 `argvus` 패키지 내에서 프로세스 내에 있으며, 셸 수집기는 없습니다.

## 명령어

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```
`block`은 바의 핫 경로입니다: 매 새로 고침 시 호출됩니다. `toggle`은 바가 표시하는 블록을 변경하고 스크롤 창을 이동시킵니다.

블록은 `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network` 및 `dev_dashboard`입니다. 작업 표시줄은 또한 `cpu-temp` 및 `gpu` 프로브를 사용합니다. GPU 프로브는 `--present`(지원되는 GPU가 DRM/sysfs에 표시되는 경우), `--available`(GPU와 btop 명령이 패널에서 사용 가능한 경우), `--available-temp`(측정 가능한 온도를 읽을 수 있는 경우) 및 `--temp`(가장 높은 유효 온도 또는 `N/A`)을 허용합니다.

GPU 온도는 GPU의 PCI 장치에 맞춰집니다. NVIDIA 독점 읽기 값은 `nvidia-smi`/NVML 버스 ID를 사용하며; NVIDIA nouveau, AMD, Intel 읽기 값은 hwmon을 사용하지만 sysfs 장치 링크가 해당 GPU와 고유하게 일치할 때만 가능합니다. 작업 표시줄 온도는 감지된 GPU 중 유효한 값의 최대값입니다. 누락되었거나, 유효하지 않거나, 모호한 읽기 값은 사용 불가능 상태로 유지됩니다. NVIDIA 독점 온도는 드라이버 제공자가 필요하며; AMD 및 Intel 온도는 연결된 hwmon 센서를 커널이 노출해야 합니다. Intel 온도는 btop 1.4.7의 GPU 패널에서 제공되지 않습니다.

## 세션 시작 시

`argvus-widget-telemetry.service` 사용자 유닛은 위젯이 활성화되어 있을 때만 세션과 함께 시작됩니다. 마스터 스위치는 설정에서 `effects.widget_telemetry_enabled`이며, 기본값으로 활성화되어 있어 새 계정은 로그인 시 위젯을 시작합니다.

토글이 `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state`를 작성하면 그 파일이 대신 결정합니다. `enabled`의 첫 번째 줄만 위젯을 시작합니다. 이전 `desktop-telemetry-state` 및 `sysinfo-state` 파일은 현재 파일이 없을 때 여전히 읽힙니다.

텔레메트리 레이블은 지원되는 모든 10개 언어에서 간결합니다. CPU, GPU, IP, DE, WM 및 커널과 같은 기술 식별자는 기술적 의미를 유지합니다. 가동 시간은 `/proc/uptime`에서 읽혀 축약 단위(예: `1d 12h 49m`)로 표시됩니다; 더 이상 시스템 로케일의 전체 단어로 확장되지 않습니다. 하드웨어 모델 이름 및 기타 제공자 값은 여전히 위젯 폭에 영향을 줄 수 있습니다.
