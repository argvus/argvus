---
title: 야간 조명
description: Hyprland에서 색 온도와 고정된 로컬 일정을 조정하세요.
---
열기 **제어 센터 > 데스크탑 및 입력 > 디스플레이 > 나이트 라이트**, **나이트 라이트**를 검색하거나 `argvus control-center displays night-light`를 실행합니다.

나이트 라이트와 그 일정은 처음에는 꺼져 있습니다. **1000에서 20000 켈빈** 사이에서 온도를 선택하세요 (기본값 **4500 K**). 낮은 값일수록 더 따뜻한 색상이 나타납니다. 페이지에는 저장된 수동 활성화, 실제 상태 및 다음 전환이 표시됩니다. 값을 입력하면 수정되며, Esc를 누르면 취소하거나 디스플레이로 돌아갑니다. 저장 전에 잘못된 값은 허용되지 않습니다. 변경 사항은 즉시 저장되고 적용됩니다. 저장 후 적용에 실패하면, 저장된 설정은 실제 상태와 오류와 함께 표시됩니다.

## 일정 및 빠른 작업

고정 일정은 현재 지역 시간대를 사용합니다. 시작 시간은 포함되고 종료 시간은 제외됩니다; **20:00–07:00**은 자정을 넘어갑니다. 시작과 종료는 서로 달라야 하며 `HH:MM`를 사용합니다. 위치나 태양 계산은 사용되지 않습니다.

**제어판**의 **디스플레이** 카드를 사용하거나 빠른 토글을 위해 **SUPER + Shift + B**를 사용하세요. 밝기 선택기는 모든 밝기 동작을 유지하며, 백엔드가 가능할 경우 야간 조명(Night Light)도 추가합니다. 밝기 백엔드가 없는 디스플레이에서도 적용됩니다. 영구적인 작업 표시줄 모듈은 없습니다.

스케줄링이 꺼져 있을 경우, 수동 활성화는 지속됩니다. 스케줄링이 켜져 있을 경우, 빠른 토글은 다음 예정된 전환 시까지 런타임 오버라이드를 생성합니다. 스케줄이나 온도를 변경하면 해당 오버라이드가 해제됩니다. 로그아웃, 새로운 컴포지터 인스턴스, 혹은 다음 전환 변경(시간대 변경 포함) 또한 이를 무효화합니다.

## 명령어와 상태

모든 인터페이스는 동일한 공개 명령어를 사용합니다:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```
`status`은 JSON 스키마 버전 1: `available`, `unavailable_reason`, `settings`(활성화, 온도, 스케줄_활성화, 시작, 종료), `active`, `effective_temperature`, `mode`(`manual` 또는 `scheduled`), `manual_override`, `next_transition`(오프셋이 있는 ISO 8601), `minimum_temperature` 및 `maximum_temperature`를 반환합니다. 사용 가능하지 않은 경우 유효한 필드는 null이며, 아이덴티티가 활성화된 경우 유효 온도는 null입니다. 사용 가능성 실패는 상태(status)에 보고되며, 동작 실패는 진단과 함께 0이 아닌 값으로 반환됩니다.

표준 기본 설정은 `/displays/night_light`이며, `argvus config`에서 `$XDG_CONFIG_HOME/argvus/config/displays.json`에 원자적으로 저장됩니다. 불안정한 재정의 및 동작 잠금은 `$XDG_RUNTIME_DIR/argvus/night-light/` 아래에서 존재합니다. 생성된 모니터 파일은 독립적입니다. 이 기능을 구성하기 위해 생성된 파일을 편집하지 마십시오.

## 런타임

Hyprland와 **hyprsunset >= 0.4.0**가 필요합니다. 세션은 업스트림 `hyprsunset.service`를 한 번 시작하고, `argvus-night-light-init.service`를 통해 저장된 상태를 적용하며, `argvus-night-light.timer` 및 `argvus-night-light.service`를 통해 매 분마다 스케줄을 평가합니다. 시계 및 시간대 변경도 평가를 트리거합니다. 스케줄링이 꺼져 있는 경우, 주기적인 작업은 수동 상태를 그대로 둡니다. 반복 적용 시 효과적인 아이덴티티/온도를 조회하고 변경되지 않은 기록은 건너뜁니다. 컴포지터 재시작은 필요하지 않습니다.

ARGVUS는 업스트림 서비스를 위한 systemd 드롭인을 제공합니다: `--identity`로 시작하며 구성 검색을 `/usr/share/argvus/session/config/night-light`으로 격리합니다. 이는 경쟁하는 네이티브 프로필을 피하고 개인 `hyprsunset.conf` 파일을 그대로 둡니다. 이 드롭인은 ARGVUS 세션 패키지가 설치되어 있는 동안 해당 업스트림 서비스에 적용되며, ARGVUS 외부에서 시작되는 경우도 포함됩니다. 기존 사용자 systemd 오버라이드는 우선권을 가질 수 있으며, 해당 상황을 진단할 때 `systemctl --user cat hyprsunset.service`을 확인하십시오. Hyprsunset 0.4.0은 `--config`의 별도 경로 인수를 거부하므로, 구성 검색은 대신 서비스 로컬 XDG 변수로 격리됩니다.

백엔드는 공식 Hyprsunset IPC(`temperature`, `identity`, `identity get`)를 사용하며, 제한된 I/O와 변경 후 읽기 기능을 갖추고 있습니다. 셰이더, 대체 컴포지터 백엔드, 시스템 구성 쓰기 또는 권한 상승은 사용되지 않습니다. 사용 불가한 컨트롤은 제어 센터에서 비활성화되고 디스플레이 카드에서는 숨겨집니다.
