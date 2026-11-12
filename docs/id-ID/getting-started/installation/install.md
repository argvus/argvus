---
title: Instal ARGVUS
description: Instal, perbarui, hapus, dan konfigurasikan ARGVUS.
---

ARGVUS didistribusikan sebagai paket Arch Linux melalui repositori paket ARGVUS resmi.

## Sebelum menginstal

Aktifkan repositori resmi Arch Linux `multilib` sebelum menginstal ARGVUS. Edit `/etc/pacman.conf` dan batalkan komentar pada bagian `[multilib]` dan baris `Include`:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```

Kemudian sinkronkan database paket dan lakukan peningkatan sistem penuh:

```sh
sudo pacman -Syu
```

Penginstal cepat mengonfigurasi repositori ARGVUS, tetapi tidak mengaktifkan `multilib` untuk Anda.

## Instal cepat

Pada instalasi Arch Linux yang baru, Anda dapat menginstal ARGVUS langsung dari TTY tanpa menginstal lingkungan desktop lain terlebih dahulu.

Jalankan:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```

Pemasang:

1. Mengimpor dan menandatangani kunci penandatanganan repositori ARGVUS secara lokal.
2. Mengonfigurasi repositori paket ARGVUS.
3. Memperbarui database paket.
4. Menginstal paket `argvus` dan dependensinya.

Setelah instalasi, keluar dan pilih **ARGVUS** dari pengelola tampilan Anda.

> **Catatan:** Penginstal cepat memerlukan `curl`, `sudo`, dan alat manajemen paket Arch Linux standar.

## Konfigurasikan repositori saja

Untuk mengonfigurasi repositori paket ARGVUS tanpa menginstal desktop, jalankan:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```

Instal ARGVUS secara manual sesudahnya:

```sh
sudo pacman -Syu argvus
```

## Instalasi manual

Jika Anda lebih suka mengonfigurasi repositori langkah demi langkah, gunakan perintah berikut.

### 1. Unduh kunci penandatanganan repositori

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```

### 2. Impor kunci penandatanganan

```sh
sudo pacman-key --add /tmp/argvus.gpg
```

### 3. Tanda tangani kunci secara lokal

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

### 4. Konfigurasikan repositori ARGVUS

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```

Tambahkan konfigurasi repositori ke `/etc/pacman.conf`:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```

Konfigurasi repositori mengaktifkan repositori ARGVUS, termasuk `argvus` dan `argvus-extras`, dengan `SigLevel = Required TrustedOnly`. Pacman menerima paket hanya jika tanda tangannya valid dan dipercaya oleh keyring lokal.

### 5. Instal ARGVUS

```sh
sudo pacman -Syu argvus
```

Setelah instalasi, keluar dan pilih **ARGVUS** dari pengelola tampilan Anda.

## Perbarui

Perbarui ARGVUS bersama dengan Arch Linux lainnya:

```sh
sudo pacman -Syu
```

Untuk memperbarui hanya paket ARGVUS:

```sh
sudo pacman -S argvus
```

> **Rekomendasi:** Arch Linux umumnya merekomendasikan peningkatan sistem penuh dengan `sudo pacman -Syu` daripada memperbarui paket individual.

## Hapus ARGVUS

Hapus ARGVUS sambil mempertahankan dependensi yang masih diperlukan oleh paket terinstal lainnya:

```sh
sudo pacman -R argvus
```

Hapus ARGVUS dan dependensi yang tidak lagi diperlukan oleh paket apa pun yang terinstal:

```sh
sudo pacman -Rns argvus
```

### Hapus repositori ARGVUS

Jika Anda juga ingin menghapus repositori paket, hapus konfigurasinya:

```sh
sudo rm /etc/pacman.d/argvus.conf
```

Kemudian hapus baris yang sesuai dari `/etc/pacman.conf`:

```text
Include = /etc/pacman.d/argvus.conf
```

Anda dapat mengedit file dengan:

```sh
sudo nano /etc/pacman.conf
```

### Hapus kunci penandatanganan

Untuk menghapus kunci penandatanganan repositori ARGVUS dari gantungan kunci pacman, pertama-tama daftarkan kuncinya:

```sh
sudo pacman-key --list
```

Kemudian hapus kunci ARGVUS dengan ID kuncinya:

```sh
sudo pacman-key --delete KEY_ID
```

Ganti `KEY_ID` dengan ID kunci penandatanganan ARGVUS.

> **Peringatan:** Hapus kunci ini hanya ketika Anda juga menghapus repositori ARGVUS. Jangan menghapus kunci yang masih dibutuhkan oleh repositori lain.

## Konfigurasi pengguna

Instalasi paket tidak menulis konfigurasi ke `$HOME`. ARGVUS membaca paket default dari `/usr/share/argvus` dan berfungsi tanpa menyalinnya ke direktori home Anda.

Konfigurasi runtime mengikuti prioritas umum ini:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

`argvus-config` adalah satu-satunya komponen yang menulis `~/.config/argvus/data/generated/`, dan proyeksi tersebut adalah lapisan aktif: salinan terkelola atau asli hanya berlaku ketika file terkait yang dihasilkan tidak ada.Konfigurasi yang dikelola ARGVUS berada di bawah `~/.config/argvus/`. Penggantian `$XDG_CONFIG_HOME/<app>` opsional tetap tersedia untuk aplikasi yang mendukung konfigurasi pengguna asli.

Dispatcher `argvus --setup` bersifat opsional dan untuk manual eksplisit
menimpa, bukan pemulihan. Jika `~/.config/argvus` dihapus, `SUPER + SHIFT + R`
membuat ulang profil kanonik dan menghasilkan status runtime melalui keadaan normal
memuat ulang pipa.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```

`--copy-all` adalah daftar yang diizinkan secara eksplisit dari file profil TUI yang didukung; itu
tidak mengkloning `/usr/share/argvus` dan tidak menyalin skrip, layanan, aset,
file yang dihasilkan atau `config.json`. Jalankan `argvus --setup --help` untuk melihat arusnya
pilihan. Setelah disalin, file menjadi milik pengguna dan dipaketkan
peningkatan tidak menggantikannya.

Untuk mengganti konfigurasi salinan yang ada, gunakan `--force`. ARGVUS membuat cadangan dengan stempel waktu sebelum menggantinya:

```sh
argvus --setup --copy <app> --force
```

## Penggantian pengguna Hyprland

Untuk panduan lengkap penggantian Hyprland, termasuk perbedaan antara konfigurasi asli penuh dan fragmen ARGVUS tambahan, lihat [Penggantian Hyprland](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

### Penggantian mesin virtual

Beberapa mesin virtual mungkin mengalami masalah rendering dengan Hyprland, Kitty, atau Quickshell. ARGVUS mendeteksi mesin virtual menggunakan driver grafis `vmwgfx` dan mengaktifkan rendering perangkat lunak bila diperlukan.

Jika deteksi tidak berfungsi pada mesin virtual Anda, aktifkan fallback secara manual di `user.lua`:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Setelah membuat atau memodifikasi `user.lua`, keluar dari Hyprland dan masuk lagi sehingga variabel diterapkan ke aplikasi yang diluncurkan oleh sesi tersebut.

Verifikasi dengan:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```

Keluaran yang diharapkan adalah:

```text
1
```

> **Catatan:** `LIBGL_ALWAYS_SOFTWARE=1` memaksa aplikasi Mesa/OpenGL menggunakan rendering perangkat lunak, biasanya melalui `llvmpipe`. Hal ini dapat mengurangi kinerja grafis dan harus digunakan sebagai pengganti untuk mesin virtual yang terpengaruh.
