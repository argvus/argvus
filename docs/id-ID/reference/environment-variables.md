---
title: Variabel lingkungan
description: Environment contracts used by ARGVUS session components.
---

Sesi ini menetapkan integrasi lingkungan XDG dan Wayland dan dapat mengekspor variabel kompatibilitas virtualisasi ketika terdeteksi. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` dan `ARGVUS_BOOTSTRAP` digunakan oleh pembantu sesi dan operator untuk menyelesaikan akar konfigurasi.

Perlakukan variabel lingkungan sebagai kontrak waktu proses: verifikasi `--help` atau sumber helper yang diinstal sebelum menambahkan penggantian persisten.

Status turunan berada di bawah root status, bukan di root konfigurasi.
`XDG_STATE_HOME` (biasanya `~/.local/state`) menerima `argvus/config-projection.json`,
manifes proyeksi `argvus-config` dibangun kembali dan `argvus sessionctl reload`
berlaku, ditambah `argvus/session.log`. Keduanya bukanlah sumber kebenaran.

## Lingkungan bahasa

Satu-satunya preferensi bahasa umum adalah `LANG` di `$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`). Tanpa nilai yang valid, ARGVUS memilih bahasa Inggris (`en-US`), meskipun `/etc/locale.conf`, PAM atau manajer login menyediakan bahasa Portugis. Nilai seperti `pt_BR.UTF-8` dinormalisasi menjadi `pt-BR` untuk katalog.

Sesi ini menerapkan preferensi ini sebelum memulai komponen desktop dan mengimpor pengguna systemd dan lingkungan aktivasi D-Bus. Ini menggantikan `LANG`, `LC_MESSAGES`, `LANGUAGE` yang diwarisi dan menghapus `LC_ALL`; pengaturan `LC_*` regional dalam file yang sama dipertahankan. Bahasa Inggris menggunakan `en_US.UTF-8` saat dibuat, jika tidak, `C.UTF-8`. Tidak ada file preferensi yang dibuat secara otomatis. Keluar dan masuk kembali setelah mengubah bahasa.

`session.language`, `argvus/data/language` dan `argvus/data/internal/language` diabaikan data lama. Mereka tidak diimpor atau diregenerasi; file yang ada dipertahankan. `argvus config set` dan `patch` menolak penulisan baru ke `session.language` dan mengarahkan penelepon ke `locale.conf`.

Untuk diagnosis, bandingkan `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` dan `systemctl --user show-environment`. `localectl status` menjelaskan sistem, bukan bahasa ARGVUS pengguna. `ARGVUS_I18N_DIR` memilih direktori katalog untuk pengembangan, bukan bahasanya.