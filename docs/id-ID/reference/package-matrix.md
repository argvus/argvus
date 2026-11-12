---
title: Matriks paket
description: tanggung jawab paket ARGVUS.
---

| Paket | Tanggung jawab |
| --- | --- |
| `argvus` | Paket master: operator, perintah dalam proses (konfigurasi, terjemahan, kunci, proyek, terminal, monitor sistem, jaringan, Bluetooth, telemetri, terminal TUI, pengaturan), data bersama dan katalog i18n. Setiap paket komponen bergantung padanya. |
| `argvus` (pembantu) | Pembantu dalam paket yang sama: Pusat Kontrol dan Penganalisis Penggunaan Disk (aplikasi terminal), ditambah aplikasi GTK 4 untuk kalender bilah tugas, perangkat yang dapat dilepas, overlay pemuatan, Fokus ARGVUS, dan Jam ARGVUS. |
| `argvus` (penyambut) | Login penyambut untuk penyambutan, dikirimkan dalam paket yang sama dan dibuat dari `crates/argvus-greeter`. |
| `argvus-desktop` | Profil desktop dan aplikasi sehari-hari diperlukan oleh paket master `argvus`. Itu tidak bergantung pada `argvus`, sehingga ketergantungannya tetap satu arah. |
| `argvus-devtools`, `argvus-pentest` | Paket meta independen opsional untuk alat pengembangan dan audit keamanan resmi. Paket master tidak menginstal profil ini. |
| `argvus-session` | Titik masuk sesi dan siklus hidup pengguna. |
| `argvus-hyprland` | Konfigurasi Hyprland. |
| `argvus-control-panel` | Panel kontrol Quickshell. |
| `argvus-taskbar`, `argvus-waybar` | Konfigurasi waybar dan biner yang ditambal. |
| `argvus-branding` | Aset logo dan tanda kata. |
| `argvus-appearance`, `argvus-wallpapers`, `argvus-fonts`, `argvus-icons` | Keadaan penampilan dan aset visual. |
| `argvus-display`, `argvus-power`, `argvus-notifications` | Penyedia desktop. |
| `argvus-lock`, `argvus-boot-splash` | Kunci layar dan boot splash. |
| `argvus-firewall` | Firewall, dengan keistimewaannya tersendiri. |
| `argvus-launcher` | Konfigurasi peluncur dan titik masuk. |
| `argvus-app-profiles` | Profil hanya data untuk aplikasi terminal yang dibundel. |
| `argvus-portal` | Portal XDG dan default Wayland/DBus. |
| `argvus-theme-*` | Satu paket opsional per kelompok tema resmi. |
| `argvus-game-snake`, `argvus-games` | Opsional. Snake adalah proyek independen; `argvus-games` mengelompokkan game opsional. |

## Paket yang diganti

`argvus` mendeklarasikan `replaces`, `conflicts` dan `provides` untuk paket yang diserapnya, sehingga pemutakhiran akan menghapus paket tersebut dan filenya tidak akan tetap menjadi yatim piatu:

`argvus-config`, `argvus-i18n` (dengan `argvus-language-en-us` dan `argvus-language-pt-br`), `argvus-keys`, `argvus-projects`, `argvus-terminal`, `argvus-system-monitor`, `argvus-network`, `argvus-widget-telemetry` dan `argvus-du`. `argvus` juga menggantikan `argvus-control-center`, `argvus-taskbar-calendar`, `argvus-loading-theme`, `argvus-removable-devices`, dan paket split sebelumnya `argvus-gtk` dan `argvus-greeter`, yang masih disediakan sehingga paket-paket yang bergantung padanya tetap terselesaikan. Manajer akun (`argvus accounts`) dibuat dari `crates/argvus-accounts` dan diinstal sebagai `/usr/lib/argvus/commands/accounts`, dengan aturan polkitnya. Disk Usage Analyzer dibuat dari `crates/argvus-du` dan diinstal sebagai `/usr/lib/argvus/commands/argvus-du`.

`/usr/bin/argvus-terminal` tetap terinstal sebagai peluncur kompatibilitas untuk `argvus terminal`. Terminal default Hyprland, katalog Pusat Kontrol, dan aplikasi default yang disimpan masih memberi nama perintah itu.

Paket `argvus-pwvucontrol` dan `argvus-snappy-switcher` yang dihapus tidak disediakan oleh paket mana pun.

Tidak diperlukan urutan instalasi di antara paket komponen ARGVUS: `argvus` adalah paket masternya, dan setiap komponen bergantung padanya. Menginstal `argvus` juga menginstal profil `argvus-desktop`; `argvus-devtools` dan `argvus-pentest` tetap ikut serta. Kehadiran sebuah paket dalam tabel ini tidak berarti paket tersebut memiliki seluruh fitur: misalnya, taskbar menggabungkan `argvus-waybar`, `argvus-taskbar`, `argvus-session` dan penyedia domain.
