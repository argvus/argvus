---
title: 기여하는
description: ARGVUS 프로젝트 및 문서의 기여 경계.
---
변경하기 전에 모든 소비 리포지토리에서 기능을 추적하세요. 공유 상태 계약, 패키지 소유권, 서비스 소유권 및 두 개의 로컬라이제이션 카탈로그를 유지하세요.

`argvus-config`는 `data/generated/`의 단일 작성자입니다. 소비자 파일을 추가하거나 변경하려면 `crates/argvus-config/src/project.rs`와 그 하위 모듈(`project/`), 그리고 `project/hyprland_projection.rs`의 Hyprland 파일에서 수행하세요. 쉘 헬퍼에서는 하지 마세요. 소비자 애플리케이션이 `config.json`를 읽을 수 없다면, 소유 패키지에 어댑터를 추가하고, 정식 커밋 후 오케스트레이터에서 호출하세요. `data/generated/`를 삭제하고, `argvus config apply`를 실행하여 `config.json`에서 바이트 단위로 트리가 다시 빌드되었는지 확인하세요. 소스, 패키지 페이로드 및 라이브 세션 동작을 각각 검증하세요; 소스 빌드만으로는 설치된 Wayland 세션이 새 파일을 사용하는지 증명할 수 없습니다.
