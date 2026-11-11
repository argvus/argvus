---
title: Gereksinimleri
description: ARGVUS için sistem gereksinimleri ve ön koşullar.
---

ARGVUS, Arch Linux'u hedefler ve Wayland özellikli bir grafik yığını, Hyprland ve paketlerin çektiği çalışma zamanı bağımlılıklarını gerektirir. Bir ekran yöneticisi isteğe bağlıdır: ARGVUS, greetd aracılığıyla grafiksel bir oturum açma akışını ve manuel bir TTY akışını destekler.

Kurulumdan önce şunlardan emin olun:

- sistem `pacman` ile tamamen güncellendi;
- çalışan bir çekirdek ve grafik sürücüsü yüklü;
- kullanıcı bir Wayland oturumu çalıştırabilir;
- paket deposu yapılandırıldığında ağ kullanılabilir.

Sanal makinelerin, [Sanal makineler](/docs/user-guide/virtual-machines/) bölümünde açıklanan uyumluluk ayarlarına ihtiyacı olabilir.