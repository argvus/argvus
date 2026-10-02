---
title: Translations
description: Maintain shared ARGVUS localization catalogs.
---

`argvus-i18n` owns the shared catalogs and QML module. Update the English and Portuguese catalogs together when changing user-visible text, then run:

```sh
argvus-i18n validate
```

Consumers include native Rust applications, shell components and session helpers.
