---
title: Greeter
description: The login greeter that greetd starts, shipped in the argvus package.
---

`argvus-greeter` is the login greeter for greetd. It is a keyboard-first terminal
frontend that discovers the installed Wayland sessions, shows the accounts with their
avatars, and applies the active ARGVUS theme for the selected account.

It is part of the `argvus` package, the single package of the desktop. Its source lives
in `crates/argvus-greeter` of the `argvus` repository, as a member of the root Cargo
workspace, and it is built together with the rest of `argvus`.

The greeter does not authenticate users itself. greetd handles authentication through
PAM. The launcher opens the greeter with `argvus tui-terminal`, so the greeter does not
need a separate terminal program.

The greeter configuration files are installed under `/etc/argvus`. Use
`argvus greeter-setup --help` to see the setup options that the package provides.
For the login flow, see [Sessions](/docs/user-guide/sessions/).
