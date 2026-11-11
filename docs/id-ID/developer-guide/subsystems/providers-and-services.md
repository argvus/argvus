---
title: Penyedia dan layanan
description: Provider packages and their system interfaces.
---

Proyek penyedia memaparkan antarmuka terfokus kepada konsumen UI:

- `argvus network` dan `argvus bluetooth` mengintegrasikan NetworkManager dan pembantu Bluetooth;
- `argvus-display` mengintegrasikan status monitor dengan Hyprland;
- `argvus-power` mengintegrasikan tindakan idle, lock, dan power;
- `argvus-firewall` mengelola `argvus-firewall.service`;
- pembantu `argvus removable-devices` di `argvus` menggunakan UDisks2;
- pembantu `argvus control-center` di `argvus` menggunakan API penyedia dan antarmuka D-Bus sistem.

Pusat Kontrol adalah klien dari penyedia ini, bukan penggantinya.