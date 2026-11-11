---
title: Accounts
description: Profil pengguna lokal, nama tampilan, avatar dan keanggotaan grup, melalui akun argvus.
---

`argvus accounts` mencantumkan dan mengedit akun pengguna lokal desktop ARGVUS. Ini adalah privasi
dapat dieksekusi dari paket `argvus`, diinstal di `/usr/lib/argvus/commands/accounts`. Itu
Perintah `argvus` memulainya, dan hanya operasi yang mengubah akun atau data sistem lain
dijalankan sebagai root.

## Perintah

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```

## Menampilkan nama dan avatar

Nama tampilan disimpan di kolom GECOS standar. Nama pengguna Unix tidak pernah diubah.

Avatar adalah gambar dalam format PNG, JPEG, atau WebP, hingga 20 MiB. Gambar diubah ukurannya menjadi 256×256
piksel dan disimpan sebagai PNG dengan mode `0644` di `~/.face`, lokasi desktop gratis dibaca oleh
penyambut dan oleh sebagian besar manajer tampilan. Avatar Anda sendiri tertulis di dalam direktori home Anda dan
tidak memerlukan otorisasi.

## Otorisasi

Hanya `list`, `show`, `self`, `self groups` dan avatar Anda sendiri yang berfungsi tanpa izin. Setiap
perubahan lainnya meminta otorisasi agen polkit desktop. Aturan ARGVUS
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) memutuskan siapa yang harus mengautentikasi:

- pengguna dalam grup administrator (`wheel`, Arch default, atau `sudo`) mengonfirmasi dengan miliknya sendiri
  kata sandi;
- pengguna lain mengonfirmasi dengan kata sandi root.

| Operasi | Tanpa otorisasi | Dengan otorisasi |
| --- | --- | --- |
| Daftar pengguna, tampilkan akun, daftar grup | ✓ | ✓ |
| Ubah avatar Anda sendiri | ✓ | ✓ |
| Ubah nama tampilan Anda sendiri | — | ✓ |
| Ubah kata sandi Anda sendiri | — | ✓ (setelah membuktikan kata sandi saat ini) |
| Ubah nama pengguna lain, avatar, grup atau kata sandi | — | ✓ |

Mengubah kata sandi Anda sendiri terlebih dahulu memeriksa kata sandi saat ini. Kata sandi yang salah ditolak
sebelum perintah otorisasi muncul.

Tidak perlu menjalankan perintah dengan `sudo`. Perintah tersebut meminta polkit untuk otorisasi kapan
itu membutuhkannya.

## Batas keamanan

- `passwd` membaca kata sandi saat ini, baru dan konfirmasi dari input standar, satu per baris,
  dan tidak pernah menganggapnya sebagai argumen. Baris kata sandi saat ini mungkin kosong ketika menjadi administrator
  mengatur ulang kata sandi pengguna lain. Contoh:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  Secara interaktif, ketikkan tiga baris dan tekan Enter setelah masing-masing baris. Masukannya ditunjukkan pada
  terminal seperti yang diketik.
- Mengedit akun di bawah UID 1000 (dan `nobody`) ditolak, begitu pula menguncinya, menghapusnya
  dan masa berlaku kata sandinya habis. Akun Anda sendiri tidak dapat dikunci atau dihapus.
- Mengganti nama atau menghapus grup di bawah GID 1000, atau grup administrator (`wheel`, `sudo`), adalah
  menolak; anggotanya masih dapat diubah, kecuali Anda tidak dapat mengeluarkan diri Anda dari suatu
  grup administrator.
- Keanggotaan administrator adalah grup `wheel` atau `sudo`. Pusat Kontrol menambahkan yang baru
  administrator ke `wheel` jika sistem memilikinya, dan ke `sudo` sebaliknya. Menghapus milik Anda sendiri
  keanggotaan administrator melalui edit ditolak.

## Contoh

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## Pemecahan masalah

- `pkexec was not found`: instal `polkit`. Paket `argvus` bergantung padanya.
- Perintah otorisasi tidak muncul: periksa apakah agen otentikasi polkit sedang berjalan
  dalam sesi Anda. Paket `argvus` bergantung pada `hyprpolkitagent`, yang menyediakannya.
- Kesalahan `unix_chkpwd` saat mengubah kata sandi: paket `pam` menyediakan pembantu.
- Jalankan `argvus accounts --verbose ...` untuk mencetak langkah-langkah perintah ke stderr.

Untuk catatan pengembang tentang batas hak istimewa, lihat `docs/internal/argvus-accounts/`.Ketika administrator mengunci atau menghapus akun yang dikonfigurasi untuk login otomatis, penyedia akun terlebih dahulu menonaktifkan entri login otomatisnya. Jika pembersihan tersebut gagal, pengoperasian akun ditolak. Jika penguncian atau penghapusan di kemudian hari gagal, login otomatis tetap dinonaktifkan. Login otomatis untuk akun lain dipertahankan; membuka kunci akun tidak mengaktifkan login otomatis. Sesi yang ada tidak ditutup dan salam tidak dimulai ulang.