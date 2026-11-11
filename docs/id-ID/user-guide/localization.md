---
title: Localization
description: Gunakan ARGVUS dalam bahasa Inggris atau Portugis.
---

`argvus-i18n` menyediakan katalog bersama, perintah `argvus i18n validate` dan modul QML `org.argvus.i18n`. ARGVUS saat ini mengirimkan katalog berbahasa Inggris dan Portugis.

Aplikasi asli dan komponen shell menggunakan katalog bersama. Oleh karena itu, string yang diterjemahkan dapat digunakan oleh lebih dari satu repositori.

Gunakan **Pusat Kontrol → Lokal & Wilayah** untuk mengonfigurasi bahasa, lokal regional, lokal sistem yang dihasilkan, zona waktu, tanggal/waktu, dan keyboard. Katalog ARGVUS saat ini menyediakan bahasa Inggris dan Portugis; ini terpisah dari daftar lokal sistem yang dihasilkan sistem operasi.

Mengubah bahasa tampilan akan menyimpan preferensi ARGVUS dan secara otomatis memulai ulang `argvus-control-panel.service`, sehingga Panel Kontrol segera mencerminkan perubahan tersebut. Aplikasi lain dan permukaan shell mungkin memerlukan sesi memuat ulang atau sesi baru. Membuat atau mengubah entri `/etc/locale.gen` memerlukan otorisasi dan menjalankan `locale-gen`; memilih lokal yang tidak dihasilkan ditolak. Tata letak keyboard, varian, dan peta tombol konsol merupakan pengaturan terpisah dan berbeda dengan bahasa antarmuka.

## Ketika variabel lingkungan mengesampingkan perubahan

Aplikasi ARGVUS mendeteksi bahasa dari file bahasa ARGVUS persisten terlebih dahulu, kemudian `LC_ALL`, `LC_MESSAGES`, dan `LANG`. Oleh karena itu, variabel lingkungan lokal dapat membuat Pusat Kontrol tampak mengabaikan perubahan bahasa, terutama ketika layanan pengguna atau sesi grafis mempertahankan lingkungan yang lebih lama.

Periksa nilai efektif dengan:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

Jika `~/.config/locale.conf` ada dan mendefinisikan `LANG`, `LC_ALL`, atau `LC_MESSAGES`, ganti namanya untuk sementara sehingga nilai tersebut tidak lagi diperkenalkan kembali. Simpan cadangan alih-alih menghapusnya:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

Terapkan bahasa itu lagi di **Pusat Kontrol → Lokal & Wilayah**. Panel Kontrol dimulai ulang secara otomatis; jika komponen lain masih menampilkan bahasa sebelumnya, keluar dan masuk sehingga sesi grafis dan `systemd --user` menerima lingkungan baru. Ulangi perintah diagnostik setelahnya. Lihat [Variabel lingkungan](/docs/reference/environment-variables/) untuk kontrak teknis dan prioritasnya.