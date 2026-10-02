---
title: Session and systemd
description: The ARGVUS session target and its user services.
---

The unit definitions are packaged by `argvus-session` under `/usr/share/argvus/session/config/systemd/user/`. The session manager owns lifecycle and environment import. Component repositories own the commands executed by their units.

Use `argvus-sessionctl` instead of manually starting a component when diagnosing the integrated session.
