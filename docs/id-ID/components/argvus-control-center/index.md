---
title: Pusat Kontrol
description: Konfigurasikan bagian persisten desktop ARGVUS Anda.
---

Pusat Kontrol adalah aplikasi pengaturan keyboard pertama ARGVUS. Gunakan saat Anda ingin mengonfigurasi desktop daripada melakukan tindakan satu kali saja.

## Pusat Kontrol dan Panel Kontrol

| Gunakan… | Untuk… |
| --- | --- |
| **Pusat Kontrol** | Tema, wallpaper, efek, tata letak, perilaku bilah tugas, font, aplikasi default, pengaturan keyboard dan input, bahasa, wilayah, dan konfigurasi sistem. |
| **Panel Kontrol** | Status saat ini dan tindakan yang sering dilakukan seperti volume, kecerahan, jaringan, notifikasi, daya, dan kontrol sesi. |

Panel Kontrol dapat menampilkan kontrol cepat untuk pengaturan yang dikonfigurasi secara permanen di Pusat Kontrol. Misalnya, gunakan panel untuk tampilan cepat atau tindakan tampilan, dan Pusat Kontrol saat Anda ingin mengubah konfigurasi desktop di belakangnya. Lihat Panel Kontrol dan [Tempat mengonfigurasi berbagai hal](/docs/user-guide/where-to-configure/).

## Area utama

Rumah Pusat Kontrol saat ini menyediakan area berikut:

Rumah adalah katalog tujuan yang dikelompokkan dalam enam kartu. Setiap tujuan membuka halamannya secara langsung, dan pencarian menemukannya di bawah kartu apa pun.
- **Tampilan** — menyesuaikan desktop: Gaya Desktop (tema, mode tampilan, warna aksen, wallpaper, ikon, dan font), Gaya Aplikasi (transparansi terminal dan peluncur), Bilah Tugas, Panel Kontrol, dan Telemetri Widget.
- **Desktop & Input** — konfigurasikan ruang kerja: Hyprland (Windows: Aturan, Spasi dan Mode, animasi, blur, batas, bayangan, dan versi terinstal), Layar, Keyboard & Pintasan, dan Mouse & Touchpad.
- **Preferensi** — pengaturan pribadi dan regional: Bahasa & Wilayah (bahasa, zona waktu, format dan hub tata letak keyboard), Tanggal & Waktu, Aplikasi Default, Daya dan Sesi.
- **Perangkat Keras & Konektivitas** — perangkat dan koneksi: Ikhtisar Perangkat Keras, Jaringan & Wi-Fi, Bluetooth, Audio dan Penyimpanan.
- **Administrasi Sistem** — mengelola sistem operasi: Paket, Layanan, Boot, Nama Host, Pengguna & Grup, Firewall, dan Notifikasi (Jangan Ganggu).
- **Pengaturan** — Pusat Kontrol itu sendiri, tercantum setelah kartu: Bahasa (bahasa pengguna) dan Gaya (ikon dekoratif dan transparansi). Tekan `s` untuk membukanya.
- **Alat & Diagnostik** — alat pengembang dan pemeliharaan: ARGVUS Selamat Datang, Proyek, Cuplikan (teks disimpan untuk `argvus snippets`, diketik dengan `SUPER + ALT + N`), kunci SSH (pasangan kunci dalam `~/.ssh` dikelola oleh `argvus keys`), kunci GPG (rangkaian kunci yang dikelola oleh `argvus gpg`; agen gpg menggunakan pinentry-gtk), Layanan Pengembang, Dokter Sistem, dan Tentang ARGVUS.

Saat Jangan Ganggu diaktifkan, pop-up notifikasi ditutup dan notifikasi baru disimpan untuk nanti. Panel Kontrol menyembunyikan riwayat pemberitahuannya hingga Jangan Ganggu dinonaktifkan.

## ARGVUS Selamat datang

Pilih **Buka Selamat Datang ARGVUS** di bawah **Alat & Diagnostik**, cari `welcome`, atau jalankan `argvus welcome`. Selamat Datang terintegrasi ke dalam Pusat Kontrol dan menggunakan satu jendelanya. Enter membuka halaman pengaturan yang dipilih; setelah selesai, kembali ke panduan dengan Esc. F6 Berikutnya memeriksa langkah saat ini dan membuka langkah berikutnya di jendela yang sama. Esc hanya kembali, dan Enter pada daftar periksa selalu membuka halaman yang dipilih. O meninjau kembali langkah tersebut, S melompati tanpa mencentangnya, dan F menyelesaikan panduan bahkan dengan langkah opsional yang tertunda. Membatalkan atau mengembalikan tidak menyelesaikan satu langkah pun. Kemajuan yang ada dipertahankan.

Panduan ini mencakup bahasa dan wilayah, tampilan, perangkat keras dan driver, jaringan, audio, Bluetooth, tampilan, daya, aplikasi default, dan permulaan sesi. Itu tidak pernah menginstal driver atau paket dan tidak menerapkan perubahan sistem dengan sendirinya. Kemajuan disimpan di `$XDG_STATE_HOME/argvus/welcome/progress.json` (atau `~/.local/state/argvus/welcome/progress.json`).Setelah desktop siap, sesi `argvus welcome --if-needed` dijalankan untuk TTY, ARGVUS Greeter, dan pengelola login lainnya. Ini membuka Selamat Datang kecuali kemajuan per pengguna mencatat `opened`, penyelesaian atau pemberhentian. Render pertama yang berhasil menyimpan `opened` di `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`, mencegah pembukaan kembali pada login berikutnya. Itu tidak bergantung pada `/etc/skel`. Akun yang ada tanpa catatan tersebut juga menerima satu pembukaan otomatis. Kegagalan sebelum rendering memungkinkan upaya lain pada login berikutnya. Panduan `argvus welcome` tetap tersedia.

Halaman penyedia yang tepat dapat bergantung pada paket dan izin yang diinstal. Jangan berasumsi bahwa domain opsional yang hilang adalah Pusat Kontrol yang rusak.

Ketika kisi memiliki lebih banyak entri daripada yang muat di layar, bilah gulir muncul di sepanjang tepi kanan untuk menunjukkan bahwa ada lebih banyak entri di bawah.

## Menemukan pengaturan

Layar beranda mendaftarkan pengaturan berdasarkan judul, kategori, pengidentifikasi, dan kata kunci. Penelusuran tidak peka terhadap aksen dan menerima istilah praktis seperti `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` dan `DPI`. Pencocokan judul yang tepat diberi peringkat pertama, diikuti dengan awalan judul, kata apa pun dari judul atau kategori yang berisi teks yang diketik, dan terakhir pencocokan kata kunci — sehingga mengetik bagian mana pun dari sebuah kata (tidak hanya bagian awalnya) masih dapat menemukan tujuannya.

Menu yang terlihat sadar akan kemampuan. Sebuah rute bisa hilang ketika penyedianya tidak diinstal, ketika perangkat keras tidak tersedia, atau ketika sesi saat ini tidak dapat memberikan kemampuan yang diperlukan. Oleh karena itu, pencarian juga merupakan cara yang berguna untuk menemukan apa yang dapat dikonfigurasi oleh instalasi ini.

Pohon pengaturan utama saat ini meliputi:

| Daerah | Tujuan saat ini |
| --- | --- |
| Penampilan | Gaya Desktop: tema, mode tampilan, warna aksen, wallpaper, ikon dan font; Gaya Aplikasi: transparansi terminal dan peluncur; Bilah Tugas; Panel Kontrol; Telemetri Widget |
| Desktop & Masukan | Hyprland (Windows: Aturan, Spasi dan Mode, animasi, blur, batas dan bayangan), Layar, Keyboard & Pintasan, Mouse & Touchpad |
| Preferensi | Bahasa & Wilayah (bahasa, zona waktu, format, lokal sistem, dan tata letak keyboard), Tanggal & Waktu, Aplikasi Default, Daya, Sesi |
| Perangkat Keras & Konektivitas | Ikhtisar Perangkat Keras (ringkasan, CPU, GPU, memori, perangkat), Jaringan & Wi-Fi, Bluetooth, Audio, Penyimpanan |
| Administrasi Sistem | Paket, Layanan, Boot, Nama Host, Pengguna & Grup, Firewall, Notifikasi |
| Pengaturan | Bahasa (bahasa pengguna), Gaya (ikon dan transparansi) |
| Alat & Diagnostik | Proyek, Cuplikan, kunci SSH, kunci GPG, Layanan Pengembang, Dokter Sistem, Tentang ARGVUS |

Beberapa di antaranya adalah halaman penuh dan beberapa lagi merupakan subhalaman yang dicapai melalui pencarian atau ringkasan domain. Daftar pastinya ditentukan oleh penyedia ARGVUS yang diinstal.

Setiap halaman menunjukkan jalurnya di header, misalnya `Control Center > Hardware & Connectivity > Audio`. Jalurnya dimulai dengan kartu Beranda yang mencantumkan halaman tersebut, sehingga halaman yang sama terbaca sama dari Beranda, dari pencarian, dan dari CLI. Root dan nama kartu diterjemahkan: antarmuka Portugis menunjukkan `Central de Controle > Hardware e conectividade > Áudio`.

## Untuk apa setiap area

### PenampilanPenampilan adalah area penyesuaian utama. Grup Desktopnya berisi tema (dengan mode Lengket atau Spasi), mode tampilan, warna aksen, wallpaper, tema ikon dan halaman Font, dan grup Gaya Aplikasinya berisi transparansi terminal dan peluncur. Taskbar, Control Panel dan Widget Telemetry terdaftar langsung di bawah Appearance di beranda dan menggunakan kembali halaman transparansi bersama Appearance untuk permukaannya sendiri. Blur adalah salah satu pengaturan global, di bawah Hyprland > Blur, dan ini berlaku untuk setiap permukaan ARGVUS yang mengaktifkan blur. Halaman tersebut mengedit tombol Aktifkan dan nilai keburaman Hyprland (Ukuran, Lulus, Kecerahan, Kebisingan, Kontras, Semangat, dan Kegelapan Semangat) sebagai draf, dan tidak ada yang berubah hingga **Terapkan** dipilih. Animasi bekerja dengan cara yang sama di Hyprland > Animations: tombol Aktifkan, kurva Kustom (Bézier, Spring, Linear, dan Ease-out) dan lima kategori animasi membentuk satu draf yang diterapkan bersama dengan **Terapkan**. Bézier dan Spring memiliki parameter yang dapat diedit (titik Bézier dan massa pegas, kekakuan dan peredam); Linear dan Ease-out adalah kurva tetap, sehingga halamannya hanya mencantumkan titik kontrolnya dan tidak dapat diedit. Pilihan gaya mengikuti apa yang digambar Hyprland 0.56.2: Pembukaan dan Penutupan menawarkan slide, popin dan gnomed, lapisan masuk dan keluar menawarkan slide, popin dan fade, dan daun ruang kerja menawarkan slide, fade, slidefade, slidevert dan slidefadevert. Kontrol ini berkoordinasi dengan tampilan dan komponen sesi, bukan hanya mengubah jendela Pusat Kontrol.

### Bilah Tugas

Taskbar mengonfigurasi posisi panel, jarak di sekelilingnya dan jendelanya, tinggi dan batasnya, grup ikon utilitas, dan blok tanggal/waktu, ditambah transparansinya sendiri. Itu tercantum di bawah **Penampilan** di rumah. Dalam mode Sticky, **Spasi > Bawah** defaultnya adalah `2`; Spasi mempertahankan garis dasar piksel `18`.

**Tinggi** memilih 24, 28 (default), 32 atau 36 piksel. **Border** memiliki tombol **Enable** dan **Thickness** 1 hingga 3 piksel, dengan 1 sebagai default. Dengan tidak adanya pembatas, jendela taskbar, grup kanan, dan pemutar MPRIS tidak memiliki pembatas. Perubahan ini masih dalam bentuk draf hingga **Terapkan** dipilih. Ketinggian dan batas yang diterapkan dipertahankan saat tema berubah; sampai diterapkan, temalah yang menentukannya.

Halaman **Ikon** akan membuka **Modul opsional**, **Presentasi ruang kerja**, **Peluncur** dan **Utilitas**. Modul opsional dapat ditampilkan atau disembunyikan satu per satu: pemutar audio, Fokus, peluncur, jaringan, profil daya, tata letak keyboard, memori, CPU dan pembacaan suhu, tanggal, jam, baterai, baki sistem, notifikasi, perangkat yang dapat dilepas, rekaman, Bluetooth, input/output audio, dan tetap terjaga. Ruang kerja tetap tersedia dan dapat menampilkan namanya (default) atau ikon aplikasi. Judul jendela terfokus, Pengaturan, dan tindakan Daya selalu tersedia. Visibilitas peluncur dikontrol dalam modul Opsional; submenunya mengubah ikon khusus. Perubahan diterapkan setelah **Terapkan**, yang memulai ulang bilah tugas.

### Panel Kontrol

Panel Kontrol mengonfigurasi visibilitas dan urutan kartu tindakan cepatnya, ditambah sesi dan transparansinya sendiri.

Halaman terbuka dan menerima masukan segera setelah pengaturan kartu dibaca. Kartu yang bergantung pada perangkat keras (Bluetooth dan Kecerahan) diperiksa di latar belakang: hingga pemeriksaan tersebut selesai, kartu tersebut tetap memiliki ketersediaan yang terakhir terlihat di jendela ini, dan daftar diperbarui dengan sendirinya ketika hasilnya tiba.

### Widget Telemetri

Widget Telemetri mengonfigurasi sesi dan transparansi widget informasi sistem.

### FontFont dapat diperoleh dari **Penampilan > Gaya Desktop > Font**. Ini memungkinkan Anda memilih kelompok dan ukuran untuk bilah tugas, telemetri/informasi sistem, Panel Kontrol, antarmuka sistem ARGVUS, aplikasi, terminal, dan browser. Ini juga memperlihatkan pengaturan rendering font seperti antialiasing, petunjuk, mode subpiksel, dan DPI. Keluarga utama default adalah IBM Plex Mono; target individual dapat diubah secara independen dan dipulihkan dari halaman Font.

### Aplikasi bawaan

Aplikasi default memilih program yang diinstal untuk peran seperti terminal, pengelola file, editor teks, editor terminal, browser, penampil gambar, penampil PDF, pemutar video, pemutar audio, alat arsip, dan peluncur. Pilihan disimpan oleh ARGVUS dan dapat memperbarui asosiasi XDG standar. Manajer file TUI menggunakan editor terminal yang dipilih saat membuka file teks; editor teks grafis tetap menjadi default untuk asosiasi file desktop.

### Cuplikan

Cuplikan adalah halaman untuk teks yang diketik `argvus snippets` dengan `SUPER + ALT + N`. Itu ada di **Alat & Diagnostik → Cuplikan** (cari `snippets` atau jalankan `argvus control-center snippets`).
- **Cuplikan baru** — ketik **Nama** dan **Konten**, lalu **Simpan cuplikan**. Nama yang sudah ada menggantikan konten cuplikan tersebut.
- **Cuplikan tersimpan** — tercantum dengan posisinya (`1  email`) dan konten dalam satu baris. Masuk pada konten, edit itu. **Ketik** ketikkan cuplikan, dan **Hapus** meminta konfirmasi.
- **Open picker** — membuka picker yang sama dengan `SUPER + ALT + N`.

Pengetikan dimulai 3 detik setelah Anda menekan Enter, karena Pusat Kontrol sendiri yang fokus pada saat itu. Klik bidang yang akan menerima teks dalam jangka waktu tersebut. Halaman ini hanya mengedit konten satu baris; cuplikan multi-baris dapat dibuat di terminal dengan `argvus snippets add`.

Halaman tersebut membaca entri dari `argvus-config` dan menulisnya melalui `argvus snippets`, sehingga peluncur menyimpan validasinya. Ketika `argvus snippets` tidak diinstal, halaman tersebut menyatakan demikian.

### Kunci SSH

Kunci SSH adalah halaman untuk pasangan kunci yang dikelola `argvus keys`. Itu ada di **Alat & Diagnostik → Kunci SSH** (cari `ssh keys` atau jalankan `argvus control-center ssh-keys`).

Halaman ini memiliki empat bagian, dari atas ke bawah:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```

#### Cara menggunakannya

1. Buka **Pusat Kontrol → Alat & Diagnostik → Kunci SSH**. Dari Beranda, Anda juga dapat mencari `ssh keys`.
2. **Temukan kunci:** tekan `/`, lalu ketik untuk hanya menampilkan pasangan yang nama atau komentarnya cocok. `Enter` mengakhiri pencarian dan menjalankan baris yang disorot, seperti di daftar lainnya, jadi periksa baris tersebut sebelum menekannya; `Esc` menghapus pencarian.
3. **Buat pasangan:** di **Generate**, pilih **Name**, tekan `Enter`, ketik nama dan tekan `Enter` lagi. Pilih **Jenis** dan tekan `Enter` untuk beralih antara `ed25519` dan `rsa`. Pilih **Buat pasangan kunci** dan tekan `Enter`. Pasangan baru muncul di bawah **Kunci** sekaligus, tanpa frasa sandi.
4. **Salin kunci publik:** di blok pasangan, pilih **Salin kunci publik** dan tekan `Enter`. Isi file `.pub` masuk ke clipboard Wayland, siap ditempelkan ke `authorized_keys` server atau host Git. Kunci pribadi tidak pernah disalin. Ini membutuhkan `wl-clipboard` (`wl-copy`); tanpanya, halaman tersebut mengatakan kuncinya tidak disalin.
5. **Muat pasangan ke agen:** di blok pasangan, pilih **Muat ke agen** dan tekan `Enter`. Kunci tanpa frasa sandi dimuat sekaligus, tanpa jendela. Kunci dengan frasa sandi membuka terminal ARGVUS, tempat OpenSSH memintanya; tutup terminal dengan `Enter` ketika dikatakan kunci telah dimuat. Halaman ini memeriksa agen setiap beberapa detik saat terbuka, sehingga barisnya berubah menjadi **Bongkar dari agen** dengan sendirinya. Untuk memperbarui sekaligus, gunakan **Refresh** di **Actions**.6. **Hapus kunci dari agen:** di blok pasangan, pilih **Bongkar dari agen**. Untuk menghapus setiap kunci, gunakan **Bongkar semua kunci dari agen** di **Tindakan**. File kunci tetap ada di disk.
7. **Hapus pasangan:** di blok pasangan, pilih **Hapus** dan tekan `Enter`. Konfirmasi berfokus pada **Batal**, sehingga `Enter` yang tersesat tidak melakukan apa pun. Tekan `y` untuk konfirmasi, atau `n`/`Esc` untuk membatalkan. Kunci yang dimuat di agen tidak dapat dihapus; membongkarnya terlebih dahulu.

Bergerak: `↑`/`↓` atau `j`/`k` menelusuri daftar, `Enter` mengaktifkan satu baris, `Esc` kembali dan `?` menunjukkan bantuan. `Tab` memindahkan fokus ke **Tindakan**, dengan `←`/`→` memilih tombol dan `Enter` menjalankannya. `↑`/`↓` mengembalikan fokus ke daftar.

Frasa sandi tidak pernah diketikkan ke Pusat Kontrol dan tidak pernah diteruskan sebagai argumen perintah. Halaman membaca daftar dengan `argvus keys list` dan menulis hanya melalui `argvus keys`. Ketika `argvus-keys` tidak diinstal, halaman tersebut menyatakan demikian.

#### Pemecahan masalah

| Gejala | Penyebab dan perbaikan |
| --- | --- |
| `Could not open a connection to your authentication agent` setelah **Muat ke agen** | Terminal tidak menerima soket agen. Hyprland mempertahankan lingkungan awalnya, dan agen mungkin memulainya nanti. Perbarui `argvus-launcher`, yang kembali ke `$XDG_RUNTIME_DIR/ssh-agent.socket`, dan periksa `systemctl --user status argvus-ssh-agent.service`. |
| **Agen** menunjukkan `Agent unavailable` | Peluncur tidak dapat menjangkau agen. Periksa layanan seperti di atas. |
| Halaman tersebut bertuliskan `argvus-keys is not installed` | Perbarui paket `argvus`: `argvus keys` adalah sub-perintahnya. |
| **Hapus** menunjukkan kesalahan | Kuncinya dimuat di agen, dan disimpan. Bongkar dulu, lalu hapus lagi. |

### Kunci GPG

Kunci GPG adalah halaman untuk gantungan kunci yang dikelola `argvus gpg`. Itu ada di **Alat & Diagnostik → Kunci GPG** (cari `gpg keys` atau jalankan `argvus control-center gpg-keys`).

Halaman ini memiliki bagian berikut, dari atas ke bawah:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```

`▾` menandai bagian yang diperluas dan `▸` menandai bagian yang diciutkan. `Enter` atau `Space` memperluas atau menciutkannya.

#### Cara menggunakannya

1. Buka **Pusat Kontrol → Alat & Diagnostik → Kunci GPG**. Dari Beranda, Anda juga dapat mencari `gpg keys`.
2. **Pientry** ditetapkan ke `pinentry-gtk`. Halaman itu hanya menampilkannya. Pertama kali kunci dibuat, ARGVUS menulis pinentry tersebut ke `gpg-agent.conf`, di antara garis penandanya sendiri, dan menyimpan baris lainnya. Jika Anda telah menulis baris `pinentry-program` di luar penanda tersebut, pembuatan akan dihentikan dengan kesalahan sehingga Anda dapat menghapus baris tersebut.
3. **Buat kunci:** di **Buat kunci ▾**, ketik **Nama lengkap** dan **Email**. Keduanya wajib diisi, dan **Buat kunci** tetap dinonaktifkan hingga keduanya memiliki teks. **Komentar** adalah opsional. ID pengguna menjadi `Name (comment) <e-mail>`.
- **Algoritma** beralih antara **RSA** dan **DSA ElGamal**. Kunci DSA mendapat subkunci enkripsi ElGamal; kunci RSA mendapat kunci RSA.
- **Kekuatan utama** menelusuri ukuran algoritme: RSA 1024, 2048, dan 4096; DSA 1024, 2048 dan 3072. gpg tidak membuat kunci DSA di atas 3072 bit, jadi 4096 tidak ditawarkan untuk DSA.
- **Tanggal kedaluwarsa** membuka halamannya sendiri. Ketik tanggal dalam tata letak bahasa Anda: `DD/MM/YYYY` dalam bahasa Portugis, Spanyol, Prancis, Italia, Jerman, dan Rusia; `YYYY/MM/DD` dalam bahasa Inggris, Cina, Jepang, dan Korea. Tanggalnya harus setelah hari ini. Aktifkan **Tidak pernah kedaluwarsa** untuk menjadikan kunci permanen; bidang tanggal kemudian dinonaktifkan dan tidak digunakan.
- **Frase Sandi** membuka halamannya sendiri. **Lindungi dengan frasa sandi** diaktifkan secara default; ketik **Frase Sandi** lalu **Konfirmasi Frasa Sandi**. Keduanya harus cocok. Menonaktifkan perlindungan akan menghapus kedua kolom, dan halaman akan meminta konfirmasi sebelum menghasilkan kunci yang tidak dilindungi.
4. Pilih **Buat kunci** dan tekan `Enter`. Kunci tersebut mendapat kunci penandatanganan dan subkunci enkripsi. Frasa sandi dikirim ke peluncur pada input standarnya, tidak pernah sebagai argumen perintah. Jika pembuatan gagal, kolom lainnya tetap ada, namun frasa sandi dihapus dan harus diketik lagi.
5. **Batal** menghapus setiap bidang formulir dan mengembalikan tanggal dan ukuran ke defaultnya.
6. **Impor:** di **Impor ▾**, pilih **Impor kunci publik…** atau **Impor kunci pribadi…**. Manajer file terbuka sehingga Anda dapat memilih file. Kunci pribadi meminta konfirmasi terlebih dahulu, dan frasa sandinya diminta oleh gpg saat kunci tersebut digunakan. Jika pengelola file berbentuk grafis, ia tidak dapat melaporkan file ke halaman; halaman tersebut kemudian membuka bidang tempat Anda mengetikkan jalur.
7. **Tombol ›** membuka gantungan kunci. Setiap kunci menunjukkan identitasnya, sidik jari, jenis, masa berlakunya, dan apakah kunci pribadinya ada di dalam gantungan kunci. Setiap kunci memiliki tindakan berikut:
   - **Tombol Edit ›** membuka halaman kunci itu sendiri (dijelaskan di bawah).
- **Ekspor kunci publik…** meminta jalur file, diusulkan sebagai `~/<fingerprint>.asc`. File dibuat dengan mode `600`. File yang ada tidak pernah diganti.
- **Ekspor kunci pribadi…** hanya muncul untuk kunci dengan bagian pribadi. Ia meminta konfirmasi, lalu jalur file, diusulkan sebagai `~/<fingerprint>-private.asc`. File dibuat dengan mode `600`.
- **Salin kunci publik** dan **Salin sidik jari** salin ke papan klip Wayland dengan `wl-copy`.
- **Tombol hapus…** meminta konfirmasi, dengan fokus pada Batal. Ini menghilangkan kunci dan bagian pribadinya.
8. **Temukan kunci:** di halaman **Kunci**, bilah pencarian di bagian atas memfilter kunci. Ketik untuk mempersempit daftar; pencarian cocok dengan identitas (termasuk komentar) dan sidik jari. `Esc` menghapus pencarian.
9. **Tombol Edit ›** menampilkan detail kunci, lalu tindakan berikut:
   - **Tanggal kedaluwarsa ›** mengubah masa berlaku kunci dan setiap subkuncinya. Gunakan **Never expired** untuk menghapus masa berlakunya. **Terapkan** menyimpan kembaliannya.
- **Ubah frasa sandi** menanyakan frasa sandi lama dan baru di jendela masuk pintu masuk.
- **Tanda kunci dengan ›** mengesahkan kunci ini dengan salah satu kunci Anda lainnya yang memiliki bagian pribadi. Frasa sandi penandatangan diminta. Halaman tersebut meminta konfirmasi terlebih dahulu.
- **Cabut kunci ›** mencantumkan alasannya: tidak ada alasan, kunci disusupi, kunci digantikan, kunci tidak lagi digunakan. Masing-masing meminta konfirmasi. Kunci yang dicabut tidak dapat digunakan lagi.
- **Kepercayaan** menetapkan seberapa jauh Anda memercayai kunci: tidak ditetapkan, marjinal, penuh, atau pamungkas.
- **Publikasikan ke server kunci…** mengirimkan kunci publik ke server kunci yang disetel di `gpg.conf`, setelah konfirmasi. **Ambil dari server kunci** mengunduh kunci dari server kunci tersebut.

Bergerak: `↑`/`↓` atau `j`/`k` menelusuri daftar, `Enter` mengaktifkan satu baris, `Esc` kembali dan `?` menunjukkan bantuan.

Frasa sandi diketik hanya ke dalam kolom bertopeng pada halaman frasa sandi, atau ke dalam jendela pintu masuk. Halaman tersebut mengirimkan frasa sandi pembuatan ke `argvus gpg` pada input standar, dan peluncur menyerahkannya ke gpg pada deskriptor file, sehingga tidak pernah muncul dalam daftar proses, dalam argumen perintah, atau di log. Perubahan penting berjalan di latar belakang; sementara yang satu berlari, yang lain menunggu. Halaman membaca keyring dengan `argvus gpg list` dan menulis hanya melalui `argvus gpg`. Ketika `argvus gpg` tidak diinstal, halaman tersebut mengatakan demikian.

Belum ada di halaman: mencari kunci di server kunci berdasarkan nama atau email. Gunakan **Ambil dari server kunci** dengan sidik jari yang dikenal.#### Pemecahan masalah

| Gejala | Penyebab dan perbaikan |
| --- | --- |
| Halaman tersebut bertuliskan `argvus gpg is not available` | `argvus-launcher` yang diinstal mendahului perintah GPG. Jalankan `argvus gpg list` di terminal: `unknown command` berarti paket harus dibangun kembali dan diinstal. Kunci juga membutuhkan `gnupg`. |
| **Buat kunci** tetap dinonaktifkan | Ketik **Nama lengkap** dan **Email**. |
| Baris status mengatakan `argvus-gpg: pinentry-gtk is not installed` | Instal paket `pinentry`, yang menyediakan `pinentry-gtk`. |
| Pembuatan berhenti dengan kesalahan `pinentry-program` | `gpg-agent.conf` menetapkan pinentry di luar blok ARGVUS. Hapus baris itu; ARGVUS kemudian menulis sendiri. |
| **Pintu masuk** menunjukkan `Not set` | gpg-agent menggunakan default bawaannya hingga kunci pertama dihasilkan dari halaman. |
| Halaman tersebut mengatakan `argvus-launcher` yang diinstal lebih tua dari Pusat Kontrol | Instal ulang `argvus-launcher`: jalankan `make build` lalu `make install` di repositorinya, dan buka kembali halaman tersebut. Tindakan utama tetap tidak aktif hingga versinya cocok. |
| Manajer file terbuka, tetapi halaman tersebut meminta jalur | Manajer file adalah manajer grafis dan tidak dapat melaporkan file yang dipilih. Ketik jalur di bidang. |
| **Salin kunci publik** menyatakan tidak dapat menyalin | `wl-copy` tidak diinstal atau sesi Wayland tidak tersedia. Instal `wl-clipboard`. |
| **Kunci Edit ›** menyatakan bahwa kunci sudah tidak ada lagi di gantungan kunci | Gantungan kunci berubah sejak halaman dibuka. Kembali ke **Tombol ›** dan segarkan. |
| Baris status menunjukkan `argvus-gpg: ...` | Peluncur menolak tindakan tersebut, dan teks memberikan alasannya, seperti masa berlaku yang tidak valid, file yang sudah ada, atau `--yes` yang hilang. |
| Suatu tindakan menunggu dan tidak terjadi apa-apa | Jendela pintu masuk disembunyikan atau tidak dibuka. Periksa sesi mengekspos `WAYLAND_DISPLAY` atau `DISPLAY`. |
| Impor ditolak | File tersebut tidak berisi kunci dari jenis yang dipilih. Kunci publik melalui **Impor kunci publik…**; kunci pribadi melalui **Impor kunci pribadi…**. |

### Bahasa & Wilayah

Kartu ini mencantumkan Bahasa (lokal regional), Zona Waktu, Tanggal & Waktu, Lokal Sistem, dan Tata Letak Keyboard, dan masing-masing dapat dibuka langsung dari rumah. Halaman keyboard juga berisi varian dan peta tombol konsol. Pencarian beranda juga menampilkan pintasan keyboard dan pengaturan mouse/touchpad sebagai rute pengaturan langsung.

Daftar Bahasa kartu ini ( ARGVUS bahasa antar-muka), Locale Regional, Zona Waktu, Tanggal & Waktu, Locales Sistem dan Tata Letak Papan Tik. Bahasa membuka pemilih 18- bahasa; Locale Regional mengatur lokal yang digunakan untuk format regional dan aplikasi partai ketiga.

* * Preferensi > Bahasa & Daerah > Bahasa * * menghemat `ARGVUS_LANGUAGE` in `$XDG_CONFIG_HOME/locale.conf` (biasanya `~/.config/locale.conf`), misalnya `ARGVUS_LANGUAGE=pt-PT`Ini berubah ARGVUS aplikasi dan permukaan ketika memelihara `LANG` untuk format daerah dan aplikasi pihak ketiga. Panel Kendali segera dimulai kembali; memulai sesi baru untuk menerapkan bahasa di seluruh komponen sesi. Memilih bahasa antar muka tidak memerlukan otorisasi administrator atau `locale-gen`.

* * Regional Locale * * menghemat `LANG` dalam berkas yang sama dan mengontrol format regional dan bahasa yang digunakan oleh aplikasi pihak ketiga. Its dipilih UTF-8 lokal sistem harus sudah dihasilkan.

* * Locales Sistem * * berubah `/etc/locale.gen` dan berjalan `locale-gen`, sehingga menerapkannya membutuhkan otorisasi administrator.

* * Regional Locale * * menyimpan `LANG` dalam berkas yang sama dan mengontrol format regional dan bahasa yang digunakan oleh aplikasi pihak ketiga. lokal sistem yang dipilih UTF-8 harus sudah dihasilkan.

**Lokal Sistem** mengubah `/etc/locale.gen` dan menjalankan `locale-gen`, sehingga penerapannya memerlukan otorisasi administrator. Pemberitahuan **[PERINGATAN]** yang terus-menerus di atas footer mengidentifikasi halaman dengan pengaturan administratif. Pemberitahuan yang sama mencakup zona waktu, tanggal/waktu, nama host, peta kunci konsol, pengeditan akun dan grup, firewall, layanan sistem, pengatur CPU, profil dan kebijakan daya, konfigurasi boot, operasi paket, snapshot, dan perubahan jaringan yang dilindungi. Preferensi pribadi, layanan pengguna, dan halaman hanya-baca tidak menerima pemberitahuan ini. Pemberitahuan ini bersifat informatif; otorisasi diminta hanya ketika suatu tindakan memerlukannya.

### Perangkat keras dan tampilanKetika kemampuan yang sesuai dipasang, Pusat Kontrol dapat menampilkan informasi perangkat keras seperti CPU, GPU, memori, daya, dan perangkat. Halaman masukan mengonfigurasi perilaku mouse dan touchpad; area tampilan menangani resolusi monitor, kecepatan refresh, skala, posisi, orientasi, tampilan utama, VRR dan HDR jika didukung oleh sesi berjalan. Perubahan tampilan segera diterapkan. Setelah perubahan yang dapat membuat layar tidak dapat digunakan (resolusi atau kecepatan refresh, posisi, pencerminan, menonaktifkan monitor, kedalaman warna) atau setelah **Terapkan**, Pusat Kontrol menanyakan apakah akan menyimpannya, dengan fokus pada **Kembalikan**: tekan `y` untuk menyimpannya; `n`, `Esc` atau `Enter` mengembalikannya, dan tanpa jawaban dalam waktu 15 detik konfigurasi sebelumnya kembali secara otomatis. Menghapus konfigurasi tersimpan dari monitor yang terputus dan menghapus profil meminta konfirmasi; profil membuka halamannya sendiri dengan **Terapkan profil**, **Ganti nama** dan **Hapus profil**. Tata letak disimpan melalui `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` dan `monitor-remove`, sehingga bagian `/displays` kanonik dari `config.json` adalah satu-satunya sumber tata letak monitor. Hanya katalog profil yang tetap berada di `display/config.toml`.

### Konektivitas dan audio

Area jaringan memperlihatkan status penyedia jaringan yang diinstal dan halaman koneksi, termasuk Wi-Fi, Ethernet, VPN, DNS, dan rute proxy bila tersedia. Firewall tercantum di **Administrasi Sistem**. Bluetooth bergantung pada dukungan Bluetooth. Area audio disediakan ketika kemampuan opsionalnya diaktifkan dan terintegrasi dengan layanan audio yang diinstal.

Beranda **Jaringan & Wi-Fi** dan laman **Status**nya terisi saat setiap kueri NetworkManager menjawab, bukan setelah yang paling lambat: daftar perangkat terlebih dahulu, lalu radio, alamat antarmuka yang terhubung dan DNS, koneksi tersimpan, dan jaringan Wi-Fi yang sudah diketahui NetworkManager (pemindaian baru hanya dijalankan dari halaman Wi-Fi dengan `r`). Hingga daftar perangkat tiba, baris bertuliskan *Memuat...*, dan sakelar Wi-Fi hanya ditawarkan ketika komputer memiliki kartu Wi-Fi. Memulai tindakan (pengalih Wi-Fi, menyambungkan, DNS) saat halaman disegarkan akan menggantikan penyegaran, dan halaman disegarkan lagi saat tindakan berakhir.

#### Koneksi, terinspirasi oleh nmtui

**Jaringan & Wi-Fi > Koneksi** mencantumkan setiap profil koneksi NetworkManager, dikelompokkan berdasarkan jenis, dengan perangkat yang aktif di dalamnya (`★ Active · eno1`), apakah itu port bridge, bond, atau tim, dan apakah itu hanya baca. `Enter` membuka profil di editor, `a` menghubungkan atau memutusnya, `/` memfilter berdasarkan nama dan `r` menyegarkan. **Aktifkan koneksi** adalah daftar yang sama di mana `Enter` menaikkan atau menurunkan profil, seperti di `nmtui`. Halaman detail antarmuka, jaringan Wi-Fi tersimpan, dan VPN memiliki baris **Edit koneksi** yang membuka profilnya.

**Tambahkan koneksi** membuat profil Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Bridge, Bond, Team, VLAN, terowongan IP, Loopback dan WireGuard, dan satu profil VPN per plugin NetworkManager VPN yang diinstal (OpenVPN memiliki bentuknya sendiri; plugin lain mengedit pengaturannya sebagai entri `key=value`). Itu juga mengimpor WireGuard `.conf` atau file plugin seperti `.ovpn` (`nmcli connection import`). Tim memerlukan paket `libteam`, DSL memerlukan `ppp` dan VPN memerlukan pluginnya (misalnya `networkmanager-openvpn`); tanpa mereka, baris tersebut dinonaktifkan dan mengatakan apa yang harus dipasang. WEP, InfiniBand, MACsec dan Open vSwitch tidak ditawarkan.Editor adalah satu daftar bagian yang dapat dilipat: **Umum** (nama, antarmuka, terhubung secara otomatis, prioritas, tersedia untuk semua pengguna, terukur), pengaturan jenisnya sendiri, **IPv4** dan **IPv6** (metode, alamat, gateway, server DNS, domain pencarian, rute, abaikan DNS atau rute otomatis, jangan pernah digunakan sebagai rute default, metrik rute, ekstensi privasi, pembuatan alamat). Pilihan dan angka berubah dengan `←/→`; `Enter` mengetikkan nilai; daftar (alamat, DNS, rute, IP yang diizinkan) membuka halamannya sendiri, tempat `Enter` mengedit entri, menghapusnya akan menghapusnya dan **Tambahkan entri** menambahkan satu. Bidang yang tidak berlaku untuk metode yang dipilih ditampilkan dinonaktifkan. Bridge, bond, atau tim memilih **Port** Ethernetnya di subhalaman; WireGuard mengedit **Peers** pada subhalaman dan dapat **Menghasilkan pasangan kunci** (dengan `wg`, dari `wireguard-tools`, ketergantungan paket `argvus`); kunci publik dari kunci privat ditampilkan di bawahnya.

Tidak ada yang mencapai NetworkManager hingga **Simpan**, yang tetap dinonaktifkan saat tidak ada yang berubah atau saat kolom tidak valid, lalu menyatakan kolom mana dan alasannya. Hanya pengaturan yang diubah saja yang tertulis. Menyimpan koneksi yang aktif ditanyakan terlebih dahulu lalu menyambungkannya kembali, sehingga pengaturan baru berlaku sekaligus (hal ini dapat mengganggu jaringan sebentar). `Esc` dengan perubahan yang belum disimpan bertanya sebelum membuangnya. **Hapus koneksi**, di zona Bahaya, meminta konfirmasi dengan fokus pada Batal; menghapus jembatan, ikatan atau tim juga menghapus portnya.

Kata sandi, kunci pribadi, dan kunci yang dibagikan sebelumnya tidak pernah dibaca kembali: barisnya bertuliskan *Tidak Berubah* hingga nilai baru diketik, dan nilai baru ditampilkan secara terselubung. Mereka ditulis melalui input standar `nmcli connection edit`, tidak pernah sebagai argumen perintah. Perubahan dijalankan sebagai pengguna sesi: anggota lokal `wheel` diizinkan oleh aturan polkit NetworkManager, dan pengguna lain dimintai kata sandi administrator oleh agen polkit sesi.

### Kekuatan dan sesi

Power menyediakan kontrol daya sistem dan halaman kebijakan yang tersedia untuk instalasi, termasuk tombol Tetap Terjaga. Saat diaktifkan, Tetap Terjaga mencegah kebijakan menganggur ARGVUS memulai kunci layar otomatis dan pengatur waktu daya tampilan. Sesi memperlihatkan status sesi ARGVUS, komponen, mulai otomatis, diagnostik, dan log. Pada **Power**, bilah **Actions** di bawah daftar berisi **Suspend**, **Hibernate** dan **Refresh** (`r` juga disegarkan): `Tab` atau `←/→` pindah ke sana, `Enter` menjalankan tombol fokus, `↑`/`↓` mengembalikan fokus ke daftar, dan `Esc` kembali. Suspend dan Hibernate minta konfirmasi terlebih dahulu. Halaman-halaman ini mungkin memerlukan izin sistem dan harus dibedakan dari tindakan cepat di Panel Kontrol.

### Alat sistem

Bagian sistem dapat memaparkan informasi boot, paket, layanan, penyimpanan, diagnostik, administrasi pengguna/grup, dan informasi sistem. Ini adalah alat administratif atau diagnostik; suatu halaman mungkin hanya dapat dibaca atau memerlukan otorisasi tergantung pada pengoperasiannya.

**Layanan** memiliki empat halaman: unit sistem, unit pengguna, unit gagal, dan jurnal (Log). Setiap daftar unit menunjukkan garis `Filter:`, yang disikluskan `f`, dan kotak `Search:` yang dibatasi, dibuka dengan `/`. Setiap unit menunjukkan status Aktif dan Diaktifkan sebagai titik terisi atau kosong. Di Log, `b` beralih antara boot saat ini dan sebelumnya, `u` memutar unit dan `p` memutar prioritas; baris `Filter:` menunjukkan nilai yang digunakan. `Enter` membuka unit atau entri log.**Layanan Dev** melaporkan tiga sumber yang berhubungan dengan pengembangan dalam satu halaman baca-saja: layanan systemd pengguna saat ini, port TCP yang sedang mendengarkan (`ss`), dan container yang berjalan dari runtime container yang diinstal (Podman, Docker). Ini hanya muncul di grid Beranda ketika setidaknya salah satu dari ini tersedia di sistem — mesin tanpa sesi pengguna systemd, tanpa Podman, dan tanpa Docker tidak menampilkan entri. `r` menyegarkan ketiga sumber secara independen, sehingga sumber yang lambat atau gagal (misalnya runtime container yang berhenti merespons) tidak menghalangi sumber lain untuk menampilkan datanya.

**Paket** dikelompokkan berdasarkan sumber. **Repositori Resmi** dan, ketika pembantu AUR (`paru` atau `yay`) diinstal, **AUR (Arch User Repository)** masing-masing terbuka **Instal**, **Hapus** dan **Perbarui**; **Cache**, **Riwayat**, **Penurunan Versi** dan **Cermin** menyusul. Ketiga tugas tersebut berbagi satu layar: kotak **Pencarian** yang terletak di bagian atas mengambil huruf yang Anda ketik, dan paket dicantumkan di bawahnya dalam blok **Hasil** (**Pembaruan** pada Pembaruan), dengan daftar kosong yang menyatakan alasannya (misalnya **Tidak ada hasil**, **Tidak ada item yang cocok**, atau **Tidak ada pembaruan yang menunggu keputusan**). **Instal** mencari repositori (`pacman -Ss`) atau AUR ketika Anda menekan `Enter` di bilah Pencarian atas; **Hapus** mencantumkan paket-paket yang diinstal (daftar AUR menunjukkan paket-paket yang dibuat di luar repositori, `pacman -Qm`, dan ketika menu AUR ditampilkan, paket-paket tersebut keluar dari daftar Resmi); **Pembaruan** mencantumkan pembaruan yang tertunda (`pacman -Qu`, atau `paru -Qua`/`yay -Qua` untuk AUR). Pada sebuah paket, `Enter` menjalankan tugas layar dan `→` membuka halaman paket, dengan **Instal** (atau **Instal Ulang**) dan, di **Zona bahaya**, **Hapus**; `←` atau `Esc` kembali. **Repositori Resmi → Hapus** dimulai dengan **Yatim Piatu**, dengan `Space` menandai paket (`[x]`), `Enter` membuka satu paket dan **Hapus yang ditandai** menghapusnya bersama-sama. Dari daftar Hasil pembaruan, `Tab` berpindah ke bilah **Tindakan** di bawah daftar (**Tingkatkan semua** dan **Segarkan basis data**, atau **Tingkatkan semua (AUR)** di AUR), `←`/`→` berpindah di antara tombol-tombolnya, `Enter` menjalankan tombol fokus dan `↑`/`↓` (atau mengetik) mengembalikan fokus ke daftar; **Tingkatkan semua** diredupkan dan dilewati saat tidak ada pembaruan yang terdaftar. Breadcrumb dan pencarian global mengikuti jalur yang sama, misalnya **Paket > Repositori Resmi > Hapus > Yatim Piatu**. Pembersihan cache berada di zona Bahaya pada halaman Cache, dan **Cermin → Konfigurasikan cermin** menyetel opsi reflektor sebelum **Buat pratinjau**. Setiap operasi paket menunjukkan rencananya, atau untuk AUR peringatan PKGBUILD, dan meminta konfirmasi (`y` mengonfirmasi, `n` atau `Esc` dibatalkan) sebelum dijalankan, dan outputnya ditampilkan saat dijalankan. Operasi AUR menjalankan pembantu sebagai pengguna Anda (`-S <package>` untuk menginstal atau memperbarui satu, `-Sua` untuk memperbarui semua); Penghapusan AUR menggunakan `pacman -Rns` yang sama dengan paket resmi. Ketika paket terinstal lainnya bergantung pada paket yang Anda hapus (resmi atau AUR), konfirmasi akan menamainya dan menawarkan dua cara: **Hapus saja** paket yang dipilih (`pacman -Rdd`, yang melewatkan pemeriksaan ketergantungan, sehingga paket yang memerlukannya mungkin berhenti bekerja) atau **Hapus dengan tanggungan** (`pacman -Rcns`, dengan jumlah paket yang dihapus). Fokusnya dimulai pada **Batal**, yang mempertahankan setiap paket; `Enter` menjalankan baris terfokus dan tidak ada pintasan `y` dalam pilihan ini.Bilah **Penelusuran** bagian atas memiliki kueri paket. `/` memfokuskannya tanpa menghapus teksnya; mengetik juga memindahkan fokus ke sana. Edit dengan tombol panah, `Home`, `End`, `Backspace`, `Delete` atau tempel. Dalam **Instal**, `Enter` mengirimkan repositori/kueri AUR bahkan ketika hasil sebelumnya ada, lalu memindahkan fokus ke hasil; penelusuran resmi memerlukan kueri yang tidak kosong dan penelusuran AUR memerlukan setidaknya dua karakter. Mengedit kueri Instal akan membatalkan permintaan yang tertunda dan menghapus hasil lama; permintaan baru menunggu `Enter`. Di **Remove**, **Update** dan **Orphans**, pengeditan langsung memfilter daftar yang dimuat tanpa kueri repositori lain. `↓` berpindah dari Penelusuran ke hasil, `↑` pada hasil pertama kembali ke Penelusuran, dan `Esc` di Penelusuran kembali ke daftar sambil mempertahankan kueri. `Enter` pada hasil melakukan tindakan paket itu. Bilah tindakan Pembaruan tetap dapat dijangkau oleh `Tab`, dan `/` kembali dari sana ke Penelusuran.

Semua operasi di **Paket** membuka terminal interaktif terintegrasi: paket resmi, AUR, pembersihan cache, mirror, dan repositori/perubahan kunci. Jawab perintah program dengan mengetik dan menekan `Enter`; pacman berjalan tanpa `--noconfirm` di terminal ini. `Esc` menyembunyikan tampilan dan menjaga proses tetap berjalan. `F8` membuka kembali sesi yang sama dari halaman Paket mana pun; **Tampilkan terminal** juga tersedia di menunya. Footer menunjukkan apakah transaksi sedang berjalan, selesai atau gagal. Setelah perintah berhasil, `Done!` muncul di dalam terminal setelah keluaran akhirnya, termasuk ketika terminal disembunyikan. `Shift+PageUp` / `Shift+PageDown` menggulir riwayat, `Shift+End` kembali ke keluaran langsung, dan `Ctrl+C` mengirimkan interupsi ke proses anak. Saat terminal terfokus, `q` dan `?` merupakan input, bukan pintasan global. Output, warna ANSI, dan kursor dipertahankan, dan pengubahan ukuran akan memperbarui terminal anak. Riwayat menyimpan hingga 10.000 baris hingga operasi berikutnya atau aplikasi keluar. Hanya satu transaksi yang berjalan dalam satu waktu; menyembunyikan terminalnya tidak memungkinkan aplikasi berhenti saat sedang berjalan. Tidak ada shell gratis yang dibuka.

Pratinjau pemutakhiran membaca database sinkronisasi yang ada (`pacman -Su --print`) tanpa mengubahnya atau memerlukan root. Setelah konfirmasi, pemutakhiran penuh berjalan `pacman -Syyu` melalui pembantu istimewa dengan `pkexec`; sinkronisasi dapat mengungkapkan pembaruan tambahan. Instalasi paket, penghapusan, penyegaran basis data, pembersihan cache, penurunan versi, perubahan repositori/kunci, dan penulisan cermin juga menggunakan bantuan istimewa tersebut. Pembantu AUR terus berkembang sebagai pengguna Anda.**Repositori Resmi → Repositori** menambahkan dan mengelola repositori pacman tidak resmi. **Tambahkan** adalah draf: pilih **Sumber** konfigurasi (**URL** file `.conf`, **Teks** diketik atau ditempel dalam editor multi-baris, atau **Formulir** dengan **Nama**, **Server** dan **SigLevel**, ditulis dalam tata letak Arch Wiki) dan, di bawah **Kunci penandatanganan**, **Kunci** dari **URL** atau **Teks** (atau **Tidak Ada**). Pilihan berpindah dengan `←`/`→` atau `Enter`; bidang teks membuka editor tempat `Enter` menyimpan satu baris, `Ctrl+S` menyimpan beberapa baris, penempelan berfungsi dan `Esc` dibatalkan. Unduhan hanya menerima alamat `https://`. **Tinjau dan tambahkan** mengunduh dan memeriksa semuanya sebagai pengguna Anda dan kemudian meminta konfirmasi, mencantumkan repositori, server dan SigLevel serta ID pengguna dan sidik jari kunci dalam kelompok empat, untuk membandingkan dengan yang diterbitkan oleh repositori. Mengonfirmasi impor kunci dan menandatanganinya secara lokal (`pacman-key --add`, `--lsign-key`), menulis `/etc/pacman.d/<name>.conf` (dinamai berdasarkan bagian pertamanya) dan menambahkan `Include = /etc/pacman.d/<name>.conf` ke `/etc/pacman.conf` (baris komentar yang ada untuk file yang sama diaktifkan sebagai gantinya). Setelah itu Pusat Kontrol menawarkan `pacman -Syyu`, sehingga database baru disinkronkan tanpa peningkatan sebagian. Membiarkan **Tambahkan** dengan draf bertanya sebelum membuangnya. **Daftar** menampilkan setiap repositori tidak resmi, yang ditulis dalam `pacman.conf` itu sendiri atau disertakan dari `/etc/pacman.d/`, diaktifkan atau dinonaktifkan; bagian resmi tidak pernah terdaftar. Halaman repositori menampilkan filenya, SigLevel, server dan kunci yang direkam, tombol **Diaktifkan** dan, di **Zona bahaya**, **Hapus**. Menonaktifkan komentar pada baris `Include`, atau mengawali setiap baris bagian sebaris dengan `#argvus-off `; menghapus menghapus baris atau bagian dan file yang disertakan. Repositori dari file yang disertakan sama diaktifkan, dinonaktifkan, dan dihapus secara bersamaan. Saat Pusat Kontrol mengimpor kunci, **Hapus** juga menawarkan penghapusannya dari gantungan kunci pacman (`pacman-key --delete`). Setiap perubahan pada `/etc/pacman.conf` menyimpan cadangan di sebelahnya (`pacman.conf.argvus-backup-…`), dan paket yang diinstal tidak pernah dihapus.

### Pengguna dan grup

Ketika penyedia akun tersedia, buka **Pusat Kontrol → Administrasi Sistem → Pengguna & Grup → Pengguna** atau cari **pengguna**. Antarmuka dapat mencantumkan akun normal dan sistem, membuat pengguna, mengedit metadata akun, mengelola grup tambahan dan utama, mengubah atau mengunci kata sandi, membuka kuncinya, memerlukan perubahan kata sandi pada login berikutnya, mengatur atau menghapus avatar, dan mengaktifkan login otomatis.

Pembuatan pengguna meminta nama pengguna, nama lengkap, shell, grup tambahan, dan konfirmasi kata sandi opsional. Kata sandi kosong membuat akun dengan kata sandi terkunci; itu tidak secara diam-diam membuat kata sandi yang dapat digunakan. Halaman pengguna mengelompokkan bidang akun (nama lengkap, shell, grup utama dan tambahan, diikuti dengan **Simpan perubahan**), tombol beralih **Login otomatis**, tindakan **Kata Sandi** (mengubah, mengunci, membuka kunci, memerlukan perubahan pada login berikutnya) dan tindakan **Avatar**. Nama pengguna, UID / GID dan direktori home ditampilkan sebagai informasi saja. **Zona bahaya** di bagian akhir berisi tindakan destruktif terpisah untuk menghapus akun sambil mempertahankan direktori home-nya atau menghapus direktori home juga. Baca teks konfirmasi dengan cermat sebelum memilih yang terakhir. Operasi akun memiliki hak istimewa dan dapat membuka perintah otorisasi sistem. Avatar dapat digunakan oleh penyambut melalui integrasi gambar wajah standar akun.**Login otomatis** memulai sesi akun yang dipilih pada boot berikutnya tanpa menampilkan layar login, konvensi yang sama yang digunakan GNOME, LightDM, dan SDDM untuk fitur ini: tidak ada kata sandi yang disimpan di mana pun. Mengaktifkannya untuk satu akun akan menonaktifkannya untuk akun yang dikonfigurasi sebelumnya (hanya satu akun yang dapat masuk otomatis dalam satu waktu). Ini berlaku sekali, pada boot berikutnya; setelah logout manual, layar login normal kembali untuk sisa sesi itu. Tombol ini diterapkan segera, dengan konfirmasi konfirmasi, dan hanya tersedia untuk administrator.

Grup memiliki daftar dan halaman editnya sendiri, termasuk manajemen anggota dan penghapusan grup. Pusat Kontrol tidak menggantikan kebijakan akun sistem: akun sistem yang dilindungi dan operasi yang memerlukan otorisasi tetap tunduk pada batasan sistem operasi.

### Pengaturan Pusat Kontrol

**Pengaturan** adalah entri terakhir di rumah. Halaman **Bahasa** mengatur bahasa pengguna melalui locale.conf, dan halaman **Gaya** mengontrol ikon dekoratif dan transparansi Pusat Kontrol. Gaya tidak mengandung keburaman: keburaman adalah pengaturan Hyprland global, di bawah Hyprland > Blur. Halaman Gaya mengubah presentasi aplikasi pengaturan; ini bukan saklar yang menonaktifkan layanan ARGVUS atau menghapus penyedia pengaturan. Menyimpan ikon mungkin memerlukan otorisasi, sementara perubahan visual diterapkan pada aplikasi saat ini. Esc pada Gaya kembali ke halaman Pengaturan, dan Esc di sana kembali ke beranda.

### Tentang ARGVUS

**Tentang ARGVUS** ada di **Alat & Diagnostik → Tentang ARGVUS** (cari `about`, `hardware` atau `firmware`, atau jalankan `argvus control-center about`). Ini memiliki lima tab, diganti dengan `Tab`/`Shift+Tab`, `←`/`→` atau `h`/`l`: **Sistem & Perangkat Keras**, **Tentang**, **Sumbang**, **Kredit** dan **Hak Cipta**. Header menunjukkan path lengkap, misalnya `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**Sistem & Perangkat Keras** bersifat hanya baca dan mengelompokkan fakta mesin ini dalam tiga bagian, dengan nilai yang disejajarkan dalam satu kolom:

| Bagian | Bidang |
| --- | --- |
| sepatu bot | Firmware (vendor, versi dan tanggal), Jenis (UEFI atau Legacy (BIOS)), Boot Aman (hanya UEFI), Virtualisasi, Bootloader |
| Sistem | ARGVUS, Nama Host, Sistem Operasi, Distributor, Kernel, Arsitektur, Window manager (Hyprland), Compositor (Wayland), driver GPU, GTK, Tema (dengan Gelap atau Terang), Tampilan (satu baris per monitor), Audio (PipeWire), Shell (Quickshell), Login (greetd), Init (systemd), Paket, Uptime, Lokal |
| Perangkat Keras | Baterai (satu baris per baterai), CPU (dengan frekuensi maksimum), Thread, Core, GPU, Memori (bekas / total), Swap, Penyimpanan, Model (hanya jika berbeda dengan motherboard), Motherboard |

Setiap nilai dibaca dari sistem yang sedang berjalan, tanpa hak istimewa, dan halaman berfungsi sama di mesin mana pun. Nilai yang tidak dapat dibaca ditampilkan sebagai `N/A`, dan baris yang tidak berlaku akan diabaikan:

- **Firmware**, **Model** dan **Motherboard** berasal dari DMI (`/sys/class/dmi/id`); nilai placeholder seperti `To be filled by O.E.M.` diabaikan. **Jenis** dan **Boot Aman** berasal dari `/sys/firmware/efi`, dan **Virtualisasi** dari `systemd-detect-virt`.
- **Bootloader** dibaca dari variabel EFI `LoaderInfo`, yang systemd-boot, Limine, dan bootloader lain yang mengikuti publikasi Antarmuka Boot Loader. GRUB tidak mempublikasikannya, sehingga dikenali oleh `/boot/grub` (atau `/efi/grub`) dan ditampilkan dengan versi paket `grub`.
- Versi ARGVUS, Wayland, Mesa, PipeWire, Quickshell,greed, systemd dan GRUB berasal dari satu kueri `pacman -Q` dan ditampilkan tanpa epoch dan rilis paket (`1:25.2.3-1` menjadi `25.2.3`). Versi Hyprland berasal dari `hyprctl version`, atau dari paket ketika tidak ada sesi yang menjawab.
- **Driver GPU** memberi nama driver yang terikat pada setiap kartu DRM di `/sys/class/drm`. Driver berpemilik menunjukkan versi modulnya (misalnya `nvidia 580.82.07`); driver dalam kernel menunjukkan versi Mesa (misalnya `amdgpu (Mesa 25.2.3)`).
- **Tampilan** mencantumkan monitor aktif dari `hyprctl monitors -j`: konektor, resolusi, kecepatan refresh, dan skala (skala dibiarkan di angka 1).
- **Paket** menghitung semua paket yang diinstal, yang diinstal secara eksplisit dan yang asing (AUR atau dibuat secara lokal), dari `pacman -Qq`, `-Qqe` dan `-Qqm`.
- **Uptime** berasal dari `/proc/uptime` dan merupakan nilai saat halaman dibuka. **Lokal** berasal dari `LANG` di `/etc/locale.conf`.
- **Baterai** mencantumkan baterai komputer dari `/sys/class/power_supply`, dipilih berdasarkan jenisnya, sehingga periferal seperti mouse nirkabel tidak dihitung. Setiap baris menunjukkan pengisian daya, status, kesehatan (kapasitas penuh dibandingkan dengan kapasitas desain) dan siklus pengisian daya saat firmware melaporkannya. Mesin tanpa baterai menunjukkan **Tidak terdeteksi**.
- **CPU** menambahkan frekuensi maksimum tertinggi di antara semua inti (cpufreq), yang merupakan nilai maksimum pada prosesor hibrid; itu ditinggalkan jika cpufreq tidak tersedia, seperti di sebagian besar mesin virtual. **Thread** dan **Core** berasal dari `/proc/cpuinfo`.
- **Memori** dan **Swap** berasal dari `/proc/meminfo` (termasuk zram); sistem tanpa pertukaran menunjukkan **Dinonaktifkan**.
- **Penyimpanan** menjelaskan disk di bawah `/`: modelnya, jenisnya (NVMe, SSD, HDD atau eMMC/SD; disk virtual tidak menunjukkan jenisnya), sistem file, serta ruang yang digunakan dan total. Disk ditemukan melalui partisi, LUKS dan LVM hingga ke perangkat fisik.

Nilai-nilai tersebut dibaca di latar belakang saat pertama kali halaman digambar, sehingga baik Pusat Kontrol maupun pembukaan halaman tidak menunggunya; tab menunjukkan **Memuat informasi sistem...** sementara itu. Tekan `c` pada tab ini untuk menyalin seluruh laporan sebagai teks biasa ke clipboard (dengan `wl-copy`), misalnya untuk menempelkannya di laporan bug.

## Pencarian dan kemampuan opsional

Layar beranda memiliki daftar rute yang dapat dicari. Istilah pencarian mencakup konsep yang dilihat pengguna seperti tampilan, tema, wallpaper, font, pintasan keyboard, mouse, touchpad, tampilan, jaringan, dan sesi. Penyedia opsional disertakan hanya ketika fitur mereka dikompilasi dan kemampuan runtime mereka terdeteksi, sehingga menu yang terlihat sengaja dibuat sadar akan kemampuan.

Saat pencarian Beranda aktif, kunci yang dapat dicetak dimasukkan ke dalam kueri, termasuk `j` dan `k`; gunakan tombol panah `↑` dan `↓` untuk berpindah melalui rute yang cocok. Tekan `Enter` untuk membuka rute yang dipilih, `Backspace` untuk mengedit kueri, dan `Esc` untuk menghapus kueri atau keluar dari mode pencarian.

## Alur kerja penampilan

Buka **Appearance** untuk melihat kontrol visual terintegrasi. Kelompoknya adalah:

- **Desktop** — **Tema** (pilih kelompok paket dan mode Lengket atau Spasi, atau ekspor, impor, terapkan, dan hapus tema Anda sendiri, lihat [Mengimpor dan mengekspor tema](/docs/components/argvus-themes/import-and-export/)), **Mode Penampilan** (Lengket atau Spasi, tidak bergantung pada tema), **Warna Aksen** (aksen gabungan atau warna RGB enam digit yang valid), **Wallpaper** (paket wallpaper atau file khusus) **Ikon** (tema ikon yang terpasang: cari daftar dengan `/`, pilih salah satu dengan Spasi atau Enter, lalu pilih **Terapkan**; tidak ada yang berubah sebelum Terapkan) dan **Font** (target font dan pengaturan rendering font, dibuka di halaman Font).
- **Gaya Aplikasi** — transparansi **Terminal** dan **Peluncur**.Kartu Penampilan menampung permukaan yang menggunakan pengaturan ini: **Telemetri Widget** (aktifkan permukaan telemetri, pilih blok yang tersedia dan susun ulang dengan `Shift+Up`/`Shift+Down` pada blok terfokus) dan **Panel Kontrol** (aktifkan, nonaktifkan, dan susun ulang kartu panel dengan `Shift+Up`/`Shift+Down` pada kartu terfokus, diterapkan segera melalui primitif yang sama dengan penggunaan drag-and-drop panel sebenarnya).

Tindakan ini memperbarui status ARGVUS logis dan menerapkan konfigurasi runtime yang terpengaruh. Lihat [Penampilan](/docs/user-guide/appearance/), [Tema](/docs/user-guide/appearance/) dan [Windows dan tata letak](/docs/argvus-hyprland/windows-and-layout/).

## Penggunaan papan ketikAplikasi pengaturan mendukung navigasi keyboard. Setiap halaman adalah satu daftar: gunakan tombol panah atau `j`/`k` untuk berpindah, `Enter` untuk membuka atau menjalankan baris yang dipilih, `Space` untuk beralih, `Esc` untuk kembali, `/` untuk mencari daftar dan `?` untuk bantuan; footer memperlihatkan kunci yang diterapkan pada baris yang dipilih. Tindakan adalah baris dalam daftar, bukan tombol, dan `Tab` tidak berpindah ke sana, kecuali pada halaman dengan bilah **Tindakan** di bawah daftar (Aplikasi Default, Zona Waktu, Tata Letak Keyboard, Lokal Sistem dan Tombol SSH, dan Buat/Edit Pintasan). Baris yang hanya menampilkan informasi dilewati. Saat bidang teks, pencarian, atau filter daftar aktif, `q` dan `?` diketik sebagai teks alih-alih keluar atau membuka bantuan. Pada **Aplikasi Default**, **Pulihkan Default** adalah tombol di bilah **Tindakan** di bawah daftar: `Tab` atau `←/→` memfokuskannya dan `Enter` meminta pengaturan ulang. **Reset default** di halaman lain dan **Restore all shortcuts** adalah baris dalam **Zona bahaya** di akhir halamannya; `r` meminta pengaturan ulang yang sama, dan pada daftar pintasan `r` memulihkan pintasan yang dipilih setelah konfirmasi. Pada **Bahasa & Wilayah → Tata Letak Keyboard**, `/` memfokuskan kotak **Pencarian** di bagian atas halaman, dan `Esc` menghapusnya; `●` menandai tata letak yang digunakan. Ini mengikuti bilah tugas dan pintasan keyboard yang memutar tata letak, dan halaman diperbarui dalam waktu sekitar satu detik tanpa menulis apa pun ke konfigurasi. `Enter` menetapkan tata letak default, yang pertama dimulai dengan Hyprland dan yang berlabel `Default` dalam detailnya; `Space` menambah atau menghapus tata letak (`[x]`). Kunci ini hanya mengubah draf, dan tombol **Terapkan** di bilah **Tindakan** di bawah daftar menulis tata letak dan default ke Hyprland dalam satu langkah. Dari daftar, `Tab` atau `←/→` memindahkan fokus ke bilah tersebut saat **Terapkan** diaktifkan (footer menampilkan `←/→ Actions` saat itu); `←/→` menelusuri tombolnya dan `↑/↓` kembali ke daftar. Membiarkan halaman dengan perubahan yang belum diterapkan akan meminta konfirmasi terlebih dahulu. Pada **Bahasa & Wilayah → Zona Waktu** tata letaknya sama: `/` memfokuskan kotak **Penelusuran** di bagian atas, `Enter` memilih zona dalam draf (ditandai dengan `●`), dan tombol **Terapkan** di bilah **Tindakan** di bawah daftar menyimpannya di bilah **Tindakan** di bawah daftar pengguna `environment.d/20-timezone.conf`; `Tab`, `←/→` dan `↑/↓` berfungsi seperti pada Tata Letak Keyboard. Berangkat dengan zona yang tidak diterapkan meminta konfirmasi terlebih dahulu. Pada **System Locales** tata letaknya sama dengan Time Zone: `/` memfokuskan kotak **Search** di bagian atas, `Enter` atau `Space` menandai atau menghapus tanda lokal (`[x]`) dalam draf, dan tombol **Apply** pada bilah **Actions** di bawah list menulis pilihan ke `/etc/locale.gen` dan menjalankan `locale-gen`. Karena hal ini membuat ulang lokal sistem, **Terapkan** meminta konfirmasi terlebih dahulu; `Tab`, `←/→` dan `↑/↓` berfungsi seperti dua lainnya. Pada **Desktop & Input → Pintasan Keyboard → Semua Pintasan** kotak pencarian juga berada di bagian atas, dengan `/` memfokuskannya, namun laman tidak memiliki bilah **Tindakan**: daftarnya menyimpan bagian pintasan dan **zona bahaya**, dan footer mencantumkan `r Restore default` hanya pada daftar itu sendiri, bukan pada **Edit pintasan**.Laman yang mengumpulkan perubahan sebelum menyimpannya (pengguna, grup, pembuatan akun, perubahan sandi, dan konfigurasi firewall) mengakhiri blok tersebut dengan baris **Simpan** atau **Terapkan** yang tetap redup hingga ada perubahan. Jika kembali akan membuang perubahan yang belum disimpan, Pusat Kontrol akan bertanya terlebih dahulu. Setiap konfirmasi menampilkan **Konfirmasi** dan **Batal** sebagai dua baris dengan fokus pada **Batal**: `Enter` menjalankan baris fokus, `y` mengonfirmasi dan `n` atau `Esc` membatalkan.

Di **Appearance**, setiap halaman merupakan satu daftar: footer hanya menampilkan kunci yang diterapkan pada baris yang dipilih, dan `Space` juga membuka atau memilih baris tersebut. Halaman yang mengumpulkan perubahan sebelum menerapkannya (Bilah Tugas, Telemetri Widget, Panel Kontrol, dan penggeser efek) diakhiri dengan baris **Terapkan**, yang tetap redup hingga ada perubahan. Memilih **Terapkan** akan menampilkan layar pemuatan hanya ketika perubahan menciptakan kembali permukaan: tinggi bilah tugas, grup utilitas, ikon, atau perubahan format tanggal dan waktu akan memulai ulang bilah tugas; menghidupkan atau mematikan Telemetri Widget atau mengubah blok atau urutannya akan memulai ulang widget; menyalakan atau mematikan Panel Kontrol, atau mengganti kartunya saat sedang aktif, akan menghidupkan ulang panel. Layar kemudian tetap ada hingga perubahan terakhir pada halaman diterapkan, sehingga tidak ada perubahan yang muncul sebelumnya, dan permukaan dimulai ulang satu kali. Perbatasan, transparansi, blur, animasi, celah, font, wallpaper, dan pengaturan lainnya berlaku di tempatnya, tanpa layar pemuatan. Pada baris persentase, `←`/`→` (atau `+`/`-`) mengubah nilai dalam langkah 5 dan `Enter` memungkinkan Anda mengetikkannya; gunakan `Esc` untuk kembali. Jika kembali akan membuang perubahan yang tidak diterapkan, Pusat Kontrol akan bertanya terlebih dahulu. Menghapus tema khusus (`d`) atau mengganti tema yang diimpor akan meminta konfirmasi; fokus dimulai pada **Batal**, `y` mengonfirmasi dan `n` atau `Esc` membatalkan.

## Ketekunan dan setel ulang

Halaman pengaturan menerapkan perubahan melalui penyedianya sendiri dan menyimpan status pengguna yang didukung. Perubahan tampilan disimpan dalam konfigurasi kanonik, dan `argvus-config` adalah satu-satunya komponen yang menulis file konsumen turunan di bawah `~/.config/argvus/data/generated/`; file yang dihasilkan tersebut bukanlah tempat untuk melakukan pengeditan permanen. Halaman masukan dan pengikatan tombol memiliki status tetap dan tindakan pengaturan ulang sendiri. Perubahan sistem lainnya mungkin memerlukan izin atau memuat ulang layanan.

Tidak ada reset global untuk setiap pengaturan ARGVUS. Pulihkan perubahan dari halaman pemiliknya, atau gunakan prosedur pemulihan terdokumentasi dari fitur terkait.

Dukungan reset sengaja dilakukan per domain:

- **Penampilan** memberikan pengaturan ulang untuk warna sorotan ke default tema aktif. Ini tidak menyediakan pengaturan ulang tampilan global tunggal; mengembalikan nilai tampilan lainnya dari kontrol mereka sendiri.
- **Font** dapat memulihkan semua pengaturan font, target, atau pengaturan individual.
- **Aplikasi bawaan** dapat memulihkan semua default, kategori, atau pemilih individual.
- **Pintasan keyboard** dapat memulihkan satu atau semua pengikatan, lalu memuat ulang pengikatan sesi yang dihasilkan.
- **Aturan jendela** ada di **Desktop & Input → Hyprland → Windows → Aturan** (cari `window rules` atau jalankan `argvus control-center window-rules`). Setiap aturan memiliki baris **Ruang Kerja** (`Enter` atau `←/→` langkah dari 1 hingga 10) dan baris **Kelas jendela** yang membuka kolom teks dengan ekspresi reguler yang dipisahkan koma. **Tambahkan aturan** membuat `rule-N` dengan ruang kerja 1 dan tanpa kelas. **Hapus aturan** setiap aturan berada di **zona Bahaya** dan meminta konfirmasi. Setiap perubahan ditulis ke `hyprland.window_rules` di `argvus config` dan segera memuat ulang konfigurasi; pola kelas yang tidak valid ditolak dan ditampilkan sebagai kesalahan.
- **Proyek** ada di **Alat & Diagnostik → Proyek** (cari `projects` atau jalankan `argvus control-center projects`). **Tambahkan folder proyek** dan **Tambahkan folder akar** berada di bagian atas halaman dan membuka bidang teks dengan jalurnya; peluncur memeriksa apakah itu adalah direktori dan menunjukkan kesalahan sebaliknya, termasuk ketika folder tersebut akan mendorong jumlah proyek efektif melewati 9 (`SUPER + CTRL + 1..9` tidak dapat mengatasi lebih banyak). Setiap folder yang dikonfigurasi, ditandai sebagai root (subfoldernya adalah proyek) atau satu proyek, mendapatkan bloknya sendiri di bawah formulir dengan baris **Hapus** yang meminta konfirmasi. Halaman tersebut memanggil `argvus-projects`, sehingga perubahan ditulis oleh peluncur dan diterapkan saat `SUPER + O` atau `SUPER + CTRL + 1..9` dijalankan berikutnya. Jika `argvus-projects` tidak diinstal, halaman tersebut menyatakan demikian.
- **Halaman sistem lain** menampilkan tindakan pengaturan ulang, penerapan, penghapusan, atau pemulihan hanya jika penyedia yang mendasarinya mendukung tindakan tersebut.

Menghapus preferensi pengguna dapat membuat penyedia kembali ke status paket/defaultnya, namun menghapus file secara manual bukanlah prosedur pemulihan umum. Lebih suka tindakan pengaturan ulang halaman atau perintah khusus fitur yang terdokumentasi.

## Titik masuk CLI

Pembantunya adalah `argvus-control-center`, dijalankan melalui `argvus control-center`. Dapat membuka area fokus secara langsung, misalnya:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```

Jalankan `argvus control-center --help` untuk rute yang tersedia di versi terinstal. Rute baris perintah adalah pintasan ke aplikasi pengaturan yang sama, bukan sistem konfigurasi terpisah.

Rute langsung yang berguna termasuk `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (dengan halaman `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` dan `proxy`), `audio`, `bluetooth`, `boot`, `packages` (dengan halaman `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` dan `mirrors`), `storage`, `system-doctor` (`diagnostics` dan `diagnostic` tetap alias), `about` (dengan tab `system`, `about`, `donate`, `credits` dan `copyright`), `power`, `session`, `displays` dan `appearance`. Penampilan juga menerima rute terfokus seperti `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` dan `widget-telemetry`; tampilan menerima `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` dan `hdr`. Gunakan output `--help` yang diinstal saat membuat skrip karena ketersediaan masih bergantung pada build yang diinstal.

## Terkait

- [Konfigurasi pertama](/docs/getting-started/)
- Panel Kontrol
- [Penampilan](/docs/user-guide/appearance/)
- [Pintasan keyboard](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Mouse dan panel sentuh](/docs/argvus-hyprland/input/)

### Dokter Sistem

Buka **Alat & Diagnostik → Dokter Sistem**, cari, atau jalankan `argvus control-center system-doctor`. Pemilih rute dan kategori `diagnostics` dan `diagnostic` sebelumnya tetap tersedia. Tekan `r` untuk menyegarkan, Enter untuk memeriksa cek, `o` secara detail untuk membuka halaman terkait, dan Esc untuk kembali. Koleksi berjalan di latar belakang dan bersifat hanya baca.Pemeriksaan menggunakan jumlah unit yang gagal sistem, alat grafis, dan snapshot Penyimpanan untuk penggunaan sistem file dan kesehatan SMART. Suatu masalah memerlukan bukti; alat yang tidak tersedia atau data SMART yang tidak dapat diakses ditampilkan sebagai tidak tersedia, dan pemeriksaan yang gagal atau habis waktunya merupakan kesalahan. Kategori tanpa bukti yang dikumpulkan tidak berarti pemeriksaan berhasil. Pembaruan paket menggunakan database pacman lokal dan tidak menyinkronkan repositori; file kunci mungkin sudah basi dan tidak membuktikan bahwa operasi sedang berjalan. SMART dapat memerlukan izin dan tidak ada akses yang lebih tinggi yang diminta. Ringkasan mencatat koleksi terakhir yang diselesaikan sebagai waktu Unix.

Penyimpanan dan Dokter Sistem berbagi snapshot dalam memori hingga 10 detik. Menekan `r` secara eksplisit mengumpulkan data baru; permintaan bersamaan menggunakan kembali koleksi yang baru saja selesai. Output subproses Diagnostik dan Penyimpanan dibatasi hingga 1 MiB per aliran, dan output berukuran besar dianggap sebagai pengumpulan gagal.

## Cahaya Malam

Buka **Desktop & Input > Tampilan > Lampu Malam** atau jalankan `argvus control-center displays night-light`. Konfigurasikan suhu, aktivasi manual, dan jadwal lokal tetap melalui `argvus config display night-light`. Lihat [Night Light](/docs/user-guide/night-light/) untuk perintah, status, dan persyaratan waktu proses.

## Cuplikan

Buka **Pusat Kontrol > Administrasi Sistem > Snapshot**, cari
`Snapper`, atau jalankan `argvus control-center snapshot`. Fitur Kargo `snapshot`
disertakan dalam `full` (default), dan tidak bergantung pada `storage`.
Halaman ini ditandai **Eksperimental** karena fitur ini masih dalam pengembangan
dan mungkin tidak lengkap.

Halaman ini hanya mengelola konfigurasi Snapper yang ada dengan `SUBVOLUME`
`/` dan `FSTYPE`-nya adalah `btrfs`. Ia memeriksa mount yang berisi `/` dengan
`findmnt`; konfigurasi bernama `root` yang menunjuk ke tempat lain tidak diterima.
Beberapa konfigurasi yang cocok memerlukan pilihan. Paket Lengkungan `snapper`
bersifat opsional dan harus diinstal secara manual. Snapper harus mendukung `--jsonout` dan
kolom aktif/default eksplisit. Tidak ada konfigurasi yang dibuat secara otomatis.

Pada root non-Btrfs, halaman menampilkan **Sistem file ini bukan Btrfs** dan menawarkan
tidak ada operasi snapshot. Kegagalan deteksi, Snapper hilang, root hilang
konfigurasi, penolakan izin, dan kesalahan backend adalah keadaan yang terpisah.
Jika permintaan pengguna normal ditolak, tekan **a — Otorisasi** untuk meminta Polkit
akses. Tindakan administratif dijalankan di pembantu `system-settings` yang dibatasi,
tidak di TUI yang ditinggikan; otentikasi dapat diminta lagi pada pertanyaan selanjutnya.
Agen Polkit grafis yang ada diperlukan untuk otentikasi.

Kunci: **↑/↓** pilih, **Masukkan** detail/konfigurasi, **c** buat, **d** hapus,
**p** kebijakan penyimpanan, **r** penyegaran, **Esc** kembali, **?** bantuan. Tentang retensi
lihat, **↑/↓** pilih kebijakan, **e** edit, **c** algoritma pembersihan siklus dan
**Masuk** buka konfirmasi. Penciptaan membuka deskripsi opsional
(hingga 256 karakter yang dapat dicetak); deskripsi kosong menggunakan “panduan ARGVUS
cuplikan”. Baik pembuatan maupun penghapusan memerlukan konfirmasi, yang awalnya difokuskan
Batalkan. Penghapusan berlaku untuk satu snapshot yang dipilih dan menolak snapshot 0, yaitu
snapshot yang sedang aktif dan snapshot default. Pembantu memeriksa ulang ini
kondisi sebelum memanggil Snapper. Daftar disegarkan setelah operasi; sebuah terputus
operasi tidak pernah dicoba ulang secara otomatis. Periksa daftar yang disegarkan terlebih dahulu.

Untuk membandingkan snapshot, pilih basis dan tekan **b**, lalu pilih yang lain
tujuan dan tekan **v**. Halaman ini menunjukkan ID yang dipesan, jumlah yang dibuat,
jalur yang dihapus dan dimodifikasi, dan enam indikator status Snapper. Pilih jalur
dan tekan **Enter** untuk meminta perbedaan konten; **Esc** kembali ke jalur yang diubah
dan kemudian daftar snapshot. Indikator mengkodekan pembuatan/penghapusan atau konten/jenisdi posisi 1, izin di 2, pemilik di 3, grup di 4, atribut yang diperluas di
5, dan ACL di 6. Titik berarti tidak ada perubahan pada posisi tersebut. Jalur daftar status;
diff menunjukkan konten tekstual. Output diff biner/non-UTF-8 tidak dirender. Beda
output di atas 256 KiB dilaporkan terpotong dan tidak ditampilkan. Ambigu atau
baris status yang formatnya salah ditolak. Format garis Snapper tidak dapat mewakili jalur
dengan jeda baris yang jelas; nama seperti itu tidak didukung. Izin
kesalahan dapat dicoba ulang melalui **a — Otorisasi**; snapshot yang dihapus atau diubah
konfigurasi root memerlukan penyegaran daftar. Tes backend menggunakan yang palsu
`ProcessRunner` dan tidak memerlukan Snapper, Btrfs atau root.

Dari perbandingan, pilih jalur dan tekan **r** untuk meninjau pemulihan file
dari snapshot tujuan. Pembantu memverifikasi bahwa kedua snapshot masih ada
milik konfigurasi root, jalurnya masih dalam perbandingan, dan
source adalah file biasa dalam tampilan `/.snapshots/<id>/snapshot` standar Snapper.
Perbedaan Snapper yang dibatasi ditampilkan saat tekstual; perbedaan biner atau besar adalah
diidentifikasi tanpa memuat konten lengkapnya ke UI. **Masuk** membuka a
konfirmasi terpisah yang menyebutkan sumber dan tujuan; **Esc** dibatalkan tanpa
mengubah berkas. Konfirmasi akan membuat snapshot keamanan `single` terlebih dahulu
menggunakan Snapper `undochange` hanya untuk jalur itu dan memeriksa apakah cocok dengan
cuplikan yang dipilih. ID snapshot keamanan dilaporkan dan snapshot disimpan.
Pemulihan membagikan kunci yang digunakan oleh peningkatan paket ARGVUS, sehingga tidak dapat tumpang tindih
transaksi pacman Pusat Kontrol resmi.

Versi pertama hanya menerima file biasa yang dimiliki oleh root di bawah induk yang dimiliki oleh root
direktori yang tidak dapat ditulis oleh grup atau orang lain. Itu menolak direktori,
symlink, hard link, file khusus, mountpoint/subvolume, internal Snapper
direktori dan lokasi yang dapat ditulis pengguna. File hilang dari snapshot yang dipilih
tidak pernah dihapus. Root Snapper `undochange` tidak menjamin aplikasi atau
konsistensi basis data dan bukan atomik; snapshot keselamatan adalah titik pemulihan,
bukan jaminan. Snapper hanya mempertahankan metadata yang didukung oleh build-nya. Pisahkan
subvolume dan file di luar root yang dikonfigurasi tetap berada di luar operasi.

Snapshot manual adalah `single`, hanya baca menurut default Snapper, tanpa pembersihan
algoritma yang ditetapkan oleh halaman ini. Pengatur waktu Snapper dan operasi eksternal yang ada
terus melamar. Snapshot bukanlah gambar sistem penuh: subvolume terpisah,
termasuk `/home` terpisah, tidak termasuk. Versi ini memberikan **tidak
rollback sistem penuh, instalasi atau pembuatan konfigurasi Snapper**. Tekan
**p** pada konfigurasi root yang didukung untuk memeriksa pengaturan retensi yang diketahui.
Nilai yang hilang diberi label sebagai fallback ARGVUS, bukan nilai yang dibaca dari instalasi
Templat pembuatan kakap; kunci yang tidak diketahui hanya bersifat diagnostik. Tekan **e** untuk mengedit nilai yang didukung. Konfirmasi menunjukkan yang lama dan
nilai-nilai baru; setelah menyimpan, halaman membaca konfigurasi lagi dan memverifikasinya.
Tekan **c** untuk memutar algoritme pembersihan dan **Enter** untuk meninjau dan mengonfirmasinya.
Pembersihan menggunakan algoritma resmi Snapper; tidak ada pratinjau pasti, dan
jumlah penghapusan yang dilaporkan diukur dengan membandingkan daftar setelah eksekusi. Arch Snapper 0.13.2 menyediakan
`snapper-timeline.timer` dan `snapper-cleanup.timer`; diperlukan pembuatan garis waktu
pengatur waktu garis waktu, dan kebijakan pembersihan yang dikonfigurasi tidak akan berjalan tanpa pembersihan
penjadwalan. Halaman ini hanya membaca status unit dan tidak mengaktifkan pengatur waktu. Batasan ruang memerlukan Btrf yang dikonfigurasi
qgroups dan tidak diekspos di sini.

### Paket operasi dan snapshot

Instalasi, pembaruan, dan penghapusan paket tidak memerlukan Snapper, Btrfs, atau a
konfigurasi cuplikan. **Tingkatkan semua** menjalankan `pacman -Syyu` melalui hak istimewa
pembantu dengan `pkexec`; pembaruan resmi individu dijalankan `pacman -S <package>`.ARGVUS tidak membuat snapshot otomatis untuk operasi ini. Jepretannya
halaman tetap independen dan mempertahankan persyaratan Btrfs/Snappernya sendiri.
Kait yang dikonfigurasi secara terpisah di pacman tetap dikontrol oleh sistem.

### Validasi manual

- Pada root ext4/xfs, buka halaman dalam bahasa Inggris dan Portugis. Dalam bahasa Portugis,
  verifikasi pesan yang sebenarnya **Sistem arsip ini tidak ada Btrfs** dan verifikasi
  c/d itu tidak membuka operasi.
- Dalam VM Btrfs sekali pakai, uji Snapper tidak ada, Snapper tanpa root
  konfigurasi, dan konfigurasi bernama `root` menunjuk ke subvolume lain.
- Dengan konfigurasi root yang ada, daftar snapshot, buka detail, buat a
  snapshot dengan spasi/karakter khusus, batalkan penghapusan, lalu hapus saja
  snapshot yang baru dibuat setelah mengonfirmasi nomor dan deskripsinya.
- Periksa otentikasi yang ditolak/dibatalkan, snapshot aktif/default yang dilindungi,
  menyegarkan, navigasi keyboard, mengubah ukuran dan kedua bahasa. Jangan menguji penghapusan
  terhadap snapshot yang diperlukan untuk pemulihan.

Hanya paket `argvus` yang berubah. Bangun kembali dari `de/argvus` dengan `make build` menjadi
menghasilkan paket `.pkg.tar.zst`; menginstalnya adalah tindakan operator terpisah.

Bahasa di bawah Bahasa & Wilayah menyimpan lokal di profil pengguna (`~/.config/locale.conf`) tanpa mengubah `/etc/locale.conf`. Sesi ini menggunakan preferensi ini setelah login lagi. Selama Selamat Datang, F6 Berikutnya atau tombol footer Berikutnya mencatat tinjauan dan kemajuan; Shift+Tab memfokuskan tombol dan Enter mengaktifkannya, termasuk pada halaman informasi.

Bluetooth muncul di Selamat Datang hanya ketika Pusat Kontrol mendeteksi adaptor. Kemajuan yang ada untuk langkah Bluetooth tersembunyi dipertahankan.

Bahasa dan tata letak keyboard sesi merupakan preferensi pribadi dan tidak memerlukan hak administrator. Profil baru secara default menggunakan tata letak keyboard Inggris dan AS; preferensi pribadi yang ada dipertahankan. Tanggal/waktu, NTP, pembuatan lokal sistem, dan peta kunci konsol (TTY) mempertahankan perilaku administratif seluruh sistemnya.

Untuk keyboard sesi Brasil, pilih `br`, terapkan tata letak, lalu pilih `abnt2` di bawah Varian. Ini menulis konfigurasi Hyprland pribadi tanpa izin administratif; peta kunci konsol terpisah. ABNT2 menggunakan kunci mati: `~` diikuti oleh `a` menghasilkan `ã`, dan `~` diikuti oleh Space menghasilkan `~`. Menerapkan varian akan mengaktifkan tata letak default yang dikonfigurasi; kegagalan aktivasi dilaporkan.

Zona Waktu menyimpan TZ di bawah `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` tanpa izin. Keluar dan masuk kembali agar aplikasi mewarisinya. Tanggal & Waktu terus menampilkan dan mengedit jam sistem di zona waktu sistem, dengan otorisasi administratif; NTP tidak berubah.

Profil tampilan dan penetapan ruang kerja diterapkan melalui konfigurasi Lua yang dihasilkan dan pemuatan ulang Hyprland. Kekuatan monitor menggunakan operator Lua DPMS yang didukung oleh Hyprland 0.56.2.

Kekuatan render bayangan dibatasi hingga 1–4 (default 4) untuk kompatibilitas Hyprland 0.57. Nilai tersimpan lama di atas 4 dijepit saat memuat konfigurasi.

## Jendela goyah

Buka **Desktop & Input → Hyprland → Animasi → Windows** dan gunakan
Kontrol **Goyangan**. Cari `wobble`, atau langsung buka halamannya:

```sh
argvus control-center hyprland animations windows
```

Goyangan dinonaktifkan secara default. Default ARGVUS-nya adalah kekakuan `100`, redaman
`10`, massa `1`, intensitas `1` dan resolusi mesh `8`. Kekakuan dan penerimaan massa
`0.0001–1000`, redaman `0–1000`, intensitas `0–2`, dan mesh bilangan bulat `2–32`.
Resolusi mesh yang lebih tinggi meningkatkan biaya rendering. Gunakan tombol panah untuk menyesuaikan
nilai atau Enter untuk mengetikkannya. **Kembalikan default Wobble** mengubah draf;**Terapkan** menyimpan semua pengeditan animasi yang tertunda dan memuat ulang Hyprland satu kali, tanpa perlu
memulai kembali sesi. Kembali ke dalam Animasi akan menyimpan draf bersama.

Sakelar Animasi global menekan Wobble sambil mempertahankan individualnya
pengaturan. Nilai kanonik berada di bawah `effects.wobble_*` di negara bagian ARGVUS;
`hypr/state.lua` dibuat dan tidak boleh diedit secara manual.

Hal ini memerlukan Hyprland **0.57.0 atau lebih baru**, yang diterapkan oleh `argvus-hyprland`
ketergantungan paket. Versi 0.56 tidak didukung untuk konfigurasi ini. Itu
intensitas maksimum mengikuti [sumber 0.57.0 dirilis](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
yang membatasinya pada `2`, meskipun dokumentasi yang lebih baru mungkin mencantumkan `3`.

Selamat datang dibuka dengan logo ARGVUS dan SELAMAT DATANG. Tekan F6 Berikutnya untuk menampilkan daftar periksa tanpa menyelesaikan satu langkah pun.

Penginstalan baru secara default menggunakan zona waktu pribadi `America/New_York` dan hanya tata letak keyboard `us` yang diaktifkan. Pengaturan pribadi yang ada dipertahankan; zona waktu sistem tidak berubah.

Opsi modul bilah tugas untuk Baterai dan Bluetooth hanya muncul ketika perangkat keras terkait terdeteksi. Menyembunyikan Tetap terjaga menyembunyikan indikatornya tanpa mengubah inhibitor suspensi. Utilitas mendukung laci hover yang diciutkan. Presentasi ikon ruang kerja menampilkan ikon untuk jendela yang terbuka; baki sistem mengontrol ikon aplikasi latar belakang.


Laman pintasan keyboard menyertakan **pemilih agen AI (paket argvus-ai opsional)** dengan SUPER + A default. Laman ini menggunakan ID katalog app.ai, jadi pemetaan ulang dan penonaktifan mengikuti konfigurasi keyboard_shortcuts kanonik yang sama seperti entri lainnya. Ikatan Lua yang dapat dieksekusi disediakan oleh argvus-ai; entri katalog saja tidak mengaktifkannya tanpa paket opsional tersebut.


Pusat Kontrol > Tampilan > Bilah Tugas > Ikon > Modul opsional kini mencakup **Agen AI**. Secara default diaktifkan dan disimpan di /taskbar/icons/ai_agents_enabled. Alihkan di draf bilah tugas dan Terapkan untuk mempertahankan dan membuat ulang grup utilitas; menonaktifkannya tidak mengubah pintasan SUPER + A atau entri peluncur aplikasi.

Sakelar mulai otomatis mengedit entri `.desktop` pengguna atau membuat salinan pengguna dari entri sistem. Nama, perintah, dan tanda pengaktifan berasal dari `[Desktop Entry]`; grup tindakan desktop tambahan tetap tidak berubah.

Konfirmasi layanan mempertahankan cakupan pengguna/sistem yang dipilih bersama dengan unit dan tindakan. Unit dengan nama yang sama di cakupan lain tidak dapat mengalihkan tindakan yang dikonfirmasi; halaman detail juga mempertahankan cakupan aslinya.

Penyimpanan mencakup titik pemasangan bersarang seperti `/home`, `/boot`, dan `/boot/efi` dari inventaris sistem file. Diagnostik menggunakan inventaris yang sama untuk pemeriksaan penggunaan sistem file root dan home yang ada.

Snapshot keamanan pemulihan baru menggunakan algoritma pembersihan `number` Snapper. Penghapusannya mengikuti kebijakan penyimpanan nomor yang dikonfigurasi saat pembersihan dijalankan. Rekam jepret yang ada tanpa algoritme pembersihan tidak akan diubah, dan pembuatan manual akan mempertahankan kebijakan yang ada.

Perbandingan cuplikan melaporkan catatan jalur yang tidak didukung secara terpisah. File yang valid tetap tersedia untuk ditinjau; jalur dengan pengkodean yang tidak valid, karakter kontrol, atau komponen yang tidak didukung tidak menawarkan tindakan diff atau pemulihan.

Pengaturan daya tulis drop-in login ARGVUS di bawah `[Login]`, buat direktorinya bila diperlukan. Kegagalan memuat ulang dilaporkan. Perilaku tutup baterai dipetakan ke `HandleLidSwitch`; konteks `default` yang lama adalah sebuah alias, sedangkan AC dan perilaku yang dipasang ke dok menggunakan kunci loginnya yang terpisah.

Aplikasi Mirrorlist hanya menerima komentar, baris kosong, dan entri HTTP/HTTPS `Server`. Muatan dibatasi hingga 64 KiB dan diperiksa sebelum ketinggian dan lagi oleh pembantu yang memiliki hak istimewa; arahan pacman seperti `SigLevel` dan `XferCommand` ditolak.Pembersihan cache untuk paket yang dihapus instalasinya tidak mempertahankan versi cache. Penolakan operasi repositori termasuk dibagikan oleh beberapa bagian repositori. Nama host dengan spasi di sekelilingnya dan pilihan lokal yang dihasilkan kosong akan ditolak. Penulisan file pengaturan menolak tujuan yang disinkronkan dan file sementara yang ada.

Pencarian paket dan pemuatan halaman membuang hasil dari halaman atau kueri sebelumnya. Halaman saat ini kemudian dimuat kembali, sehingga hasil lama tidak dapat menandai tampilan berbeda sebagai yang baru saja disegarkan.

Output transaksi dibersihkan dan dibatasi. Layar mempertahankan hingga 10.000 baris atau 4 MiB dan laporan menghilangkan keluaran. Aliran proses yang diambil memenuhi batas yang dikonfigurasi, defaultnya adalah 8 MiB untuk eksekusi langsung; kelebihan output dikuras dan dilaporkan setelah proses selesai.

Inventaris boot membaca kernel Arch `pkgbase` untuk menemukan image kernel, file initramfs, dan preset mkinitcpio sambil menampilkan rilis kernel secara terpisah. Array mkinitcpio literal dibaca tanpa menyertakan tanda kurung atau menjalankan konfigurasi.

Perubahan daya untuk penutup, tombol daya, dan tetap terjaga disimpan ke konfigurasi kanonik hanya setelah tindakan runtime berhasil. Membatalkan otorisasi akan membuat nilai yang disimpan sebelumnya tetap utuh. Ketika tidak ada drop-in login ARGVUS yang memberikan nilai, konfigurasi logind utama dibaca sebagai fallback.

Setelah perubahan retensi gagal, halaman akan membaca ulang pengaturan saat ini sebelum menampilkan nilainya lagi. Pemilihan konfigurasi dimulai pada entri pertama.

Operasi paket ARGVUS berbagi kunci eksklusif dengan pemulihan selektif. Penginstalan, penghapusan, penginstalan ulang, penurunan versi, penyegaran metadata, pembersihan cache, dan perubahan repositori atau cermin ditolak saat kunci tersebut ditahan. Koordinasi ini mencakup operasi yang dimulai melalui pembantu ARGVUS; alat pacman dan AUR eksternal tidak menggunakan kunci ini.

Saat operasi latar belakang menerapkan perubahan, `q` dan `Ctrl+C` tetap membuka Pusat Kontrol dan menampilkan pemberitahuan untuk menunggu. Ini juga berlaku setelah keluar dari halaman yang memulai tindakan. Tunggu hasilnya, lalu minta keluar lagi. Penyegaran read-only biasa tidak menghalangi jalan keluar.

Kata sandi Wi-Fi, DNS manual, dan bidang pencarian jaringan menerima tempel dalam tanda kurung. Kontrol yang ditempel dan jeda baris dihilangkan. Pintasan karakter Ctrl-saja atau Alt-saja yang tidak didukung tidak akan menyisipkan huruf. Editor menerima peristiwa AltGr yang dapat dicetak yang dikodekan sebagai Ctrl+Alt; Ctrl+S hanya menyimpan tanpa Alt. Editor teks bersama menjaga navigasi vertikal di kolom visual dan menyelaraskan pengguliran horizontal untuk karakter lebar.

Perubahan volume audio dan bisu untuk mikrofon dan output non-default tidak lagi menimpa preferensi output default yang disimpan. Backend mempertahankan preferensi tersebut hanya jika dapat mengidentifikasi node yang dipilih sebagai output default saat ini. Halaman tersebut menampilkan volume yang diperkuat di atas 100% seperti yang dilaporkan oleh PipeWire; pengeditan volume eksplisit tetap dibatasi hingga 0–100%.

Ikhtisar Perangkat Keras mengidentifikasi model GPU secara terpisah dari alamat PCI dan kelas perangkatnya, dengan menggunakan nama komersial dalam tanda kurung. Ringkasannya menunjukkan penyaji OpenGL yang ditanyakan untuk sesi tersebut dan semua nama perangkat yang dikembalikan oleh probe Vulkan, termasuk perangkat perangkat lunak. Hasil global ini tidak disajikan sebagai kemampuan masing-masing GPU fisik.

Ketika administrator mengunci atau menghapus akun yang dikonfigurasi untuk login otomatis, penyedia akun terlebih dahulu menonaktifkan entri login otomatisnya. Jika pembersihan tersebut gagal, pengoperasian akun ditolak. Jika penguncian atau penghapusan di kemudian hari gagal, login otomatis tetap dinonaktifkan. Login otomatis untuk akun lain dipertahankan; membuka kunci akun tidak mengaktifkan login otomatis. Sesi yang ada tidak ditutup dan salam tidak dimulai ulang.


## Membuat dan mengelola pintasan keyboard**Desktop & Input → Pintasan Keyboard** membuka menu dengan **Buat Pintasan** dan **Semua Pintasan**. Alias ​​CLI yang ada (`keybindings`, `shortcuts`, `keyboard`) buka menu ini; tambahkan `create` atau `list` untuk membuka anak secara langsung. Pencarian global juga menemukan kedua anak tersebut.

**Buat Pintasan** meminta deskripsi, kombinasi keyboard, dan jenis tindakan. Pilih **Pintasan saat ini** dan tekan `Enter`, lalu tekan kombinasi tombol yang diinginkan. **Ubah pintasan** memulai pengambilan yang sama; tidak ada yang membuka editor teks. Menangkap perubahan hanya drafnya. Jika kombinasi sudah digunakan, konflik akan segera ditampilkan, bahkan sebelum deskripsi dan perintah diisi. Mengonfirmasi **Ganti** memilih kombinasi dan mencatat pilihan Anda; itu tidak membuat pintasan atau menonaktifkan pintasan lain hingga **Buat**/**Simpan**. Membatalkan konflik akan mempertahankan kombinasi sebelumnya. **Perintah** berjalan saat pintasan ditekan; ciptaan tidak pernah menjalankannya. **Petugas operator Hyprland** menggunakan nama operator Lua yang didukung, seperti `window.float`, dengan argumen opsional sebagai objek JSON, misalnya `{"action":"toggle"}`. Subpeta, pengikatan mouse, dan tanda lanjutan tetap berada di luar formulir ini.

Gunakan `Tab` untuk beralih antara formulir dan **Tindakan**, `←/→` untuk berpindah antara **Buat** dan **Batal**, dan `Enter` untuk menjalankan tindakan yang dipilih. Batalkan atau `Esc` membuang draf tersebut. Setelah dibuat, **Semua Pintasan** terbuka dengan item baru dipilih dan terlihat. Entri khusus muncul di **Pintasan Khusus**; pencarian cocok dengan deskripsi, kunci, dan tindakan. `Enter` mengedit deskripsi, tindakan, dan tombol, `Space` mengaktifkan/menonaktifkan, dan memilih baris **Hapus** dan menekan `Enter` akan membuka konfirmasi yang berfokus pada Batal. Mengonfirmasi akan menghapus catatan dan kembali ke daftar yang disegarkan; membatalkan menyimpannya.

Pintasan yang diaktifkan dalam konteks yang sama tidak dapat menggunakan kembali kombinasi secara diam-diam: konflik yang dapat dikonfigurasi memerlukan konfirmasi untuk menggantinya, dan pintasan yang dilindungi tidak dapat diganti. Penggantian menyimpan pintasan baru dan menonaktifkan konflik dalam satu transaksi konfigurasi. **Pulihkan default** hanya berlaku untuk entri resmi; **Pulihkan semua pintasan** mempertahankan catatan khusus. Jika pemulihan akan bertabrakan dengan pintasan khusus yang aktif, ubah atau nonaktifkan pintasan khusus tersebut terlebih dahulu.

Simpan dan terapkan berjalan di latar belakang. Jika penyimpanan gagal, draf tetap tersedia. Jika aplikasi ke sesi gagal setelah disimpan, halaman mengatakan sesi telah disimpan dan mempertahankan ID yang sama untuk dicoba lagi. Catatan khusus ada di `/hyprland/custom_shortcuts` dalam konfigurasi kanonik, diproyeksikan ke `data/generated/hypr/custom-shortcuts.lua`; jangan mengedit file yang dihasilkan.

Kesalahan pintasan tetap berada di dalam TUI: ukuran popup sendiri sesuai dengan pesan yang dibungkus dan membuat **Enter OK** tetap terlihat. Diagnostik pintasan teknis masuk ke jurnal lokal (`journalctl -t argvus-control-center`) alih-alih stdout/stderr. Penghapusan yang disimpan tetapi tidak dapat diterapkan melaporkan perbedaan tersebut; muat ulang konfigurasi sesi untuk menghapus pengikatan aktif sebelumnya.

**Status Pembaruan** di **Bilah Tugas → Ikon → Modul opsional** menampilkan jumlah pembaruan yang tersedia dan diaktifkan secara default. **Periksa AUR** dinonaktifkan secara default dan memerlukan `paru` atau `yay`; terapkan draf bilah tugas untuk menyimpan pengaturan mana pun. Pemeriksaan dijalankan saat startup dan setiap jam; mengklik indikator akan membuka halaman resmi pembaruan Paket.

## Mode jendela global

Buka **Pusat Kontrol → Desktop & Input → Hyprland → Windows → Mode** dan pilih **Tile Windows (Default)** atau **Floating Windows**. Pemilih yang sama tersedia dari ikon mode jendela di grup utilitas bilah tugas.Preferensi berlaku untuk setiap monitor dan ruang kerja dan diingat setelah logout. Mengambang mengubah jendela aplikasi yang ada dan jendela yang baru dibuka. Tile hanya memulihkan jendela yang diubah kebijakannya; dialog dan jendela lain yang sudah mengambang tetap mengambang. Sebuah jendela yang secara eksplisit disematkan ketika Floating aktif juga tetap mengambang untuk mempertahankan pinnya. Tata letak ubin dihitung ulang; pohon terbelah sebelumnya tidak dipulihkan.

`SUPER + SHIFT + Space` mempertahankan perilaku normal per jendela di Tile. Dalam Mengambang, jendela tidak dikembalikan ke ubin. Panel, bilah tugas, dan notifikasi merupakan permukaan lapisan cangkang dan tidak terpengaruh. Layar penuh dan maksimalisasi terus menggunakan perilaku normal Hyprland.

Preferensi kanoniknya adalah `/hyprland/window_mode` di `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Proyeksi yang dihasilkannya adalah `data/generated/hypr/window-mode.lua`; jangan mengeditnya. Registri pemulihan per jendela berada di bawah `$XDG_RUNTIME_DIR/argvus`, tercakup dalam instance kompositor, dan bertahan saat memuat ulang konfigurasi. Itu tidak digunakan kembali di sesi baru.

Jika aplikasi tertunda atau gagal, antarmuka akan membedakannya dari preferensi yang disimpan. Pilih opsi lagi untuk mencoba lagi. Tanpa kompositor, preferensi disimpan untuk login berikutnya.

## Tombol jendela dan bilah aplikasi

Mode mengambang menambahkan bilah judul ARGVUS dengan **Minimalkan**, **Maksimalkan / Pulihkan**, dan **Tutup** ke jendela aplikasi normal. Mengklik dua kali bilah judul akan mengaktifkan maksimalisasi. Dekorasi tambahan disembunyikan di Tile dan layar penuh. Aplikasi yang menggambar kontrolnya sendiri dapat menampilkan kedua bilah; kontrol ARGVUS yang seragam memang disengaja.

Waybar kedua muncul di setiap monitor, di tepi seberang taskbar utama. Ini hanya mencantumkan jendela ruang kerja monitor saat ini, termasuk jendela yang diperkecil. Tab grup tersembunyi tidak membuat entri tambahan. Entri yang diperkecil mempertahankan asosiasi monitor/ruang kerja aslinya. Klik jendela yang tidak aktif untuk mengaktifkannya, jendela aktif untuk memperkecilnya, atau jendela yang diperkecil untuk memulihkannya; menu konteksnya menawarkan pemulihan, perkecil, maksimalkan/pulihkan, dan tutup. Ikon aplikasi yang tidak dikenal kembali ke simbol jendela.

`SUPER + Alt + Up` menempatkan bilah utama di bagian atas dan bilah jendela di bagian bawah. `SUPER + Alt + Down` melakukan sebaliknya. Mengulangi suatu arah membuat posisinya tidak berubah. Ubin hanya memiliki bilah utama. Kembali ke Ubin akan memulihkan semua jendela yang diperkecil sebelum menghapus bilah jendela, dan kemudian mempertahankan kebijakan ubin/mengambang yang asli.

Minimalkan menggunakan ruang kerja `special:argvus-minimized` yang dicadangkan. Jangan gunakan ruang kerja itu secara manual. Proses `argvus windows service` mencatat identitas, ruang kerja/monitor asli, geometri, layar penuh/maksimalisasi, dan status pin dalam registri runtime khusus instance kompositor di bawah `$XDG_RUNTIME_DIR/argvus`. Catatan bertahan saat layanan dimulai ulang atau memuat ulang konfigurasi, tetapi tidak melintasi sesi kompositor. Menutup jendela akan menghapus catatannya. Jika monitor hilang, pemulihan menggunakan output yang tersedia. Bilah pemulihan yang hilang atau gagal mencegah minimalisasi baru; restorasi yang gagal mempertahankan catatan dan bilah pemulihan.

Sesi ini memiliki `argvus-windows.service` dan `argvus-window-bar.service`. Yang terakhir membuat anak Waybarnya tidak aktif di Tile. Plugin hyprbars dibundel oleh `argvus-hyprland`, dibuat untuk Hyprland 0.57.0 ABI pada x86_64; itu tidak dikompilasi di direktori home pengguna. Bangun kembali plugin dengan pembaruan ABI kompositor.

Perintah: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all`, dan `minimize|restore|activate|maximize|close ADDRESS`. Alamat jendela berasal dari daftar. `position top --persist` menyimpan/memproyeksikan preferensi tanpa memulai ulang bilah utama. Detail kesalahan konfigurasi dan runtime ditulis ke jurnal; kegagalan tindakan yang terlihat juga menampilkan pemberitahuan yang diterjemahkan.

## Submenu Windows dan menutup aplikasi**Desktop & Input → Hyprland → Windows** mengelompokkan **Aturan**, **Spasi** dan **Mode**.
Pengeditan aturan penempatan per aplikasi; Spasi mengedit celah dalam dan luar; Modus
berisi pemilih Ubin/Mengambang langsung, terpisah dari formulir aturan.
Pencarian membuka halaman yang sama, dan Kembali kembali ke Windows sebelum Hyprland.

Rute langsung adalah `argvus control-center hyprland windows`, diikuti secara opsional
oleh `rules`, `spaces` atau `mode`. `window-rules` dan
Rute `appearance window-spaces` tetap alias dan menampilkan remah roti baru.

`SUPER + Q` segera menutup jendela Pusat Kontrol khusus, tanpa
Konfirmasi jendela OS Kitty. Draf yang belum diterapkan akan dibuang saat jendela dibuka
ditutup dengan cara ini. Penggantian ini mencakup jendela `argvus-control-center`
kelas; terminal reguler dan TUI lainnya mempertahankan kebijakan konfirmasi tertutupnya.

`SUPER + Alt + Space` mengaktifkan mode Ubin/Mengambang global, menggunakan preferensi yang sama seperti bilah tugas dan Pusat Kontrol. Perintah yang setara adalah `argvus config window-mode toggle`. Pemanggilan serentak dibuat berseri; beralih menggunakan preferensi yang disimpan bahkan ketika aplikasi tertunda. Pintasan dapat diubah atau dinonaktifkan di Pintasan Keyboard; `SUPER + Shift + Space` mempertahankan tindakan jendela individualnya.

## Memasang tema dari Git

Pasang repositori skeleton melalui HTTPS atau SSH memakai kredensial Git yang tersedia. Pemasangan per pengguna tidak mengaktifkan tema. `--ref` memilih cabang atau tag; `--replace` mengganti tema pribadi. ID resmi terpasang dilindungi.

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

Tema muncul di **Custom themes**; tekan `r` untuk memperbarui. Terapkan tema lain sebelum menghapus tema aktif melalui CLI atau Control Center. Terapkan kembali setelah menggantinya.

Harus ada satu tema schema 2 di `src/usr/share/argvus/themes/<id>/` dengan ID sesuai. Simpan gambar dalam tema atau `src/usr/share/`. Skrip tidak dijalankan. Submodul dan Git LFS tidak didukung. Diperlukan `git` dan `openssh` untuk SSH. Tujuan: `$XDG_DATA_HOME/argvus/themes/<id>`. Paket resmi tetap dikelola pacman.
