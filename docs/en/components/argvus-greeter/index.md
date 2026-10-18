---
title: Greeter
description: The login greeter that greetd starts, shipped as the argvus-greeter package.
---

`argvus-greeter` is the login greeter for greetd. It is a keyboard-first terminal
frontend that discovers the installed Wayland sessions, shows the accounts with their
avatars, and applies the active ARGVUS theme for the selected account.

It is the third split package of the `argvus` master package. Its source lives in the
`greeter/` directory of the `argvus` repository, as its own Cargo workspace, and the
package is built together with `argvus` and `argvus-gtk`. Installing `argvus` does not
require any install order.

The greeter does not authenticate users itself. greetd handles authentication through
PAM. The launcher opens the greeter with `argvus tui-terminal`, so the greeter does not
need a separate terminal program.

The greeter configuration files are installed under `/etc/argvus`. Use
`argvus greeter-setup --help` to see the setup options that the package provides.
For the login flow, see [Sessions](/docs/user-guide/sessions/).
