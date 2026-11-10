---
title: 세션 및 systemd
description: ARGVUS 세션 대상과 해당 사용자 서비스.
---
단위 정의는 `argvus-session`에 의해 `/usr/share/argvus/session/config/systemd/user/` 아래에 패키지됩니다. 세션 관리자는 수명 주기와 환경 가져오기를 소유합니다. 구성 요소 리포지토리는 해당 단위에서 실행되는 명령을 소유합니다.

통합 세션을 진단할 때 구성 요소를 수동으로 시작하는 대신 `argvus sessionctl`를 사용하십시오.
