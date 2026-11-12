---
title: Proyek
description: Pemilih proyek dan konfigurasi proyek, melalui proyek argvus.
---

`argvus projects` mencantumkan dan membuka proyek Anda. Ini adalah perintah dalam proses dari `argvus`
paket, dan `SUPER + O` membuka pemilih.

## Perintah

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```

## Apa itu proyek

Proyek adalah direktori yang terdaftar dalam konfigurasi (`/projects/paths`), atau subdirektori dari
root terdaftar di `/projects/roots`. Paling banyak sembilan proyek yang terdaftar: jumlah proyek
Pintasan `SUPER + CTRL + N`. Pembukaan proyek memfokuskan ruang kerja Hyprland-nya. Pertama kali a
proyek terbuka, ARGVUS memulai terminal dan editor mengatur `/projects/editor` di dalamnya.

Konfigurasi dibaca dan ditulis melalui `argvus config`, sehingga Pusat Kontrol dan
dia akan melihat daftar yang sama.

## Proyek ruang kerja dan bilah tugas

Proyek menggunakan ruang kerja biasa bernama Hyprland, bukan ruang kerja khusus. `SUPER + O` menunjukkan nomor proyek, direktori dan pintasan `SUPER + CTRL + N` langsung. Kedua titik masuk memfokuskan ruang kerja yang sama. Hanya ruang kerja terbuka yang muncul di bilah tugas, dengan label `Project: name`, status aktif, dan keterangan alat yang berisi jalur lengkap dan pintasan. Klik tombol ruang kerja untuk kembali ke sana. Bilah tugas mengikuti monitor ruang kerja.

Jalur direktori kanonik menentukan identitas ruang kerja yang stabil; mengubah urutan daftar akan mengubah pintasan, bukan ruang kerja. Proyek dengan nama direktori yang sama tetap terpisah, dan tautan simbolis ke direktori yang sama akan dihapus duplikatnya. Peluncuran terminal dan editor secara eksplisit menargetkan ruang kerja proyek. Editor yang menggunakan kembali proses/jendela yang ada dapat mempertahankan perilaku ruang kerja aplikasi itu sendiri.

Ruang kerja yang dibuka sebelum pembaruan tetap mempertahankan jendela dan nama aslinya. Membuka proyek setelah pembaruan menggunakan identitas turunan jalur barunya; tutup jendela proyek lama jika memungkinkan. Tidak ada asosiasi atau perpindahan otomatis yang dilakukan karena nama lama tidak dapat mengidentifikasi direktori dengan andal. Dengan nama direktori duplikat, gunakan picker, nomor, atau jalur kanonik lengkap, bukan nama yang ambigu.

Membuka proyek akan membuat ulang `$XDG_RUNTIME_DIR/argvus/project-workspaces.json` secara atom dari daftar proyek yang dikonfigurasi dan terjemahannya. Ini adalah metadata sesi untuk label ruang kerja dan keterangan alat, bukan konfigurasi; jangan mengeditnya. Buka kembali proyek setelah mengubah daftar atau bahasa untuk menyegarkan labelnya. Perbarui `argvus` dan `argvus-waybar`, ditambah konfigurasi `argvus-taskbar`, untuk integrasi ini.
