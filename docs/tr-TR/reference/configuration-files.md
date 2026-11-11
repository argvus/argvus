---
title: Yapılandırma dosyaları
description: Kullanıcı, sistem, paket varsayılanı ve oluşturulan ARGVUS yapılandırması.
---

| Katman | Konum | Anlamı |
| --- | --- | --- |
| Sistem | `/etc/argvus/` | Hizmetler ve sağlayıcılar için yönetici yapılandırması. |
| Paket varsayılanları | `/usr/share/argvus/` | Salt okunur bileşen varsayılanları ve çalışma zamanı varlıkları. |
| Kullanıcı | `$XDG_CONFIG_HOME/argvus/config.json` | Kanonik taşınabilir tercihler. |
| Yönetilen veriler | `$XDG_CONFIG_HOME/argvus/data/` | Bileşen verileri, uyumluluk dosyaları ve dahili meta veriler. |
| Oluşturuldu | `$XDG_CONFIG_HOME/argvus/data/generated/` | Yalnızca argvus-config tarafından yazılan `config.json` projeksiyonları. Asla düzenleme yapmayın, asla konfigürasyon olarak başka bir yere kopyalamayın ve asla kaynak olarak tekrar okumayın. |
| Çalışma zamanı |  `argvus-config` `$XDG_RUNTIME_DIR/` ve `$XDG_CACHE_HOME/` | Soketler, kilitler ve geçici durum. |

Etkin tema, vurgu, özel duvar kağıdı ve efekt durumu mantıksal kullanıcı durumudur. Yönetilen mantıksal model [Kanonik yapılandırma](/docs/reference/argvus-config/) belgesinde belgelenmiştir. Oluşturulan Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminal, yazı tipi ve çıkarılabilir aygıt dosyaları, bağımsız gerçek kaynakları olarak değerlendirilmemelidir.

`argvus-config` ayrıca tema katmanını `data/waybar/argvus-taskbar.{jsonc,css}` ve `data/waybar/argvus-widget-telemetry.{jsonc,css}`'ya yansıtır. Bu kopyalar, bir görünüm değişikliği üzerine paketlenmiş varsayılanlardan değiştirilir, ardından sınırlandırılmış yönetilen blokları yeniden yazılır; bu nedenle, bunları elle tutulan dosyalar yerine türetilmiş dosyalar olarak ele alın. `$XDG_CONFIG_HOME/<app>` kapsamındaki yerel uygulama yapılandırması farklıdır: bu, görünüm orkestratörleri tarafından yazılan bir bağdaştırıcı hedefidir, asla projektör tarafından yazılmaz.

## Tek yazar kuralının istisnası

`argvus config`, `data/generated/`'nun tek yazarıdır. Geçiş için bir istisna mevcuttur:
Paketlenmiş bir dosyanın kullanıcı kopyası oluşturulduğunda, eski sürümün altında bırakılan bir dosya
`data/generated/` kullanıcı konumuna (`data/`) taşınır, böylece kullanıcı düzenlemelerini korur.
taşıma yalnızca geçerli sürümün oluşturmadığı dosyalar için gerçekleşir. Buna güvenme
yeni dosyalar için yol: oluşturulan dosyayı buraya değil projeksiyona ekleyin.