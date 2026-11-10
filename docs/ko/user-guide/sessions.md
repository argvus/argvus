---
title: 세션
description: ARGVUS 그래픽 세션을 시작하고 관리합니다.
---
`argvus-session`는 세션 진입점과 `systemd --user` 라이프사이클을 소유합니다. 이는 `argvus start`을 통해 Hyprland를 시작하고 세션 환경을 가져오며 ARGVUS 대상을 관리합니다.

- [그래픽 세션](/docs/argvus-session/graphical-session/)
- [TTY 세션](/docs/argvus-session/tty-session/)
- [그리터](/docs/components/argvus-greeter/)

유용한 진단:

```sh
argvus sessionctl status
argvus sessionctl logs
argvus sessionctl restart waybar
```
서비스 소유권에 대해서는 개발자 [runtime lifecycle](/docs/developer-guide/architecture/runtime-lifecycle/)]를 참조하세요.
