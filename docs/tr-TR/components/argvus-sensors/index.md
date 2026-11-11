---
title: Sıcaklık sensörleri
description: Lm_sensors JSON tarafından desteklenen canlı sıcaklığa genel bakış.
---

`argvus sensors`, görev çubuğu CPU sıcaklığı açılır penceresi tarafından kullanılan işlem içi Ratatui panelini açar. `sensors -j`'yu çalıştırır, okumaları donanım tarafından bildirilen çip adlarına ve etiketlere göre gruplandırır ve her iki saniyede bir yenilenir. GPU izleme `argvus system-monitor --gpu` tarafından gerçekleştirilir.

Okumalar Celsius'u kullanır. Panel, `max`, `crit` ve lm_sensors bunları sağladığında alarm durumunu gösterir. Bir gösterge yalnızca pozitif bir limit mevcut olduğunda gösterilir. Eksik `lm_sensors`, komut hataları, geçersiz JSON ve boş okumalar kurtarılabilir durumlar olarak görünür durumda kalır; bir sonraki döngüde yoklama yeniden denenir.

Kapatmak için `q` veya `Esc` tuşuna basın. Görev çubuğu Kitty'deki paneli `argvus tui-terminal` aracılığıyla açar; yerelleştirilmiş alt sekme başlığı yakın ipucunu gösterir. Bu komut CPU sıcaklığı ayrıntılarına sahiptir; GPU sıcaklığı simgesi yalnızca GPU'ya ait btop monitörünü açar. Görev çubuğu paketi zaten `lm_sensors`'ya bağlıdır, dolayısıyla ayrı bir pakete veya bağımlılığa gerek yoktur.