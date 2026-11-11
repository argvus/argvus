---
title: Kabuk yüzeyleri
description: Görev çubuğu, kontrol paneli ve telemetri sahipliği.
---

`argvus-waybar`, yamalı `/usr/bin/waybar`'yu sağlar. `argvus-taskbar`, görev çubuğu eylemlerine ve kullanıcı deneyimine sahiptir; Gönderdiği görev çubuğu JSONC ve CSS, `config.json`'nun projeksiyonlarıdır ve `argvus-config`, `data/generated/waybar/` altındaki aktif katmanı artı `data/waybar/` altında yansıtılmış bir kopyayı yazar. `argvus-control-panel`, Quickshell kartlarına sahiptir. `argvus-widget-telemetry`, isteğe bağlı telemetri çubuğuna ve onun blok semantiğine sahiptir; JSONC ve CSS'si, bir görünüm değişikliğinde paketlenmiş varsayılanlardan `data/waybar/`'ya yazılır ve bunun ardından `argvus-config` yalnızca sınırlandırılmış `ARGVUS_TELEMETRY_*` ve yazı tipi bloklarını yeniden yazar. `argvus-session` kullanıcı hizmetlerinin sahibidir.

Bu çalışma zamanı modelinde güncel `argvus-shell` paketi veya `argvus-shell.service` yok.