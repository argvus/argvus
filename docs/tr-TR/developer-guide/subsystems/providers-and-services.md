---
title: Sağlayıcılar ve hizmetler
description: Sağlayıcı paketleri ve bunların sistem arayüzleri.
---

Sağlayıcı projeleri, odaklanmış arayüzleri kullanıcı arayüzü tüketicilerine sunuyor:

- `argvus network` ve `argvus bluetooth`, NetworkManager ve Bluetooth yardımcılarını entegre eder;
- `argvus-display`, monitör durumunu Hyprland ile entegre eder;
- `argvus-power` boşta kalma, kilitleme ve güç işlemlerini entegre eder;
- `argvus-firewall`, `argvus-firewall.service`'yu yönetir;
- argvus'daki `argvus removable-devices` yardımcısı UDisks2'yi kullanır;
- `argvus`'daki `argvus control-center` `argvus`  yardımcısı, sağlayıcı API'lerini ve sistem D-Bus arayüzlerini kullanır.

Kontrol Merkezi bu sağlayıcıların müşterisidir, onların yerine geçmez.