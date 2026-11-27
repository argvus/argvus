---
title: Kabuk yüzeyleri
description: Görev çubuğu, kontrol paneli ve telemetri sahipliği.
---

`argvus-shell`, yamalı `Quickshell Taskbar`'yu sağlar. `argvus-shell`, görev çubuğu eylemlerine ve kullanıcı deneyimine sahiptir; Gönderdiği görev çubuğu JSONC ve CSS, `config.json`'nun projeksiyonlarıdır ve `argvus-config`, `data/generated/taskbar/` altındaki aktif katmanı artı `data/taskbar/` altında yansıtılmış bir kopyayı yazar. `argvus-shell`, Quickshell kartlarına sahiptir. `argvus-shell` provides the optional Telemetry surface, while the `argvus widget-telemetry` command in `argvus` retains block-control semantics; JSONC ve CSS'si, bir görünüm değişikliğinde paketlenmiş varsayılanlardan `data/taskbar/`'ya yazılır ve bunun ardından `argvus-config` yalnızca sınırlandırılmış `ARGVUS_TELEMETRY_*` ve yazı tipi bloklarını yeniden yazar. `argvus-session` kullanıcı hizmetlerinin sahibidir.

Bu çalışma zamanı modelinde güncel `argvus-shell` paketi veya `argvus-shell.service` yok.