---
title: Services
description: ARGVUS systemd and user services.
---

Target pengguna utama adalah `argvus-session.target`. Layanan pengguna umum termasuk `argvus-taskbar.service`, `argvus-control-panel.service`, `argvus-widget-telemetry.service`, `argvus-hyprland.service`, `argvus-dunst.service`, `argvus-wallpaper.service`, `argvus-hypridle.service`, layanan papan klip dan `argvus-session-loading.service`.

Firewall adalah layanan sistem bernama `argvus-firewall.service`. Paket kalender menginstal `argvus-taskbar-calendar.service` sebagai unit pengguna, dan `argvus-clock.service` (dinonaktifkan secara default) memulai Jam ARGVUS yang disembunyikan setelah login ketika **Mulai dengan sesi** aktif.

Periksa unit aktif dengan:

```sh
systemctl --user status argvus-session.target
systemctl --user list-units 'argvus-*.service'
```

## Unit milik sesi

Paket sesi menginstal unit pengguna ini di bawah `/usr/lib/systemd/user` (payload sumber berada di bawah pohon konfigurasi bersama paket sesi):

| Satuan | Peran |
| --- | --- |
| `argvus-session.target` | Batas seumur hidup untuk sesi grafis ARGVUS. |
| `argvus-session-prepare.service` | Persiapan startup sekali pakai dan konfigurasi yang dihasilkan. |
| `argvus-control-panel.service` | Panel Kontrol Quickshell. |
| `argvus-taskbar.service` | Bilah tugas ARGVUS utama. |
| `argvus-widget-telemetry.service` | Permukaan telemetri/sysinfo opsional. |
| `argvus-hyprland.service` | Unit sekali pakai sesuai permintaan yang menjalankan `hyprctl reload`. |
| `argvus-config.service` | Memproyeksikan konfigurasi kanonik (`argvus config project`); memuat ulang menjalankan `argvus sessionctl reload-config-transaction`. |
| `argvus-session-reload.service` | Muat ulang login (`argvus sessionctl reload-desktop --login`): memuat ulang Hyprland hanya jika proyeksi memengaruhinya. |
| `argvus-ssh-agent.service` | Agen SSH untuk sesi ini. |
| `argvus-secret-service.service` | Memulai dinas rahasia melalui `argvus sessionctl start-secret-service`. |
| `argvus-wallpaper.service` | Backend wallpaper aktif. |
| `argvus-dunst.service` | Daemon notifikasi Dunst. |
| `argvus-hypridle.service` | Kebijakan menganggur dan mengunci. |
| `argvus-clipboard-text.service` / `argvus-clipboard-image.service` | Pengamat sejarah papan klip. |
| `argvus-keyboard-layout.service` | Notifikasi tata letak keyboard. |
| `argvus-polkit.service` | Integrasi agen PolicyKit. |
| `argvus-session-loading.service` | Hamparan pemuatan sesi dari handoff kompositor hingga komponen desktop dimulai. |

`argvus-blueman-applet.service` juga dikirimkan dengan paket sesi. Beberapa unit bersifat opsional atau dimulai secara kondisional; mereka tidak semuanya aktif di setiap sesi.

Untuk memeriksa definisi efektif suatu unit, bukan hanya pemeriksaan sumbernya:

```sh
systemctl --user cat argvus-session.target
systemctl --user status argvus-session.target
journalctl --user -u argvus-session.target -u 'argvus-*.service'
```

`argvus-firewall.service` tingkat sistem adalah milik `argvus-firewall`. Paket kalender secara terpisah mengirimkan `argvus-taskbar-calendar.service` sebagai unit pengguna. `argvus-clock.service` adalah unit pengguna paket `argvus`, `WantedBy=graphical-session.target`, yang diaktifkan dari pengaturan Jam ARGVUS.

## Perintah manajemen sesi

Perintah `argvus sessionctl` mengelola siklus hidup sesi dan dapat memulai ulang komponen tertentu tanpa keluar:

### Siklus hidup sesi

```sh
argvus sessionctl start               # Start a new session
argvus sessionctl stop                # Stop the current session
argvus sessionctl restart             # Restart the entire session
argvus sessionctl reload              # Reload configuration and active services
argvus sessionctl status              # Show session status
argvus sessionctl logs                # Display session logs
```

### Mulai ulang khusus komponen

Mulai ulang layanan individual tanpa memengaruhi keseluruhan sesi:

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

### Sinkronisasi lingkungan

Impor variabel lingkungan ke dalam sesi:

```sh
argvus sessionctl import-environment
```

Ini menyinkronkan variabel Wayland, Hyprland, Qt, kursor, dan DBus antara layanan pengguna shell dan systemd.

## Referensi unit layanan

Unit pengguna tambahan dapat dikelola langsung dengan `systemctl --user`:

```sh
systemctl --user status argvus-session.target
systemctl --user start argvus-session.target
systemctl --user stop argvus-session.target
systemctl --user restart <unit>

# View logs for a specific service
journalctl --user -u argvus-wallpaper.service -b -f
```

Log untuk bootstrap sesi awal disimpan di:

```
${XDG_STATE_HOME:-$HOME/.local/state}/argvus/session.log
```
