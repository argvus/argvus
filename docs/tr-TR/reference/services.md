---
title: Hizmetler
description: ARGVUS sistem ve kullanıcı hizmetleri.
---

Ana kullanıcı hedefi `argvus-session.target`'dur. Yaygın kullanıcı hizmetleri arasında `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, pano hizmetleri ve `argvus-session-loading.service`.

Güvenlik duvarı, `argvus-firewall.service` adlı bir sistem hizmetidir. Takvim paketi, `argvus-taskbar-calendar.service`'yu bir kullanıcı birimi olarak yükler ve `argvus-clock.service` (varsayılan olarak devre dışıdır), **Oturumla başlat** açık olduğunda oturum açtıktan sonra ARGVUS Saatini gizli olarak başlatır.

Aktif üniteleri aşağıdakilerle inceleyin:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## Oturuma ait birimler

Oturum paketi, bu kullanıcı birimlerini `/usr/lib/systemd/user` altına yükler (kaynak verisi, oturum paketinin paylaşılan yapılandırma ağacının altındadır):

| Birim | Rol |
| --- | --- |
| `argvus-session.target` | ARGVUS grafik oturumu için yaşam süresi sınırı. |
| `argvus-session-prepare.service` | Tek seferde başlatma hazırlığı ve oluşturulan yapılandırma. |
| `argvus-control-panel.service` | Quickshell Kontrol Paneli. |
| `argvus-taskbar.service` | Ana ARGVUS görev çubuğu. |
| `argvus-widget-telemetry.service` | İsteğe bağlı telemetri/sysinfo yüzeyi. |
| `argvus-hyprland.service` | `hyprctl reload` çalıştıran isteğe bağlı tek seferlik ünite. |
| `argvus-config.service` | Kurallı yapılandırmayı yansıtır (`argvus config project`); yeniden yükleme `argvus sessionctl reload-config-transaction`'yu çalıştırır. |
| `argvus-session-reload.service` | Oturum açmayı yeniden yükleme (`argvus sessionctl reload-desktop --login`): Hyprland'ı yalnızca projeksiyon onu etkilediğinde yeniden yükler. |
| `argvus-ssh-agent.service` | Oturum için SSH aracısı. |
| `argvus-secret-service.service` | Gizli servisi `argvus sessionctl start-secret-service` aracılığıyla başlatır. |
| `argvus-wallpaper.service` | Aktif duvar kağıdı arka ucu. |
| `argvus-dunst.service` | Dunst bildirim arka plan programı. |
| `argvus-hypridle.service` | Boşta kalma ve kilitleme politikası. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Pano geçmişi gözlemcileri. |
| `argvus-keyboard-layout.service` | Klavye düzeni bildirimleri. |
| `argvus-polkit.service` | PolicyKit aracı entegrasyonu. |
| `argvus-session-loading.service` | Birleştirici aktarımından masaüstü bileşenleri başlayana kadar oturum yükleme katmanı. |

`argvus-blueman-applet.service` ayrıca oturum paketiyle de gönderilir. Bazı birimler isteğe bağlıdır veya koşullu olarak başlatılır; hepsi her oturumda aktif değildir.

Bir birimin yalnızca kaynak kontrolünden ziyade etkili tanımını incelemek için:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

Sistem düzeyi `argvus-firewall.service`, `argvus-firewall`'ya aittir. Takvim paketi `argvus-taskbar-calendar.service`'yu kullanıcı birimi olarak ayrı olarak gönderir. `argvus-clock.service`, `argvus` paketinin bir kullanıcı birimidir (`WantedBy=graphical-session.target`), ARGVUS Saat ayarlarından etkinleştirilir.

## Oturum yönetimi komutları

`argvus sessionctl` komutu, oturum yaşam döngüsünü yönetir ve belirli bileşenleri oturumu kapatmadan yeniden başlatabilir:

### Oturum yaşam döngüsü

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### Bileşene özgü yeniden başlatmalar

Tüm oturumu etkilemeden bireysel hizmetleri yeniden başlatın:

```sh
# Desktop surfaces
argvus sessionctl restart waybar
argvus sessionctl restart wallpaper
argvus sessionctl restart shell

# Notifications and switcher
argvus sessionctl restart dunst

# Clipboard and input
argvus sessionctl restart clipboard keyboard-layout

# Bluetooth applet (optional)
argvus sessionctl restart blueman-applet
```

### Ortam senkronizasyonu

Ortam değişkenlerini oturuma aktarın:

```sh
argvus sessionctl import-environment
```

Bu, kabuk ve sistemd kullanıcı hizmetleri arasında Wayland, Hyprland, Qt, imleç ve DBus değişkenlerini senkronize eder.

## Hizmet birimleri referansı

Ek kullanıcı birimleri doğrudan `systemctl --user` ile yönetilebilir:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

İlk oturum önyüklemesine ilişkin günlükler şu konumda saklanır:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
