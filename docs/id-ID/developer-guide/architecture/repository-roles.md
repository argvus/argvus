---
title: Peran repositori
description: Tanggung jawab repositori sumber ARGVUS setelah konsolidasi ruang kerja.
---

Repositori dikelompokkan berdasarkan tanggung jawab, bukan berdasarkan halaman navigasi:

- ruang kerja: `argvus` (pengirim, perintah dalam proses, pembantu GTK 4, peti bersama, penyambut di `crates/argvus-greeter` dan data perintah);
- sesi dan integrasi: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash`, dan penyambut (`argvus-greeter`, dibuat dari `argvus/crates/argvus-greeter`);
- UI di luar ruang kerja: Panel Kontrol, bilah tugas (`argvus-taskbar`), peluncur dan Waybar;
- penyedia: tampilan, daya, firewall, notifikasi;
- aset dan kemasan: tampilan, wallpaper, font, ikon, profil.

## Dikonsolidasikan ke dalam ruang kerja

Repositori ini digabungkan menjadi `de/argvus`. Dokumentasi mereka sekarang berada di bawah `docs/` ruang kerja, dan repositori mereka disimpan hanya untuk riwayat:

| Repositori bekas | Sekarang | Dokumentasi |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [baris perintah](/docs/reference/command-line/), [file konfigurasi](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Komponen internasionalisasi](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | baris perintah |
| `argvus-terminal` | `argvus terminal` | [Komponen terminal](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Komponen monitor sistem](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Komponen jaringan](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | baris perintah |
| `argvus-control-center` | `argvus control-center` (aplikasi terminal) | [Komponen Pusat Kontrol](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (pembantu GTK 4) | [Komponen kalender bilah tugas](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (pembantu GTK 4) | [Komponen perangkat yang dapat dilepas](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (pembantu `splash`) | baris perintah |
| `argvus-tui-terminal` | `argvus tui-terminal` (peti di ruang kerja) | baris perintah |
| `argvus-tui`, `argvus-theme` | Peti bersama `crates/argvus-tui` dan `crates/argvus-theme` | catatan pengembang |
| `argvus-greeter` | Sumber dalam `crates/argvus-greeter`, dikirimkan dalam paket `argvus` | [komponen penyambut](/docs/components/argvus-greeter/) |
| `argvus-themes` | Pensiun. Setiap tema resmi memiliki paket `argvus-theme-*` tersendiri. | [memasang tema](/docs/user-guide/appearance/) |

Catatan pengembang komponen konsolidasi ada di `docs/internal/<repository>/`.

Bahasa bukanlah batas kepemilikan: setiap komponen tetap menggunakan pembantu runtimenya dengan paketnya. Satu-satunya aturan keras yang berlaku di seluruh paket: hanya `argvus-config` yang menulis di bawah `data/generated/`. Komponen yang perlu menjangkau aplikasi yang tidak dapat membaca `config.json` menambahkan adaptor di pembantunya sendiri dan membiarkan orkestrator memanggilnya setelah penerapan kanonik; itu tidak pernah menulis file yang dihasilkan. CLI publik termasuk dalam `/usr/bin`; pembantu internal berada di bawah `/usr/lib/argvus/<component>` atau di direktori perintah ketika dibutuhkan saat runtime; data yang tidak bergantung pada arsitektur tetap berada di bawah `/usr/share/argvus/<component>`.

Lihat [matriks paket](/docs/reference/package-matrix/) untuk batas paket yang diinstal.
