---
title: 첫 번째 세션
description: 설치 후 ARGVUS를 시작하십시오.
---
그래픽 로그인용으로는, 패키지화된 greetd 통합을 `argvus greeter-setup --help`과 함께 구성하고 [Sessions](/docs/user-guide/sessions/)에 설명된 display-manager 흐름을 사용하십시오.

TTY 세션의 경우, 가상 터미널에서 로그인하고 다음을 실행하십시오:

```sh
argvus --start-desktop
```
로그인 후, ARGVUS 사용자 대상을 검사하려면 `argvus sessionctl status`를 사용하세요. 컴포지터나 셸이 시작되지 않으면 [session troubleshooting](/docs/user-guide/troubleshooting/session-startup/)]를 따르세요.
