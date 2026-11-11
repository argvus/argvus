---
title: Penyambut
description: Penyambut login yang memulai salam, dikirimkan dalam paket argvus.
---

`argvus-greeter` adalah penyambut login untuk salam. Ini adalah terminal yang mengutamakan keyboard
frontend yang menemukan sesi Wayland yang diinstal, menunjukkan akun dengan sesi mereka
avatar, dan menerapkan tema ARGVUS aktif untuk akun yang dipilih.

Ini adalah bagian dari paket `argvus`, paket tunggal desktop. Sumbernya hidup
di `crates/argvus-greeter` dari repositori `argvus`, sebagai anggota root Cargo
ruang kerja, dan dibangun bersama dengan `argvus` lainnya.

Penyambut tidak mengautentikasi pengguna itu sendiri. salam menangani otentikasi melalui
PAM. Peluncur membuka penyambut dengan `argvus tui-terminal`, sehingga penyambut tidak
memerlukan program terminal terpisah.

File konfigurasi penyambut diinstal di bawah `/etc/argvus`. Gunakan
`argvus greeter-setup --help` untuk melihat opsi pengaturan yang disediakan paket.
Untuk alur login, lihat [Sesi](/docs/user-guide/sessions/).

Soket handoff login mengotorisasi setiap klien dengan kredensial rekan kernel. Hanya root, pemilik helper dan akun yang diautentikasi dengan salam yang dapat mengirimkan perintah kontrol. Akun lokal lainnya ditolak meskipun soket mengizinkan koneksi antar pengguna. Kerangka DRM yang telah disiapkan tetap terikat pada deskriptor aslinya; pembantu menunggu deskriptor itu sebelum mengambil alih tampilan.

Ketika bidang otentikasi difokuskan, karakter yang dapat dicetak (termasuk `j` dan `k`) memasukkan respons. Tab dan Shift-Tab memindahkan fokus; Backspace menghapus karakter terakhir. Ini berlaku untuk perintah rahasia dan non-rahasia.

Daftar akun bergulir agar akun yang dipilih tetap terlihat saat Anda menavigasi atau mengubah ukuran terminal.