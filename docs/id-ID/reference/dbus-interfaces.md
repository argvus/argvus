---
title: D-Bus interfaces
description: D-Bus contracts used by ARGVUS integrations.
---

ARGVUS menggunakan API D-Bus sistem daripada mengekspos satu bus di seluruh desktop. Konsumen penting termasuk BlueZ dan logind di Pusat Kontrol dan penyapa, UDisks2 di perangkat yang dapat dilepas, dan ratbagd di pengaturan perangkat keras.

Antarmuka persisnya milik penyedia atau layanan hulu. Periksa repositori pemilik dan data introspeksi yang diinstal sebelum mengandalkan metode atau properti.