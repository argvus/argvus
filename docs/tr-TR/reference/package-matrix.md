---
title: Paket matrisi
description: ARGVUS paketinin sorumlulukları.
---

| Paket | Sorumluluk |
| --- | --- |
| `argvus` | Ana paket: dağıtıcı, süreç içi komutlar (yapılandırma, çeviriler, anahtarlar, projeler, terminal, sistem monitörü, ağ, Bluetooth, telemetri, TUI terminali, kurulum), paylaşılan veriler ve i18n katalogları. Her bileşen paketi buna bağlıdır. |
| `argvus` (yardımcılar) | Yardımcılar aynı pakette: Kontrol Merkezi ve Disk Kullanım Analizörü (terminal uygulamaları), ayrıca görev çubuğu takvimi için GTK 4 uygulamaları, çıkarılabilir cihazlar, yükleme katmanı, ARGVUS Focus ve ARGVUS Clock. |
| `argvus` (selamlayan) | Selamlama için giriş selamlayıcı, aynı pakette gönderilir ve `crates/argvus-greeter`'dan üretilmiştir. |
| `argvus-desktop` | `argvus` ana paketinin gerektirdiği masaüstü ve günlük uygulama profili. `argvus`'ya bağlı değildir, dolayısıyla bağımlılık tek yönlü kalır. |
| `argvus-devtools`, `argvus-pentest` | Geliştirme araçları ve yetkili güvenlik denetimi için isteğe bağlı bağımsız meta paketler. Ana paket bu profilleri yüklemez. |
| `argvus-session` | Oturum giriş noktaları ve kullanıcı yaşam döngüsü. |
| `argvus-hyprland` | Hyprland yapılandırması. |
| `argvus-control-panel` | Quickshell kontrol paneli. |
| `argvus-taskbar`, `argvus-waybar` | Waybar konfigürasyonu ve yamalı ikili. |
| `argvus-branding` | Logo ve kelime işareti varlıkları. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Görünüm durumu ve görsel varlıklar. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Masaüstü sağlayıcıları. |
| `argvus-lock`, `argvus-boot-splash` | Kilit ekranı ve önyükleme sıçraması. |
| `argvus-firewall` | Güvenlik duvarı, kendi ayrıcalıklarına sahiptir. |
| `argvus-launcher` | Başlatıcı yapılandırması ve giriş noktaları. |
| `argvus-app-profiles` | Paketlenmiş terminal uygulamaları için yalnızca veri profilleri. |
| `argvus-portal` | XDG portalı ve Wayland/DBus varsayılanları. |
| `argvus-theme-*` | Resmi tema ailesi başına bir isteğe bağlı paket. |
| `argvus-game-snake`, `argvus-games` | İsteğe bağlı. Snake bağımsız bir projedir; `argvus-games` isteğe bağlı oyunları gruplandırır. |

## Değiştirilen paketler

`argvus`, emdiği paketler için `replaces`, `conflicts` ve `provides`'yu bildirir; böylece bir yükseltme bunları kaldırır ve dosyaları yetim olarak kalmaz:

`argvus-config`, `argvus-i18n` (`argvus-language-en-us` ve `argvus-language-pt-br` ile), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` ve `argvus-du`. `argvus` ayrıca `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices` ve önceki bölünmüş paketler `argvus-gtk` ve yerine geçer. `argvus-greeter`, onlara bağlı paketlerin çözümlenmeye devam etmesi için hala sağlıyor. Hesap yöneticisi (`argvus accounts`), `crates/argvus-accounts`'dan oluşturulmuş ve polkit kuralıyla `/usr/lib/argvus/commands/accounts` olarak yüklenmiştir. Disk Kullanım Analizörü `crates/argvus-du`'dan oluşturulmuş ve `/usr/lib/argvus/commands/argvus-du` olarak yüklenmiştir.

`/usr/bin/argvus-terminal`, `argvus terminal` için uyumluluk başlatıcısı olarak yüklü kalır. Hyprland'ın varsayılan terminali, Kontrol Merkezi kataloğu ve kayıtlı varsayılan uygulamalar hala bu komutu adlandırmaktadır.

Kaldırılan `argvus-pwvucontrol` ve `argvus-snappy-switcher` paketleri hiçbir paket tarafından sağlanmaz.

ARGVUS bileşen paketleri arasında kurulum sırası gerekli değildir: `argvus` onların ana paketidir ve her bileşen ona bağlıdır. `argvus`'yu yüklemek aynı zamanda `argvus-desktop` profilini de yükler; `argvus-devtools` ve `argvus-pentest` tercihli olmaya devam ediyor. Bir paketin bu tabloda bulunması, özelliğin tamamına sahip olduğu anlamına gelmez: örneğin, görev çubuğu `argvus-waybar`, `argvus-taskbar`, `argvus-session` ve etki alanı sağlayıcılarını birleştirir.