---
title: Developer Guide
description: Architecture and development guidance for ARGVUS contributors.
---

ARGVUS is a collection of independently packaged projects joined by shared configuration, systemd user services, Wayland/Hyprland contracts and common i18n.

## Start with ARGVUS Workflow

Before working on the ARGVUS infrastructure, read the [`README.md` in the ARGVUS Workflow repository](https://github.com/argvus/workflow). The same repository is mirrored at [GitLab](https://gitlab.com/argvus/workflow).

The workflow repository is the central Make-based workspace coordinator for ARGVUS. It creates and organizes the independent checkouts under `de/`, `web/` and `misc/`, builds Arch packages, collects the resulting artifacts under `builds/`, installs selected packages on an Arch host and provides workspace helpers. It is infrastructure for developing and assembling ARGVUS; it is not a replacement for the Makefile or build system owned by an individual project.

The usual workflow commands are:

```sh
make clone full
make build full
make collect
make install full
make status
make claude
```

Use `make clone de <project> [project...]`, `make build <project> [project...]` or `make install <project> [project...]` when working on a selected set of desktop projects. The `claude` target prepares the workflow checkout for Claude by creating its documented links to `AGENTS.md` and the skills directory. Installation invokes `pacman` and requires an Arch Linux host with the necessary privileges.

The cloned projects remain independent Git repositories. The workflow coordinates them but does not own their source history, package versions or project-specific dependencies. Its configurable variables and workspace setup rules are documented in its README; consult that document before changing the local infrastructure or adding a new project.

- [Architecture](/docs/developer-guide/architecture/overview/)
- [Subsystems](/docs/developer-guide/subsystems/session-and-systemd/)
- [Development](/docs/developer-guide/development/environment/)
- [Reference](/docs/reference/)

Source repositories under `de/` are authoritative. Package manifests and installed units define the runtime boundary; README files are supplementary.
