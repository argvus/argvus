---
title: Dosya konumları
description: Önemli yüklü ARGVUS dosyaları ve dizinleri.
---

- `/usr/bin/argvus*` — genel komutlar.
- `/usr/share/argvus/hyprland/` — Hyprland yapılandırması ve yardımcıları.
- `/usr/share/argvus/control-panel/` — Kontrol Paneli QML'si ve komut dosyaları.
- `/usr/share/argvus/taskbar/` — görev çubuğu yapılandırması ve komut dosyaları.
- `/usr/share/argvus/lock/` — Hyprlock yapılandırması.
- `/usr/share/argvus/portal/` — portal entegrasyon dosyaları.
- `/usr/share/backgrounds/argvus/` — duvar kağıdı varlıkları.
- `/etc/argvus/` — sistem sağlayıcı yapılandırması.
- `/etc/argvus/greeter.toml` — karşılayıcı yapılandırması.
- `/etc/argvus/removable-devices/config.json` — çıkarılabilir depolama için sistem varsayılanları.
- `$XDG_CONFIG_HOME/argvus/config.json` — standart taşınabilir tercihler.
- `$XDG_CONFIG_HOME/argvus/data/` — yönetilen bileşen verileri, yedeklemeler, kilitler ve oluşturulan çıktı.
- `$XDG_CONFIG_HOME/waybar/argvus-taskbar.{jsonc,css}` — isteğe bağlı eksiksiz yerel Waybar görev çubuğu geçersiz kılmaları.
- `$XDG_CONFIG_HOME/argvus/data/taskbar/argvus-taskbar.{jsonc,css}` — yönetilen ARGVUS görev çubuğu kopyaları.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/` — `argvus-taskbar.css` ve `argvus-widget-telemetry.{jsonc,css}` dahil olmak üzere oluşturulan Waybar profilleri; bunlar yukarıdaki kopyalara göre önceliklidir.
- `$XDG_CONFIG_HOME/argvus/data/generated/waybar/argvus-widget-telemetry.{jsonc,css}` — aynı zamanda `data/waybar/` altında yansıtılır; bu, bir görünüm değişikliğinde paketlenmiş varsayılandan değiştirilir ve daha sonra `ARGVUS_TELEMETRY_*` ve yazı tipi blokları yeniden yazılır.
- `$XDG_CONFIG_HOME/argvus/data/generated/yazi/` — Yazi tadı sağlamayan temalar için değişkenlere duyarlı bir geri dönüş içeren öngörülen Yazi yapılandırma ağacı.
- `$XDG_CONFIG_HOME/argvus/data/generated/superfile/` — öngörülen Superfile tema seti.
- `$XDG_CONFIG_HOME/argvus/data/generated/terminal/` — öngörülen terminal profilleri.
- `$XDG_CONFIG_HOME/argvus/data/generated/qt6ct/` — yansıtılan Qt6ct paleti.
- `$XDG_CONFIG_HOME/argvus/data/generated/gtk/` — öngörülen GTK tema dosyaları.
- `$XDG_CONFIG_HOME/argvus/data/rofi/` — Rofi yapılandırmasının, temalarının ve modunun kullanıcı kopyaları. ARGVUS, dosyayı ilk değiştirdiğinde her kopyayı oluşturur; değiştirilmemiş dosyalar `/usr/share/argvus/launcher/config/`'dan okunur.
- `$XDG_CONFIG_HOME/argvus/data/generated/removable-devices/theme.css` — öngörülen çıkarılabilir aygıtlar stil sayfası.
- `$XDG_CONFIG_HOME/argvus/data/focus/settings.json`, `$XDG_DATA_HOME/argvus/focus/history.json`, `$XDG_STATE_HOME/argvus/focus/session.json` — ARGVUS Odak ayarları, oturum geçmişi ve mevcut oturum kontrol noktası.
- `$XDG_RUNTIME_DIR/argvus/focus/status.json` — görev çubuğu tarafından tüketilen geçici ARGVUS Odak durumu; Focus çıktığında kaldırılır.
- `$XDG_CONFIG_HOME/argvus/data/clock/settings.json`, `$XDG_DATA_HOME/argvus/clock/alarms.json`, `$XDG_STATE_HOME/argvus/clock/timers.json` — ARGVUS Saat ayarları ve dünya saatleri, alarmlar ve zamanlayıcı/kronometre kontrol noktası.
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/current/time/theme.css` — ARGVUS Focus ve ARGVUS Clock'un renkleri (tema tüketicisi `time`).
- `/usr/share/argvus/sounds/` — ARGVUS Focus ve ARGVUS Clock sesleri (`argvus-chime.oga`, `argvus-beacon.oga`, `argvus-bell.oga`).
- `$XDG_CONFIG_HOME/argvus/data/generated/theme/` — argvus theme render tarafından ve her tema değişikliği ve vurgu değişikliği ile yazılan tema yapıları (`builds/<id>-<variant>-<hash>/`) ve aktif temanın yapısına  `argvus theme render` `current` sembolik bağlantısı.
- `$XDG_CONFIG_HOME/argvus/data/hypr/` — kullanıcı Hyprland Lua geçersiz kılmaları ve yerel Hyprland projeksiyonları.
- `$XDG_CONFIG_HOME/hypr/hyprland.lua` — `argvus start` tarafından seçilen isteğe bağlı eksiksiz yerel Hyprland yapılandırması; bu, artan bir kaplama değildir.
- `$XDG_STATE_HOME/argvus/config-projection.json` — projeksiyon bildirimi: oluşturma, etkili yapılandırma karması, değiştirilen bölümler ve bekleyen çalışma zamanı çalışması. `argvus sessionctl reload` tarafından okundu; asla gerçeğin kaynağı değildir.
- `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` — eski telemetri bloğu tercihleri. Yalnızca geçiş girişi; kanonik değerler `config.json`'da bulunur ve bu dosya artık yazılmaz.
- `$XDG_STATE_HOME/argvus/session.log` — grafiksel oturum günlüğü (normalde `~/.local/state/argvus/session.log`).
- `/usr/share/wayland-sessions/argvus.desktop` — grafiksel oturum girişi.
- `/usr/share/plymouth/themes/argvus/` — paketlenmiş önyükleme açılış varlıkları.

Tam yükler pakete özeldir; geliştirici bileşeni referansına ve ilgili paket bildirimine bakın.