---
title: 포장
description: ARGVUS Arch 패키지를 빌드하고 검사합니다.
---
각 구성 요소는 해당 리포지토리 내에서 자신의 Arch 패키징을 관리합니다. 페이로드를 문서화하기 전에 `packaging/archlinux/`, 패키지 기능 및 설치된 파일 목록을 확인하십시오. `/usr/share/argvus` 아래의 패키지 기본값과 `/etc/argvus` 아래의 관리자 구성은 서로 교체할 수 없습니다.

런타임 검증을 위해 패키지 내용을 `pacman -Ql <package>`와 비교하고 활성 사용자 재정의를 별도로 검사하십시오.

로컬 빌드는 Cargo 객체를 패키지 소스 트리 외부의 `build/cargo-target`에 보관하므로, 반복된 `make build`는 변경된 부분만 다시 컴파일합니다. `make clean`는 해당 디렉토리를 제거하며, 다음 빌드는 처음부터 시작됩니다.

리포지토리 이름만으로 런타임 페이로드를 추측하지 마십시오. 패키지가 바이너리, 사용자 서비스, `/usr/share/argvus` 기본값, `/etc/argvus` 관리자 구성 또는 자산을 소유할 수 있습니다. 설치 후 리포지토리의 `PKGBUILD`와 `pacman -Ql`를 통해 실제 페이로드를 확인하십시오.
