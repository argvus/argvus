---
title: Konfigurasi pertama
description: Jalur boot pertama yang praktis untuk mengonfigurasi ARGVUS.
---

Setelah sesi grafis pertama, Anda tidak perlu mengkonfigurasi setiap bagian ARGVUS. Mulailah dengan pengaturan yang memengaruhi tampilan desktop, lalu konfigurasikan perangkat keras dan layanan sesuai kebutuhan.

## Pesanan yang bermanfaat

1. Baca [ikhtisar desktop](/docs/user-guide/desktop/) untuk mengidentifikasi jendela, ruang kerja, bilah tugas, Panel Kontrol, dan peluncur.
2. Buka [Pusat Kontrol](/docs/components/argvus-control-center/) dan pilih tema, mode, aksen, dan wallpaper di **Penampilan**.
3. Sesuaikan [jendela dan tata letak](/docs/argvus-hyprland/windows-and-layout/) jika celah, batas, atau jarak bilah tugas default tidak sesuai dengan layar Anda.
4. Konfigurasikan [taskbar](/docs/argvus-taskbar/taskbar/) dan tentukan kartu Panel Kontrol mana yang ingin Anda tampilkan.
5. Tinjau [pintasan keyboard](/docs/argvus-hyprland/keyboard-shortcuts/) dan ubah hanya binding yang sebenarnya Anda gunakan.
6. Konfigurasikan [mouse dan touchpad](/docs/argvus-hyprland/input/), tampilan, jaringan, dan daya dari halaman pengaturan yang relevan.

Empat langkah pertama adalah personalisasi opsional. Langkah-langkah selanjutnya biasanya hanya diperlukan ketika perangkat keras, bahasa, atau alur kerja Anda memerlukannya.

## Dua tempat untuk mengubah pengaturan

Gunakan **Pusat Kontrol** untuk pengaturan yang menjelaskan cara ARGVUS dikonfigurasi: tampilan, tata letak, font, input, pintasan, bahasa, wilayah, dan aplikasi default. Gunakan **Panel Kontrol** untuk status dan tindakan sesi yang sering dilakukan seperti volume, kecerahan, status jaringan, notifikasi, daya, dan kontrol sesi.

Lihat [Pusat Kontrol vs Panel Kontrol](/docs/components/argvus-control-center/) untuk perbedaan selengkapnya dan [Di mana mengonfigurasi berbagai hal](/docs/user-guide/where-to-configure/) untuk indeks cepat.

## Perubahan, ketekunan dan pemulihan

Sebagian besar perubahan Pusat Kontrol diterapkan saat sesi berjalan dan disimpan sebagai status pengguna ARGVUS. Perubahan tampilan dilakukan pada konfigurasi kanonik, dan `argvus-config` memproyeksikan file yang digunakan oleh Hyprland dan shell darinya. Pengalihan Panel Kontrol mungkin berupa tindakan langsung atau perubahan status sesi, bukan konfigurasi permanen.

Saat halaman menawarkan **Reset default**, gunakan tindakan halaman tersebut daripada menghapus file yang dihasilkan. Penampilan memiliki tindakan pengaturan ulang terpisah untuk aksen, wallpaper, dan nilai tata letak; halaman pengikat kunci memiliki tindakan pemulihannya sendiri. Tidak ada satu tombol reset universal untuk setiap pengaturan desktop.

## Langkah selanjutnya

- [Penampilan](/docs/user-guide/appearance/) — tema, wallpaper, efek, dan tata letak.
- [Tata letak desktop](/docs/argvus-hyprland/windows-and-layout/) — memahami celah, batas, dan ruang panel.
- [Perangkat input](/docs/argvus-hyprland/input/) — kontrol mouse dan touchpad.
- [Pemecahan masalah tema dan wallpaper](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — pemulihan ketika perubahan visual tidak seperti yang Anda harapkan.