---
title: Hyprland 통합
description: ARGVUS 합성기 구성 및 통합 계약.
---
`argvus-hyprland`는 `/usr/share/argvus/hyprland/config/hyprland.lua`, 키바인딩 및 헬퍼 스크립트를 패키징합니다. `argvus-session`는 Hyprland를 실행하고 세션 호환성 설정을 적용합니다; `argvus-config`는 Hyprland 테마, 레이아웃, 테두리 및 간격 파일을 `data/generated/hypr/`에 투사하며, `argvus-appearance`는 `hyprctl` 위의 컴포지터를 조정합니다; `argvus-display`는 모니터 오버라이드를 제공합니다.

레거시 패키지 경로는 현재 설치된 레이아웃의 일부가 아닙니다.
