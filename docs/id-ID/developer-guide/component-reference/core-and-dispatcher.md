---
title: Inti dan operator
description: Ruang kerja argvus, operatornya, dan peti Rust bersama.
---

`argvus` adalah ruang kerja Kargo di `de/argvus` dan paket yang menyediakan `/usr/bin/argvus`. Peti `argvus-cli` adalah operatornya: ia membaca argumen pertama dan menjalankan perintah perpustakaan dalam proses, menjalankan pembantu GTK 4 atau executable pribadi, atau menjalankan skrip yang dikemas. Daftar subperintah lengkap ada di [referensi baris perintah](/docs/reference/command-line/).

Ruang kerja juga menyimpan peti yang dibagikan oleh perintah:

- `argvus-config`: penyimpanan konfigurasi kanonik dan satu-satunya penulis pohon yang dihasilkan (`data/generated/`). Ini juga merupakan perpustakaan yang digunakan dalam proses oleh `argvus config`, `theme-switch` dan `effects-toggle`.
- `argvus-i18n` dan `argvus-i18n-core`: katalog di `resources/locales/`, validasinya, dan modul QML `org.argvus.i18n`.
- `argvus-tui` dan `argvus-theme`: UI terminal bersama dan pemuatan tema.
- `argvus-runtime`: pesan yang diterjemahkan dan pembantu proses yang dibagikan oleh perintah.

`argvus-session` tetap menjadi repositori terpisah. Ini menyediakan titik masuk sesi (`session`, `start`, `tty`) dan unit siklus hidup, dan `argvus sessionctl` menjangkau pengontrolnya.