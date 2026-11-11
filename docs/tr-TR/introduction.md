---
title: Giriş
description: ARGVUS nedir, masaüstünün nasıl çalıştığı ve bileşenlerinin nereye yerleştirildiği.
---

ARGVUS, Arch Linux için modüler bir Hyprland ve Wayland masaüstü ortamıdır. Tek bir monolitik uygulama yerine entegre bir paket koleksiyonudur: oturum, derleyici, kabuk yüzeyleri, ayar uygulamaları, sağlayıcılar ve görsel varlıklar, paylaşılan durum ve açık çalışma zamanı sözleşmeleri aracılığıyla birlikte çalışır.

Bu sayfa belgelerin haritasıdır. Tüm masaüstünü yüksek düzeyde açıklar; bağlantılı sayfalar operasyonel ve uygulama ayrıntılarını içerir.

## ARGVUS nasıl çalışır?

Normal oturum şu yolu izler:

```text
graphical login or TTY
        ↓
argvus-session → argvus start
        ↓
Hyprland + argvus-hyprland
        ↓
argvus-session.target (systemd --user)
        ↓
taskbar · control panel · notifications · wallpaper · idle · clipboard
```

`argvus-session`, oturum yaşam döngüsüne ve ortam kurulumuna sahiptir. Her masaüstü özelliğini uygulamaz: her bileşen kendi komutlarının, yapılandırmasının ve hizmet yükünün mülkiyetini korur.

Kurulum ve başlatma için bkz. [Kurulum](/docs/getting-started/installation/) ve [Oturumlar](/docs/user-guide/sessions/). Hizmet sahipliği için geliştiriciye [çalışma zamanı yaşam döngüsü](/docs/developer-guide/architecture/runtime-lifecycle/) bakın.

## Ana katmanlar

### Giriş noktaları ve altyapı

`argvus` paketi, `/usr/bin/argvus` dağıtıcısını ve ana paket giriş noktasını sağlar. `argvus-session`, grafiksel ve TTY giriş noktaları, kullanıcı hedefi ve `argvus sessionctl` sağlar. Paylaşılan TUI ve yerelleştirme işlevi, `argvus` çalışma alanından (`argvus-tui` ve `argvus-i18n` kasaları) ve `argvus i18n` komutundan gelir.

### Besteci ve masaüstü kabuğu

`argvus-hyprland`, Hyprland yapılandırmasını ve tuş bağlamalarını sağlar. `argvus-waybar` yamalı Waybar ikili dosyasını sağlarken `argvus-taskbar` görev çubuğu yapılandırmasına ve eylemlerine sahiptir. `argvus-control-panel`, Quickshell panelini sağlar ve `argvus widget-telemetry`, `argvus`'da işlem sırasında isteğe bağlı sistem widget'larını sağlar.

Kullanım için [Masaüstü](/docs/user-guide/desktop/) ve uygulama sahipliği için [kabuk yüzeyleri](/docs/developer-guide/subsystems/shell-surfaces/) konusuna bakın.

### Ayarlar ve uygulamalar

`argvus control-center`, `argvus`'da yardımcı olarak gönderilen, klavye öncelikli ayarlar uygulamasıdır. Takvim ve çıkarılabilir cihaz araçları, `argvus`'daki GTK 4 yardımcılarıdır; sistem monitörü ve terminal `argvus`'nun alt komutlarıdır; başlatıcı `argvus-launcher`'dur. Hepsi kabuk ve dağıtıcı ile entegredir.

Bkz. [Uygulamalar](/docs/user-guide/applications/) ve [komut referansı](/docs/reference/command-line/).

### Sağlayıcılar ve entegrasyonlar

Odaklanan projeler tarafından ekran, ağ, Bluetooth, güç, bildirimler, güvenlik duvarı ve çıkarılabilir depolama sağlanır. Kontrol Merkezi ve Kontrol Paneli bu sağlayıcıları kullanır; onların yerine geçmezler.

Oturum açma, kilitleme ve başlatma görselleri de ayrıdır: `argvus-greeter` greetd'yi yönetir, `argvus-lock` Hyprlock'u yönetir, `argvus`'da gönderilen geçiş yardımcısı oturum açma döndürücüyü çizer, yükleme katmanı tema geçişlerini yönetir ve `argvus-boot-splash` önyükleme sıçramasını yönetir.

Bkz. [Donanım](/docs/user-guide/hardware/), [Gizlilik ve güvenlik](/docs/user-guide/privacy-and-security/) ve geliştirici [bileşen referansı](/docs/developer-guide/).

## Paylaşılan görünüm durumu

Tema, vurgu, duvar kağıdı, yazı tipi ve efekt tercihleri `$XDG_CONFIG_HOME/argvus` (normalde `~/.config/argvus`) altında mantıksal durum olarak saklanır. `argvus-config`, Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, bildirimler, kilit ekranı, Yazi, Superfile ve terminaller için tüketici dosyalarını yansıttığı `data/generated/` yazan tek bileşendir. Rofi bir istisnadır: `argvus launcher`, yapılandırmasını ve temalarını düz dosyalar olarak paketler ve kullanıcının düzenlenen kopyaları `data/rofi/` altında yayınlanır. Görünüm yardımcıları durum değişikliğini gerçekleştirir ve ardından harici uygulamaları uzlaştırır; oluşturulan ağacı asla yazmazlar.Oluşturulan dizin gerçeğin kaynağı değildir. Bkz. [Görünüm](/docs/user-guide/appearance/) ve [yapılandırma ve durum](/docs/developer-guide/architecture/configuration-and-state/).

Görsel sistem aynı zamanda düzeni de kontrol eder. Yapışkan mod kompakt ve karedir; Aralıklı mod, daha büyük pencere boşlukları, yuvarlatılmış köşeler ve görev çubuğu ile kabuk çevresinde daha geniş kenar boşlukları kullanır. Görev çubuğu varsayılan olarak üst kenardadır ancak paylaşılan düzen durumu aracılığıyla üst/alt yerleşimi destekler. Efektler, besteci ve masaüstü yüzeylerindeki şeffaflığı, bulanıklığı, gölgeleri ve animasyonları kontrol eder.

Mevcut aileler ve renkler için [tema tablosuna](/docs/user-guide/appearance/), paylaşılan efektler sözleşmesi için [efektlere](/docs/argvus-appearance/effects/) ve çubuk ve pencere yerleşimi için [görev çubuğuna](/docs/argvus-taskbar/taskbar/) bakın.

## ARGVUS nedir ve ne değildir

ARGVUS, Arch Linux'un üstündeki masaüstü ortamı katmanıdır. Arch Linux işletim sistemi ve paketleri sağlar; Hyprland, Wayland kompozisyonunu sağlar; ARGVUS grafik oturumunu, kabuk yüzeylerini, ayarları, görünümü, masaüstü hizmetlerini ve entegre uygulamaları koordine eder.

Bu nedenle bir Hyprland konfigürasyonundan veya kişisel nokta dosyaları koleksiyonundan daha fazlasıdır. ARGVUS, desteklenen bir oturum giriş noktasını tanımlar, masaüstü hizmetlerinin yaşam döngüsüne sahip olur, ortak ayarlar ve görünüm modeli sağlar ve ağ, ekranlar, güç, bildirimler, hesaplar ve çıkarılabilir depolama gibi odaklanmış bileşenleri birbirine bağlar.

ARGVUS'un her uygulamayı değiştirmesi gerekmez. Masaüstü genelinde davranışı ve yapılandırmayı tutarlı tutarken, en güvenilir seçim olan seçilen yukarı akış uygulamalarını entegre eder.

İsim iki fikri birleştiriyor. *Argüman vektöründen* gelen *ARGV*, bir programın aldığı argümanların listesidir ve komutlar ve kısayollar aracılığıyla masaüstünü yönlendirmeye yönelik klavye öncelikli hedefini yansıtır. *Kullanıcı Sistemi*'nden *ABD*, kullanıcısını merkeze alan bir masaüstü hedefini yansıtır. Tam açıklama ve telaffuz sesi için bkz. [ARGVUS Hakkında](/about/).

Uygulama sınırları ve paket sahipliği için geliştiriciye [mimariye genel bakış](/docs/developer-guide/architecture/overview/) ve [depo rolleri](/docs/developer-guide/architecture/repository-roles/) bakın.

## Hedefe göre devam et

- [SSS'yi okuyun](/docs/user-guide/faq/).
- [ARGVUS'u yükleyin](/docs/getting-started/installation/).
- [Bir grafik veya TTY oturumu başlatın](/docs/user-guide/sessions/).
- [Temaları ve duvar kağıtlarını yapılandırın](/docs/user-guide/appearance/).
- [Görev çubuğunu ve Denetim Masasını kullanın](/docs/user-guide/desktop/).
- [Komutları, yolları ve hizmetleri anlayın](/docs/reference/).
- [Mimariyi inceleyin](/docs/developer-guide/architecture/overview/).