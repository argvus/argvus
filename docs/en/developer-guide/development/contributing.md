---
title: Contributing
description: Contribution boundaries for ARGVUS projects and documentation.
---

Trace a feature through all consuming repositories before changing it. Preserve shared state contracts, package ownership, service ownership and both localization catalogs.

`argvus-config` is the single writer of `data/generated/`. Add or change a consumer file there, in `argvus-config/src/project.rs`, not in a shell helper. If a consumer application cannot read `config.json`, add an adapter in the owning package and call it from the orchestrator after the canonical commit. Delete `data/generated/`, run `argvus-config apply` and confirm the tree is rebuilt byte-for-byte from `config.json`. Validate source, package payload and live-session behavior separately; a source build alone does not prove that an installed Wayland session uses the new files.
