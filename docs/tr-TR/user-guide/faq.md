---
title: Sıkça sorulan sorular
description: ARGVUS, mimarisi ve günlük kullanımı hakkında sık sorulan sorular.
---

Bu sayfada ARGVUS'un ne olduğu, nasıl dağıtıldığı ve nerede yapılandırılacağı ile ilgili sık sorulan soruların yanıtları verilmektedir. Ayrıntılı prosedürler için her yanıttaki bağlantıları izleyin.

## ARGVUS nedir?

ARGVUS, Hyprland etrafında oluşturulmuş, Wayland ve Arch Linux için modüler bir masaüstü ortamıdır. Hyprland'den bağımsız bir masaüstü ortamı değildir; Hyprland üzerine monte edilmiş ve Arch Linux'a uyarlanmış bir masaüstü deneyimidir. ARGVUS, bir paket koleksiyonu aracılığıyla grafik oturumu, kabuk yüzeylerini, ayar uygulamalarını, görünümü, masaüstü hizmetlerini ve entegre araçları koordine eder.

Bir Hyprland yapılandırmasından veya kişisel nokta dosyaları koleksiyonundan daha fazlasıdır: desteklenen bir oturum giriş noktası, paylaşılan yapılandırma ve görünüm durumu ve görev çubuğu, Kontrol Merkezi, bildirimler, ağ oluşturma ve ekranlar gibi bileşenler için tanımlanmış sahiplik sağlar. Parçaların birbirine nasıl uyduğuna dair genel bir bakış için [giriş](/docs/introduction/) bölümüne bakın.

## ARGVUS ücretsiz mi?

Evet. ARGVUS'u indirmek ve kullanmak ücretsizdir ve temel proje paketleri, [GNU Genel Kamu Lisansı sürüm 3](https://www.gnu.org/licenses/gpl-3.0.html) kapsamında ücretsiz yazılım olarak dağıtılmaktadır. Bu, kullanıcılara kapsanan yazılımı lisans koşulları kapsamında çalıştırma, inceleme, değiştirme ve yeniden dağıtma özgürlüğü verir.

ARGVUS imzalı Arch Linux paketleri olarak dağıtılır. Yazılımın kendisinin bir satın alma fiyatı yoktur, ancak paketleri indirmek yine de normal internet veya barındırma maliyetlerini gerektirebilir. Üçüncü taraflarca sağlanan bileşen ve uygulamaların kendi lisansları olabilir.

## ARGVUS bir Linux dağıtımı mıdır?

Hayır. ARGVUS, Arch Linux'un üstüne kurulan masaüstü ortamı katmanıdır. Arch Linux işletim sistemi, çekirdek, paket yöneticisi ve temel paketleri sağlar; Hyprland, Wayland kompozisyonunu sağlar; ARGVUS, masaüstü oturumunu ve entegre bileşenlerini koordine eder.

Desteklenen kurulum modeli için [kurulum](/docs/getting-started/installation/) konusuna ve mimari ayrım için [ARGVUS nedir ve ne değildir](/docs/introduction/#what-argvus-isand-is-not) konusuna bakın.

## Neden dağıtım yerine masaüstü ortamı?

ARGVUS'u bir masaüstü ortamı olarak tutmak, kullanıcılara sistemlerinin geri kalanı üzerinde daha fazla kontrol sağlar. Bir dağıtım normalde çekirdek, paket seçimi, depolar, varsayılanlar ve sistem ilkeleri dahil olmak üzere eksiksiz bir temel çizgiyi seçer ve korur. ARGVUS, masaüstü deneyimine odaklanırken bu daha geniş kararları kullanıcıya ve temeldeki Arch Linux kurulumuna bırakır.

Bu model aynı zamanda projeyi modüler tutar: oturum yönetimi, Hyprland yapılandırması, görünüm, görev çubuğu, ayarlar ve sağlayıcılar tek bir masaüstü bilgisayar olarak birlikte çalışırken ayrı ayrı geliştirilip paketlenebilir.

## Hangi sistemler destekleniyor?

Şu anda desteklenen temel Arch Linux'tur. ARGVUS, Arch Linux için paketlenmiştir ve [kurulum gereksinimlerinde](/docs/getting-started/installation/requirements/) açıklanan Wayland, Hyprland ve sistem bileşenlerine bağlıdır.

Diğer dağıtımlar da aynı yukarı akış bileşenlerinden bazılarını sağlayabilir ancak bu, ARGVUS paket setinin veya oturum entegrasyonunun tamamının orada desteklendiği anlamına gelmez.

## ARGVUS'u nasıl kurarım?

[Kurulum kılavuzunu](/docs/getting-started/installation/) izleyin. Gereksinimleri, imzalı ARGVUS paket deposunun nasıl yapılandırılacağını, paket setinin nasıl kurulacağını ve ilk oturumun nasıl başlatılacağını açıklar.

ARGVUS, koordineli bir Arch paketleri seti olarak teslim edilir. `argvus` paketi ana kurulum hedefidir ve koordine ettiği masaüstü bileşenlerini çeker.

## ARGVUS ayarlarını nasıl değiştiririm?Temalar, duvar kağıtları, efektler, düzen, yazı tipleri, klavye, giriş, dil, bölge ve varsayılan uygulamalar dahil olmak üzere kalıcı masaüstü yapılandırması için **Kontrol Merkezi**'ni kullanın. Bunu açmanın en etkili yolu `SUPER + Alt + C` kısayolu veya `SUPER + D` içeren başlatıcıdır: **ARGVUS Kontrol Merkezi** yazın. ARGVUS uygulamalarını listelemek için `argvus` da yazabilirsiniz.

Ek bir seçenek olarak, bunu bir terminalden aşağıdakilerle açabilirsiniz:

```sh
argvus --control-center
```

`argvus control-center` ile odaklanmış bir alanı da açabilirsiniz, örneğin `argvus control-center appearance themes`. Yüklü sürümde mevcut olan rotaları görmek için `argvus control-center --help` komutunu çalıştırın.

Geçerli durum ve ses seviyesi, parlaklık, ağ, bildirimler, güç ve oturum kontrolleri gibi sık yapılan işlemler için **Denetim Masası**'nı kullanın. İkinci bir konfigürasyon veritabanı değildir. Ayrım için bkz. [Nerede yapılandırılmalı](/docs/user-guide/where-to-configure/) ve [Kontrol Merkezi](/docs/components/argvus-control-center/).

## ARGVUS ayarları nerede saklanıyor?

Kullanıcıya ait yapılandırma `$XDG_CONFIG_HOME/argvus` (normalde `~/.config/argvus`) altında saklanır. Kanonik mantıksal yapılandırma burada korunurken Hyprland, Waybar ve telemetri widget'ı, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, terminaller, yazı tipleri, efektler, takvimler, giriş, klavye, güç ve çıkarılabilir cihazlar için türetilmiş tüketici dosyaları `~/.config/argvus/data/generated/` altında bulunur. Bu ağaç yalnızca `argvus-config` tarafından yazılmıştır ve kanonik yapılandırmadan yeniden oluşturulmuştur; `argvus-appearance` görünüm değişikliklerini gerçekleştirir ve ardından yalnızca kanonik yapılandırmayı kendi başına okuyamayan harici tüketicileri uzlaştırır. Rofi bir istisnadır: `argvus launcher`, yapılandırmasını ve temalarını paketler ve kullanıcının düzenlenen kopyaları `~/.config/argvus/data/rofi/` altında yayınlanır.

Oluşturulan dosyaları kalıcı bir yapılandırma yöntemi olarak düzenlemeyin. Özelleştirmek istediğiniz bileşen için Denetim Merkezi'ni veya belgelenen yerel geçersiz kılma yolunu kullanın. Bkz. [yapılandırma dosyaları](/docs/reference/configuration-files/) ve [yapılandırma ve durum](/docs/developer-guide/architecture/configuration-and-state/).

## Değişiklik uygulanmazsa ne yapmalıyım?

Öncelikle değişikliğin sahiplik sayfasında kaydedildiğini veya uygulandığını onaylayın. Ardından, etkilenen bileşenin oturuma veya hizmetin yeniden yüklenmesine ihtiyaç duyup duymadığını kontrol edin. Taahhütlü bir değişiklik `argvus-config` tarafından tüketici dosyalarına yansıtılır, ancak bunların yansıtılması çalışan her tüketicinin bunları yeniden yüklediğini garanti etmez. `argvus sessionctl reload`, tekrar yansıtmak yerine projeksiyon bildiriminden zaten taahhüt edilmiş bir oluşturmayı uygular ve kurallı durum değişmediğinde grafiksel yeniden yüklemeyi atlar.

Kurulum, oturum başlatma, grafikler, ekranlar, Hyprland, temalar ve duvar kağıtları için ilgili [sorun giderme kılavuzunu](/docs/user-guide/troubleshooting/) kullanın. Bir sorunu bildirirken bileşeni, değiştirilen ayarı, kullanılan komutu veya sayfayı ve ilgili durum veya teşhis komutunun çıktısını ekleyin.

## Daha fazlasını nereden öğrenebilirim?

- Önerilen ilk oturum akışı için [İlk yapılandırma](/docs/user-guide/getting-started/).
- Özelliklere özgü talimatlar için [Kullanım kılavuzu](/docs/user-guide/).
- Komutlar, yollar, hizmetler ve yapılandırma sözleşmeleri için [Referans](/docs/reference/).
- Mimari, paket sahipliği ve katkı iş akışları için [Geliştirici kılavuzu](/docs/developer-guide/).