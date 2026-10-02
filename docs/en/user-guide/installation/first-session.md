---
title: First session
description: Start ARGVUS after installation.
---

For a graphical login, configure the packaged greetd integration with `argvus-greeter-setup --help` and use the display-manager flow described in [Sessions](../sessions/).

For a TTY session, log in on a virtual terminal and run:

```sh
argvus-tty
```

After login, use `argvus-sessionctl status` to inspect the ARGVUS user target. If the compositor or shell does not start, follow [session troubleshooting](../troubleshooting/session-startup/).
