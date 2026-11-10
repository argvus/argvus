---
title: 제공자 및 서비스
description: 제공자 패키지와 그들의 시스템 인터페이스.
---
프로바이더 프로젝트는 UI 사용자를 위한 집중된 인터페이스를 제공합니다:

- `argvus network`과 `argvus bluetooth`는 NetworkManager와 Bluetooth 헬퍼를 통합합니다;
- `argvus-display`는 Hyprland와 모니터 상태를 통합합니다;
- `argvus-power`는 유휴, 잠금 및 전원 동작을 통합합니다;
- `argvus-firewall`는 `argvus-firewall.service`를 관리합니다;
- argvus의 `argvus removable-devices` 헬퍼는 UDisks2를 사용합니다;
- `argvus`의 `argvus control-center` `argvus`  헬퍼는 프로바이더 API와 시스템 D-Bus 인터페이스를 사용합니다.

제어 센터는 이러한 프로바이더의 클라이언트일 뿐, 그 대체물이 아닙니다.
