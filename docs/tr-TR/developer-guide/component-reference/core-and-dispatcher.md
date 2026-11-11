---
title: Çekirdek ve dağıtıcı
description: Argvus çalışma alanı, dağıtıcısı ve paylaşılan Rust kasaları.
---

`argvus`, `de/argvus` adresindeki Kargo çalışma alanıdır ve `/usr/bin/argvus` sağlayan pakettir. `argvus-cli` sandığı dağıtıcıdır: ilk argümanı okur ve işlem sırasında bir kitaplık komutunu çalıştırır, bir GTK 4 yardımcısını veya özel bir yürütülebilir dosyayı çalıştırır veya paketlenmiş bir komut dosyasını çalıştırır. Tam alt komut listesi [komut satırı referansında](/docs/reference/command-line/) bulunur.

Çalışma alanı aynı zamanda komutların paylaştığı kasaları da barındırır:

- `argvus-config`: kanonik yapılandırma deposu ve oluşturulan ağacın tek yazıcısı (`data/generated/`). Aynı zamanda `argvus config`, `theme-switch` ve `effects-toggle` tarafından süreç içinde kullanılan bir kitaplıktır.
- `argvus-i18n` ve `argvus-i18n-core`: `resources/locales/`'daki kataloglar, bunların doğrulanması ve `org.argvus.i18n` QML modülü.
- `argvus-tui` ve `argvus-theme`: paylaşılan terminal kullanıcı arayüzü ve tema yükleme.
- `argvus-runtime`: çevrilmiş mesajlar ve komutlar tarafından paylaşılan işlem yardımcıları.

`argvus-session` ayrı bir depo olarak kalır. Oturum giriş noktalarını (`session`, `start`, `tty`) ve yaşam döngüsü birimlerini sağlar ve `argvus sessionctl` denetleyicisine ulaşır.