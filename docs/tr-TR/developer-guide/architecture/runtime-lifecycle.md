---
title: Çalışma zamanı yaşam döngüsü
description: Oturum başlatma ve kullanıcı hizmeti sahipliği.
---

`argvus-session` ortamı içe aktarır, Hyprland'ı `argvus start` aracılığıyla başlatır ve besteci hazır olduktan sonra `argvus-session.target`'yu başlatır. Hedef, kabuk, bildirim, duvar kağıdı, boşta kalma ve pano hizmetlerini çeker. `argvus sessionctl` başlatma, durdurma, yeniden yükleme, yeniden başlatma, durum ve günlükleri koordine eder.

Hyprland çıktığında oturum hedefi durdurur. Bireysel projeler kendi ikili dosyalarından ve yapılandırmalarından sorumlu olmaya devam eder.

## Başlangıç

Grafiksel giriş noktası `/usr/share/wayland-sessions/argvus.desktop`'dur ve `argvus session`'yu çağırır. Oturum, ARGVUS ve XDG yapılandırma ortamını içe aktarır, bunu `systemd --user` ve D-Bus aktivasyonuyla senkronize eder ve besteci başlatma yetkisini `argvus start`'ya devreder.

`argvus start`, Hyprland yapılandırmasını doğrular, stokta oluşturulan bir dosya bulunduğunda veya kullanıcı yapılandırması geçersiz olduğunda paketlenmiş geri dönüşü uygular, sanallaştırma uyumluluğunu algılar ve Hyprland'ı başlatır. Bestecinin hazırlık köprüsü daha sonra `argvus sessionctl ready`'yu çağırır ve bu, ARGVUS hedefini mevcut Wayland ve Hyprland ortamıyla başlatır.

Hedef, uzun süreli bileşenlere başlamadan önce tek seferlik bir hazırlık ünitesi kullanır. Hazırlık, oluşturulan konfigürasyonu, temayı ve vurgu durumunu, monitör durumunu, uygulama entegrasyonunu ve oturum ortamını uygular. Kalıcı hizmetler, geçici arka plan işlemlerine değil, `systemd --user`'ya aittir.

`argvus start`, Hyprland'ı başlatmadan hemen önce `argvus-config.service` (`--no-block`)'yu sıraya koyar: projeksiyonun birleştiriciye ihtiyacı yoktur, dolayısıyla birleştirici başlangıcıyla örtüşür. `argvus sessionctl ready` daha sonra hedefi başlatır ve ünitenin aktif olduğunu veya işini beklediğini tespit eder, böylece kanonik konfigürasyon hazırlık çalıştırmalarından önce hâlâ yansıtılır. Hedef başladıktan sonra, `sessionctl ready`, yükleme katmanını kaldırmadan önce duvar kağıdının (bir arka plan katmanı yüzeyi) ve görev çubuğunun eşlenmesi için 1,5 saniyeye kadar bekler; çünkü basit bir hizmet, çizilmeden önce başlatılmış sayılır; ünitesi çalışmayan bir yüzey beklenmez ve bekleme, her yüzeyin aldığı süre için `session.desktop-surfaces`, `session.wallpaper-surface` ve `session.taskbar-surface` ile günlüğe kaydedilir. Ölümcül bir projeksiyon veya hedef başlangıç ​​hatası günlüğe kaydedilir ve bağımsız yükleme katmanını durdurarak görünüşte sonsuz bir döndürücüyü önler. İsteğe bağlı görsel tüketiciler, oturum hedefini değiştirmeden bir geri dönüşü çözmeli veya bozulmuş bir hatayı bildirmelidir.

### Giriş maliyeti

Yapılandırma değişikliği olmayan bir oturum açma, pahalı bir işi tekrarlamaz:

- Projeksiyon, tema anahtarını ve efekt bağdaştırıcısını yalnızca görünüm veya efekt bölümleri değiştiğinde veya bunların gerçek çıktıları (gerçekleştirilmiş kullanıcı kopyaları ve `data/state/effects/<theme>.conf`) eksik olduğunda çalıştırır.
- Hazırlık, yalnızca `$XDG_CACHE_HOME/argvus/session/theme-startup.stamp`'daki damga artık paketlenmiş tema dosyalarıyla ve vurgu işaretleyiciyle eşleşmediğinde başlangıç ​​temasını ve onun üzerindeki vurguyu uygular. Damga parmak izi dosya içeriği, dolayısıyla aynı dosyaların yeniden yüklenmesi onu geçersiz kılmaz. Değiştirilen tema girişleri veya yeni bir aksan, onu bir kez geçersiz kılar. Daha eski bir zaman damgası tabanlı damganın taşınması da bir kez yeniden oluşturulur. Tema yeniden uygulandığında hazırlık, önceki dosyalarda başlatılan Hyprland'ı da yeniden yükler.
- GTK ayarları (`gsettings`) yalnızca başlangıç ​​teması yeniden uygulandığında yazılır. Oturum açma işlemleri arasında dconf bunları tutar, böylece kararlı bir oturum açma işlemi bunları tekrar çalıştırmaz.
- `argvus-dunst.service`, `Type=dbus`'dur: Dunst, `org.freedesktop.Notifications`'ya sahip olduğunda başlatılmış sayılır, dolayısıyla oturum hedefi duraklatma durumunun geri yüklenmesini beklemez.
- Hazırlık, `spaces-switch.sh` ve `borders-switch.sh` (`--apply-static`) yan yana çalışır: ayrık dosyalar yazar (çubukların JSONC kenar boşlukları ve etkili geometrisi ve çubukların CSS'si ve Rofi teması).- Projeksiyon, oluşturulan dosyayı yalnızca içeriği değiştiğinde yeniden yazar. Değiştirilmemiş dosyalar olduğu gibi bırakılır, bu nedenle kararlı bir oturum açma işlemi onları yeniden fsync'lemez.
- Hazırlığın görüntüleme adımı (`argvus config display session-prepare`) yalnızca DPMS'yi ve devre dışı bırakılan çıkışları uygular: ne yeniden yansıtır ne de birleştiriciyi yeniden yükler.
- `argvus-session-reload.service`, Hyprland'ı yalnızca projeksiyon bildiriminde etkilenen tüketiciler arasında hyprland listelendiğinde yeniden yükleyen `argvus sessionctl reload-desktop --login`'yu çalıştırır. Masaüstü bileşenleri bunu beklemez.

Oturum döndürücü (Wayland modundaki aktarım yardımcısı), birleştirici aktarımdan  `hyprland` `argvus sessionctl ready` hedefi başlatana kadar eşlenir, böylece tek bir döndürücü tüm başlangıcı kapsar. `PartOf=`'su yoktur, çünkü yeni bir oturum açma sırasında hedefin yeniden oluşturulması onu kaldırmamalıdır; `sessionctl` bunu açıkça durdurur ve `RuntimeMaxSec=30` herhangi bir durak gelmezse bunu sonlandırır.

Her hazırlık, yeniden yükleme ve projeksiyon bağdaştırıcı adımı, süresini `<epoch-ms> timing <step> <N>ms` olarak oturum günlüğüne ekler; bkz. [oturum başlatma sorunlarını giderme](/docs/user-guide/troubleshooting/session-startup/).

Yol çözümü, oturum hazırlığı ve Hyprland yeniden yükleme planı Rust'a aittir.
`argvus config session-prepare` ve `argvus config session-reload-config` çalıştırır
başlatma ve yeniden yükleme adımları; eski `hypr-init.sh` betiği kaldırıldı.

## Hizmetler ve kapatma

Hedef genellikle etkinleştirildiğinde `argvus-control-panel.service`, `argvus-taskbar.service`, `argvus-widget-telemetry.service`, `argvus-wallpaper.service`, `argvus-dunst.service`, `argvus-hypridle.service`, pano izleyicileri, klavye düzeni bildirimleri ve PolicyKit entegrasyonu. Oturum yükleme katmanı hedef tarafından değil `argvus start` tarafından başlatılır. İsteğe bağlı bir hizmetteki arıza tüm oturumun yerine geçmez.

Kontrol Paneli ve boşta yönetici de dahil olmak üzere Wayland istemci bağlantısına ihtiyaç duyan hizmetler, istemcilerini başlatmadan önce kullanılabilir bir Hyprland çalışma zamanı bekler. Bu, besteci aktarımı sırasında eski bir yuvaya karşı Quickshell veya hipridle'ın başlatılmasını önler; Hyprland değiştirilirse, yeni çalışma zamanı kullanılabilir hale geldikten sonra systemd bileşeni yeniden başlatabilir.

Bir tema anahtarı, taahhüt etme, oluşturma, projeksiyon ve tüketici yeniden yüklemelerini tema kilidi altında tek bir yaşam döngüsü işlemi olarak serileştirir. `argvus-config.service`'yu yeniden yüklemez veya hizmetleri yeniden başlatmaz: çalıştırdığı projeksiyon yalnızca dosyaları yazar ve kilit tutulduğunda temayı anahtara bırakır ve çalışan tüketiciler yerinde yeniden yükleme yapar (bkz. [Görünüm ve temalar](/docs/developer-guide/subsystems/appearance-and-themes/)).

Hyprland çıktığında `argvus start`, `argvus-session.target`'yu ve ona bağlı hizmetleri durdurur. Bu, görev çubuğu, kabuk, bildirim, boşta kalma ve pano işlemlerinin oturum kapatmadan sağ çıkmasını engeller.

## Yeniden yükleniyor

```sh
argvus sessionctl reload
```

Yeniden yükleme, grafik ortamı yeniden içe aktarır, ardından `apply-theme`, `accent`, `set`/`patch` gibi önceki bir etki alanı komutu veya bir widget-telemetri blok değişikliği değişikliği zaten yansıttığında taahhütlü bir oluşturma **uygular**. Bu durumda bildirim planında listelenen tüketicileri geri yükler ve `argvus config project`'yu tekrar çağırmaz çünkü yeniden projelendirme aynı ağacı yeniden oluşturup planı iptal eder. Projeksiyon, geçiş ve `ensure` yalnızca geçerli bir bekleyen plan mevcut olmadığında, oturum başlangıcında veya `argvus config recover`'nun önceki belgeyi geri yüklemesi gerektiğinde gerçekleştirilir. Temiz bir işlem yapılmamasına yalnızca projeksiyonların ve yönetilen çalışma zamanının her ikisinin de güncel olması durumunda izin verilir. Kurallı durum değişmediyse ancak bir besteci soketi, hizmeti veya tüketicisi eskiyse, yeniden yükleme erken geri dönmek yerine onu uzlaştırır. Normal yayma, projeksiyon bildirimine göre Hiprland'ı, görev çubuğunu, widget telemetrisini, Kontrol Panelini, bildirimleri, duvar kağıdını ve boşta kalmayı kapsarken, değişmeyen isteğe bağlı tüketicilere dokunulmaz. `data/generated/` altındaki her dosya bu noktada `argvus-config` tarafından yazılır, dolayısıyla oturum hiçbir zaman bir temanın kendisini yeniden oluşturmaz. Daha sonra yeniden yükleme, standart belgeyi veya yayınlanan projeksiyonu kendi başlarına okuyamayan tüketicileri uzlaştıran `theme-adapters.sh`'yu çalıştırır. Bu bağdaştırıcılar yalnızca taahhüt edilen nesli okur: kendi tüketici dosyalarını ve önbelleklerini yazarlar ve hiçbir zaman standart belgeyi veya `data/generated/`'yu yazmazlar, dolayısıyla tek bir kullanıcı eylemi hala bir mutasyon, bir projeksiyon, bir yayınlama ve bir yeniden yüklemeye mal olur. `argvus sessionctl restart <component>`'yu yalnızca sahip olunan tek bir hizmetin açıkça yalıtılmış bir yeniden başlatmaya ihtiyacı olduğunda kullanın. Kesin ünite adları için [hizmet referansını](/docs/reference/services/) inceleyin.

Projeksiyon ve yeniden yükleme işlemleri, kullanıcı çalışma zamanı kilidiyle serileştirilir. Geçici bir zamanlama takibi için, yeniden yüklemeyi başlatmadan önce `ARGVUS_SESSION_PERF=1`'yu ayarlayın ve `~/.local/state/argvus/session.log`'yu inceleyin; izleme, normal yeniden yükleme davranışını değiştirmeden aşama sınırlarını kaydeder.