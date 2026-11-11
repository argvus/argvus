---
title: Jaringan dan Bluetooth
description: Kelola NetworkManager dan Bluetooth melalui ARGVUS.
---

Subperintah `argvus network` melaporkan dan mengontrol NetworkManager, dan `argvus bluetooth` melaporkan dan mengontrol status dan ketersediaan adaptor Bluetooth. Keduanya berjalan dalam proses di dalam `argvus`; `networkctl` dan `bluetoothctl` tetap sebagai alias. Panel Kontrol dan Waybar memanggil perintah ini.

Gunakan `--help` pada salah satu perintah untuk subperintah yang diinstal. Integrasi UI jaringan dan Bluetooth terpisah dari manajer lanjutan opsional seperti Blueman.

Pusat Kontrol memisahkan status jaringan dari halaman konfigurasi: **Jaringan → Status** merangkum status saat ini, sedangkan antarmuka, Wi-Fi, Ethernet, VPN, DNS, proxy, dan firewall merupakan rute terpisah ketika penyedianya tersedia. **Bluetooth → Status**, **Perangkat** dan **Pasangan** mencakup status adaptor dan tindakan perangkat. Halaman-halaman ini melakukan tindakan sistem yang didukung penyedia; bilah tugas dan Panel Kontrol memperlihatkan permukaan status/tindakan cepat.

Perubahan jaringan dan Bluetooth mungkin memerlukan otorisasi dan mungkin berlaku segera melalui NetworkManager atau layanan Bluetooth. Hal ini tidak sama dengan mengubah indikator taskbar ARGVUS.

Pusat Kontrol mengirimkan kata sandi Wi-Fi ke nmcli melalui input standar dengan `--ask`; kata sandi dikecualikan dari argumen proses dan dihapus dari kesalahan perintah yang dikembalikan.

## Manajemen jaringan baris perintah

Gunakan `argvus networkctl` untuk menanyakan dan mengontrol jaringan tanpa antarmuka grafis:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```

## Manajemen Bluetooth baris perintah

Gunakan `argvus bluetoothctl` untuk adaptor Bluetooth:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```

Ketika kernel tidak mencantumkan pengontrol di `/sys/class/bluetooth`, `status` segera menjawab `status=no-controller`, tanpa menanyakan `bluetoothctl` (yang sebaliknya akan menunggu BlueZ ketika layanan dihentikan). Modul bilah tugas dan kartu Panel Kontrol membaca status yang sama.

Gunakan `--help` pada salah satu perintah untuk melihat opsi tambahan dan subperintah khusus untuk versi yang Anda instal.

Saat dialog kata sandi Wi-Fi terbuka, pembaruan pemindaian akan mempertahankan SSID dan jenis keamanan yang dipilih. Jika jaringan tersebut hilang atau keamanannya berubah, konfirmasi terhenti karena kesalahan dan memerlukan pilihan baru.

DNS manual mengirimkan server IPv4 dan IPv6 ke pengaturan NetworkManager masing-masing sebelum memperbarui file penyelesai. Symlink penyelesai yang dikelola layanan dipertahankan dengan menolak penggantian file. Pengembalian DNS otomatis yang berhasil akan memulihkan dan menghapus cadangan, sehingga siklus manual selanjutnya akan menyimpan status penyelesai saat ini.

Menutup editor profil jaringan menunggu hingga tugasnya selesai, sehingga penyimpanan atau penghapusan profil tidak dapat dihentikan dengan membuang editor. Penyegaran latar belakang tetap dapat dibatalkan.

Permintaan koneksi Wi-Fi berbagi penjaga tindakan jaringan: tindakan yang sedang berjalan atau tugas editor profil selesai sebelum mutasi lain dapat dimulai. Pesan kemajuan dan penyelesaian mengikuti bahasa yang dipilih.

Konfirmasi penghapusan Bluetooth mempertahankan alamat perangkat asli meskipun daftar perangkat disegarkan. Persentase baterai dari bluetoothctl dikenali, dan kueri detail perangkat dijalankan dengan konkurensi terbatas dan batas waktu individual.

Kata sandi Wi-Fi, DNS manual, dan bidang pencarian jaringan menerima tempel dalam tanda kurung. Kontrol yang ditempel dan jeda baris dihilangkan. Pintasan karakter Ctrl-saja atau Alt-saja yang tidak didukung tidak akan menyisipkan huruf. Editor menerima peristiwa AltGr yang dapat dicetak yang dikodekan sebagai Ctrl+Alt; Ctrl+S hanya menyimpan tanpa Alt. Editor teks bersama menjaga navigasi vertikal di kolom visual dan menyelaraskan pengguliran horizontal untuk karakter lebar.Indikator jaringan bilah tugas mengikuti peristiwa NetworkManager menggunakan `argvus networkctl waybar --follow`, dengan snapshot awal dan koneksi ulang monitor otomatis. Ia tidak lagi menunggu interval pemungutan suara 10 detik. Terhubung, akses Internet terbatas, dan terputus tetap merupakan kondisi yang berbeda. Status Wi-Fi membaca titik akses yang di-cache tanpa meminta pemindaian. `argvus networkctl waybar` memancarkan satu snapshot Waybar JSON.