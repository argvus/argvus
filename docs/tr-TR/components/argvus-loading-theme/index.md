---
title: Yer paylaşımı yükleniyor
description: Oturum doğrudan yeniden yüklenirken gösterilen yer paylaşımı.
---

Yükleme katmanı, ARGVUS oturumu doğrudan yeniden yüklerken gösterilen GTK 4 katmanlı kabuk yüzeyidir
(`argvus sessionctl reload`). Bir tema anahtarı bunu kullanmaz: masaüstü yerinde değişir. `argvus` paketinde gönderilen bir yardımcıdır.
`/usr/lib/argvus/loading-theme/splash` ve `/usr/bin`'ya yerleştirilmiyor.

Oturum açma işlemi bunu kullanmaz: karşılama ekranı ve oturum başlatma döndürücü, Wayland modudur.
Aynı döndürücüyü GTK olmadan çizen ve içindeki ilk kareyi gösteren geçiş yardımcısı
milisaniye (bkz. [Karşılama, kilitleme ve sıçrama](/docs/developer-guide/subsystems/greeter-lock-and-splash/)).
`--spinner-only` ile kaplama yalnızca o döndürücüyü gösterir ve `SIGTERM` üzerinden çıkar. `--detached` ile onu başlatan işlemden çıkıldığında çıkılmaz; oturumun yeniden yüklenmesi her iki bayrağı da kullanır ve onu pid dosyası aracılığıyla durdurur.
Seçilen temayı ARGVUS veri dizininden okur ve paketlenmiş ARGVUS'a geri döner.
Karanlık palet. Hazır olma durumunu yalnızca eşlediği her yüzeyin ilk karesinden sonra bildirir;
süreç başlar.