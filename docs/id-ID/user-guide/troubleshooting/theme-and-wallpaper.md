---
title: Pemecahan masalah tema dan wallpaper
description: Mendiagnosis propagasi tema dan menghasilkan file tampilan.
---

Periksa bagian `appearance` kanonik di `$XDG_CONFIG_HOME/argvus/config.json`, lalu periksa file yang dihasilkan di bawah `$XDG_CONFIG_HOME/argvus/data/generated`. File dot lama hanya merupakan masukan migrasi.

`argvus-config` adalah satu-satunya penulis di bawah `data/generated/`. Untuk memulihkan yang rusak
tema atau wallpaper, membangun kembali seluruh generasi dari dokumen kanonik dan
terapkan hasilnya alih-alih mengedit konsumen yang dihasilkan secara langsung:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```

`argvus theme-switch` (yang menjalankan `argvus config theme-apply`) dan `argvus config accent`
adalah orkestrator: mereka memvalidasi, mengunci, menampilkan splash, melakukan commit
`argvus-config` dan kemudian merekonsiliasi konsumen eksternal yang tidak bisa membaca
`config.json`. Mereka tidak pernah menulis `data/generated/`, dan transaksi tema menulisnya
tidak menulis file GTK, penyimpanan, atau `mode.css` melalui skrip terpisah.
Gunakan `argvus config accent --apply` untuk menjalankan kembali rekonsiliasi aksen saja.

Perubahan tema resmi menerapkan nilai tampilan yang disimpan dan memperbarui konsumen runtime yang terpengaruh. Menerapkan tema khusus akan memulai ulang bilah tugas, memuat ulang Hyprland, wallpaper, dan layanan Panel Kontrol. Peralihan Sticky atau Spasi akan menerapkan kembali spasi, batas, dan margin bilah tugas mode tersebut, lalu memulai ulang bilah tugas, telemetri, dan layanan muat ulang Hyprland. Warna notifikasi Dunst adalah pengecualian saat Jangan Ganggu aktif: pemuatan ulangnya ditunda hingga DND dimatikan. Jika permukaan lain tetap menggunakan warna lama, periksa apakah pembuatan tema aktif dan itu adalah tema yang Anda pilih, lalu jalankan pemuatan ulang lagi:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```

Setiap file bertema berasal dari build: tema yang tidak diinstal (tidak ada `theme.toml` di bawah `/usr/share/argvus/themes/` atau `~/.local/share/argvus/themes/`) atau tidak dirender akan ditolak, dan tema sebelumnya tetap ada. `argvus theme check DIRECTORY` menunjukkan templat dan baris dari setiap kesalahan render. Bilah Waybar memuat ulang warnanya hanya dengan `reload_style_on_change` di JSONC-nya. Jika saklar tampak macet, periksa apakah saklar masih berjalan; ia mencatat dirinya sendiri di `theme-switch.active`, di samping kuncinya:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```

Pid pada marker yang masih hidup berarti peralihan belum selesai. Sebuah saklar menunggu paling lama 60 detik untuk kunci tema dan proyeksi, dan 5 detik untuk perintah muat ulang; penanda yang prosesnya telah keluar sudah basi dan saklar berikutnya mengabaikannya. Sertakan output dari perintah di atas saat melaporkan masalah.

Jika hanya penyambut pra-login yang berwarna putih atau menggunakan tema cadangan, bandingkan sumber pribadi dengan proyeksi publiknya:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```

Perbaiki proyeksi tanpa mengubah konfigurasi yang disambut:

```sh
sudo argvus greeter-setup --sync-themes
```

File tersebut diterbitkan untuk pra-otentikasi oleh orkestrator penampilan dan
tinggal di luar `data/generated/`, jadi `argvus config apply` biasa tidak
menyegarkannya.

Jika penyambut melaporkan `Invalid color name: 'null'` dalam cache Kitty, perbarui
Paket `argvus terminal` dan buat ulang cache dengan membuka penyambut lagi.
Aksen kanonik yang tidak ada adalah warna default, bukan warna Kitty; terminal saat ini
peluncur menolak JSON `null` dan nilai lama yang tidak valid sebelum ditulis
`active_tab_foreground`.

Jika terminal default berhenti terbuka setelah beberapa kali perubahan tema, periksa cache Kitty yang diperoleh alih-alih mengeditnya secara manual:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```

Perintah di atas membaca preferensi kanonik. `.active-theme` hanya a
proyeksi kompatibilitas. Cache dibuat ulang oleh `argvus terminal` dan
disinkronkan dengan aplikasi tema. Hilang atau basi
`/usr/lib/argvus/loading-theme/splash` menunjukkan pembaruan paket yang tidak lengkap;
membangun kembali dan menginstal paket terkoordinasi sebelum menilai sesi langsung
hasil.

Jika batas jendela tetap menggunakan warna tema sebelumnya, jangan edit Lua Hyprland
mengajukan secara manual. Perubahan tema dan aksen diproyeksikan dari `config.json`, lalu`argvus sessionctl reload` memuat ulang Hyprland dan menerapkan kembali yang aktif, tidak aktif,
dan mengelompokkan warna batas sepanjang siklus hidup sesi:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```

Batas *preferensi* diubah melalui `argvus config borders` (atau
Delegasi `borders-switch.sh`) di Pusat Kontrol dan Panel Kontrol; memuat ulang
menerapkan kembali warna yang dikomit melalui siklus hidup sesi dan tidak
menurunkannya kembali.

Periksa rencana dan nilai kompositor yang efektif:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```

Jika `hyprctl` tidak dapat mencapai soket Hyprland saat ini, proyeksi perbatasannya adalah
dilaporkan sebagai gagal dan komponen dependen tidak dianggap berhasil
dimuat ulang. Periksa jurnal sesi pengguna sebelum mencoba lagi.
