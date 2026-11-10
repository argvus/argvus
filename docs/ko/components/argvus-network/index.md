---
title: 네트워킹 및 블루투스
description: ARGVUS를 통해 NetworkManager와 Bluetooth를 관리합니다.
---
`argvus network` 하위 명령어는 NetworkManager를 보고하고 제어하며, `argvus bluetooth`는 블루투스 어댑터 상태와 사용 가능성을 보고하고 제어합니다. 두 명령어는 `argvus` 내부에서 프로세스로 실행되며, `networkctl`와 `bluetoothctl`는 그대로 별칭으로 남아 있습니다. 제어판과 Waybar는 이 명령어들을 호출합니다.

설치된 하위 명령어를 확인하려면 두 명령어 중 어느 쪽에서든 `--help`를 사용하십시오. 네트워크와 블루투스 UI 통합은 Blueman과 같은 선택적 고급 관리 기능과 별도로 동작합니다.

제어 센터는 네트워크 상태를 구성 페이지와 분리합니다: **네트워크 → 상태**는 현재 상태를 요약하며, 인터페이스, Wi-Fi, Ethernet, VPN, DNS, 프록시 및 방화벽은 제공자가 사용 가능한 경우 별도의 경로로 나뉩니다. **블루투스 → 상태**, **장치**, **페어링**은 어댑터 상태와 장치 동작을 다룹니다. 이 페이지들은 공급자 기반 시스템 동작을 수행하며, 작업 표시줄과 제어판은 빠른 상태/동작 접근을 제공합니다.

네트워크 및 블루투스 변경 사항은 권한이 필요할 수 있으며 NetworkManager 또는 블루투스 서비스를 통해 즉시 적용될 수 있습니다. 이는 ARGVUS 작업 표시줄 표시기 변경과 동일하지 않습니다.

제어 센터는 `--ask`를 사용하여 표준 입력을 통해 nmcli에 Wi-Fi 비밀번호를 전송합니다. 비밀번호는 프로세스 인수에서 제외되며 반환된 명령 오류에서 가려집니다.

## 명령줄 네트워크 관리

그래픽 인터페이스 없이 네트워킹을 쿼리하고 제어하려면 `argvus networkctl`를 사용하십시오:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## 커맨드 라인 블루투스 관리

블루투스 어댑터에는 `argvus bluetoothctl`을 사용하세요:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
커널이 `/sys/class/bluetooth`에서 컨트롤러를 나열하지 않으면, `status`는 bluetoothctl에 묻지 않고 즉시 `status=no-controller`에 응답합니다(서비스가 중지될 때 BlueZ를 기다리게 되는 경우와 달리). 작업 표시줄 모듈과 제어판 카드도 동일한 상태를 읽습니다.

설치된 버전에 특정한 추가 옵션과 하위 명령을 보려면 두 명령 중 하나에서  `bluetoothctl` `--help`를 사용하십시오.

Wi-Fi 비밀번호 대화 상자가 열려 있는 동안, 스캔 업데이트는 선택된 SSID와 보안 유형을 유지합니다. 해당 네트워크가 사라지거나 보안이 변경되면 확인이 오류와 함께 중단되고 새로운 선택이 필요합니다.

수동 DNS는 해결자 파일을 업데이트하기 전에 IPv4와 IPv6 서버를 해당 NetworkManager 설정으로 전송합니다. 서비스가 관리하는 해결자 심링크는 파일 교체를 거부하여 유지됩니다. 자동 DNS로의 성공적인 복귀는 백업을 복원하고 제거하므로, 이후 수동 사이클에서 현재 해결자 상태를 저장할 수 있습니다.

네트워크 프로필 편집기를 닫으면 현재 작업이 완료될 때까지 대기하므로, 편집기를 버려도 프로필 저장이나 삭제가 중단되지 않습니다. 백그라운드 새로 고침은 계속 취소할 수 있습니다.

Wi-Fi 연결 요청은 네트워크 동작 보호를 공유합니다: 이미 실행 중인 동작이나 프로필 편집기 작업이 완료된 후에야 다른 변경이 시작될 수 있습니다. 진행 및 완료 메시지는 선택한 언어로 표시됩니다.

블루투스 제거 확인은 장치 목록이 새로 고쳐져도 원래 장치 주소를 유지합니다. bluetoothctl에서 가져온 배터리 백분율이 인식되며, 장치 상세 조회는 제한된 동시 실행과 개별 시간 초과로 실행됩니다.

Wi-Fi 비밀번호, 수동 DNS 및 네트워크 검색 필드는 괄호로 묶인 붙여넣기를 허용합니다. 붙여넣은 컨트롤과 줄 바꿈은 생략됩니다. 지원되지 않는 Ctrl 전용 또는 Alt 전용 문자 단축키는 문자를 삽입하지 않습니다. 편집기는 Ctrl+Alt로 인코딩된 인쇄 가능한 AltGr 이벤트를 허용합니다; Ctrl+S는 Alt 없이 저장만 합니다. 공유 텍스트 편집기는 시각적 열에서 세로 탐색을 유지하고, 넓은 문자를 위해 가로 스크롤을 정렬합니다.
