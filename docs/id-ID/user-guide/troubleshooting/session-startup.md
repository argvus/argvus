---
title: Pemecahan masalah permulaan sesi
description: Mendiagnosis kegagalan sesi pengguna Hyprland dan systemd.
---

Jalankan:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

Periksa penyambut, `argvus start`, Hyprland dan unit pengguna yang relevan secara terpisah. Pembuatan paket yang berhasil tidak membuktikan bahwa payload yang diinstal atau penggantian pengguna aktif adalah yang terbaru.

Gunakan cek dalam urutan ini:

1. Konfirmasikan bahwa penyambut menggunakan pengaturan penyambutan ARGVUS yang diinstal.
2. Baca log sesi dan cari konfigurasi Hyprland Lua yang hilang atau tidak valid.
3. Periksa kesiapan dan status target dengan `systemctl --user status argvus-session.target`.
4. Periksa komponen yang gagal daripada memulai ulang setiap layanan sekaligus.

Jika `argvus-session.target` tidak aktif saat overlay pemuatan aktif, periksa
kesalahan proyeksi sebelum memulai kembali layanan:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config` adalah satu-satunya penulis di bawah `data/generated/`, jadi `project` adalah
cara otoritatif untuk mereproduksi generasi sambil mendiagnosis. Permulaan sesi
menjalankannya karena belum ada rencana yang berkomitmen; rencana yang ditulisnya
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` adalah apa
nanti `argvus sessionctl reload` berlaku, tanpa memproyeksikan lagi.

Sesi ini menghentikan overlay pemuatan ketika persiapan fatal atau target startup
gagal. Kegagalan konsumen visual harus menggunakan fallback yang terdokumentasi atau sebaliknya
dilaporkan secara eksplisit; itu tidak boleh disembunyikan sebagai sesi yang sukses.

## Login lambat

Jika `greeter.handoff-takeover-failed` muncul tanpa `handoff.master-acquired`,
periksa `handoff.master-unavailable card=<name> errno=<n>` untuk akuisisi
kesalahan. Seorang pengamat dibuka sementara kompositor lain menjadi master dapat mengembalikan kesalahan
13 (`EACCES`) bahkan setelah kompositor merilis master. Pembantu membuka kembali
GPU yang sama untuk memperoleh master secara implisit; `handoff.master-reopened` mencatat ini
pemulihan dan framebuffer baru disiapkan pada deskriptor itu. Penyambut membatalkan pembantu yang gagal (`greeter.handoff-cancelled`), jadi
tidak dapat menambahkan waktu tunggu lagi sebelum kompositor pengguna memulai. Pengiriman sesi
`release` sementara akuisisi tertunda mendapatkan `handoff.release-before-master`:
helper menutup deskriptor DRMnya sebelum mengakuinya. Seorang komposer penyambut
kecelakaan di Hyprland/Aquamarine adalah temuan terpisah; membatalkan helper akan melakukannya
tidak menetapkan bahwa masalah driver atau kompositor telah diperbaiki.

`greeter.vt-switch-returned active=<tty>` mencatat VT aktif saat login
beralih panggilan kembali. Batas waktu penarikan juga mencatat `active-vt=<tty>` di dalamnya
kesalahan. Ini adalah snapshot, bukan bukti penyerahan DRM yang telah selesai.

Untuk perbandingan sebelum/sesudah, simpan pekerjaan dan gunakan lima login per versi di
pengaturan mesin, tema, dan monitor yang sama. Sebelum setiap logout berikutnya, salin
`/run/argvus-greeter/timing.log`, log sesi dan jurnal boot terpisah
direktori sampel di bawah `/tmp/argvus/`; proses penyambut berikutnya menggantikan waktunya
mencatat. Catat ID boot dengan setiap sampel dan bandingkan stempel waktu hanya di dalamnya
boot. Ukur otentikasi untuk handoff frame pertama, entri pembungkus sesi,
peluncuran Hyprland terakhir dan pemberhentian pemuatan-pemintal. Bandingkan median dan terburuk
waktu otentikasi-ke-pemintal-berhenti dan periksa secara visual apakah ada bingkai hitam,
berkedip atau pemintal macet. Durasi permukaan desktop saja bukanlah waktu yang tepat
waktu login ujung ke ujung. Ulangi tes kegagalan dan pemulihan dalam sesi sekali pakai,
tanpa menghentikan sesi yang berisi pekerjaan yang belum disimpan.

Setiap langkah startup mencatat durasinya di log sesi. Untuk melihat langkah-langkah login terakhir, paling lambat dulu:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

Untuk menentukan waktu peralihan dari penyapa ke desktop, bacalah tanda penyapa dan serah terima bersama dengan jurnal. Semuanya menggunakan `CLOCK_MONOTONIC`, jadi berbaris:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` hanya menampung login terakhir: penyambut, permukaan handoff DRM (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) dan tanda `greeter.*`. Garis `handoff.crtc` mencatat setiap perubahan CRTC sementara handoff menunggu master DRM; baris dengan `fb=0` atau `mode_valid=false`, atau `handoff.output-found ... framebuffer=0`, berarti pembuat penyambut mematikan output sebelum handoff mengambilnya, sehingga monitor harus melakukan sinkronisasi lagi. `greeter.watchdog-stop-requested` berarti sesi penyapa berakhir tanpa menunggu kompositornya, dan `greeter.compositor-exit-deferred` berarti kompositor diminta keluar setelah permukaan handoff hilang; `greeter.compositor-exit-requested` saja artinya diminta keluar sekaligus. Jika baris salam dan login di jurnal menunjukkan jarak sekitar setengah detik antara penutupan sesi penyambut dan pembukaan sesi pengguna, pembongkaran kompositor penyapa akan dijalankan saat login. Hal ini diharapkan ketika tanda menunjukkan `compositor-in-foreground=true`: hangupnya terminal penyambut mengakhiri compositor sebelum pintu keluar yang ditangguhkan. Hal ini terjadi ketika penyambut tidak dapat meninggalkan VT-nya sebelum kompositornya keluar: cari `greeter.vt-switch-requested` diikuti oleh `greeter.handoff-drawing`; ketika yang kedua hilang, `greeter.handoff-takeover-failed error=<reason>` menjelaskan alasannya. `handoff.page-flip` berarti handoff menggantikan frame terakhir penyambut tanpa modeset; `handoff.page-flip-failed errno=<n>` diikuti oleh `handoff.modeset` berarti kernel menolak pembalikan halaman. `handoff.overlay plane=<id>` dan kemudian `handoff.retired` berarti overlay menyembunyikan frame pertama Hyprland hingga pemintal sesi muncul di layar; `handoff.overlay-unavailable` berarti CRTC tidak memiliki bidang overlay gratis, dan `handoff.retire-timeout` berarti pemintal sesi tidak pernah mengirim `retire`. Pemintal sesi (`argvus-greeter-handoff`) menulis `first layer-shell frame callback` dan `stop requested` ke jurnal.

Entri `project.theme-switch` atau bukan nol `prepare.startup-theme` diharapkan setelah perubahan tampilan, perubahan file tema, aksen baru, atau migrasi cap startup lama. Menginstal ulang file yang identik akan mempertahankan cache startup berbasis konten. Jika muncul pada setiap login, file tema yang terwujud hilang; jalankan `argvus theme-switch <theme>` sekali.

Log sesi disimpan di bawah `$XDG_STATE_HOME/argvus/session.log` (biasanya `~/.local/state/argvus/session.log`). Setelah mengubah penggantian Hyprland pengguna, keluar dan masuk sehingga titik masuk sesi memvalidasi dan memilihnya lagi. Lihat [Sesi grafis](/docs/argvus-session/graphical-session/) untuk alur startup normal.