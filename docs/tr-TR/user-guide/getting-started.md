---
title: İlk yapılandırma
description: ARGVUS'u yapılandırmak için pratik bir ilk önyükleme yolu.
---

İlk grafik oturumundan sonra ARGVUS'un her parçasını yapılandırmanıza gerek yoktur. Masaüstünün nasıl hissettiğini etkileyen ayarlarla başlayın, ardından donanım ve hizmetleri gerektiği gibi yapılandırın.

## Yararlı bir düzen

1. Pencereleri, çalışma alanlarını, görev çubuğunu, Denetim Masasını ve başlatıcıyı tanımlamak için [masaüstüne genel bakış](/docs/user-guide/desktop/) bölümünü okuyun.
2. [Kontrol Merkezini](/docs/components/argvus-control-center/) açın ve **Görünüm** bölümünden bir tema, mod, vurgu ve duvar kağıdı seçin.
3. Varsayılan boşluklar, kenarlıklar veya görev çubuğu aralığı ekranınıza uymuyorsa [pencereler ve düzen](/docs/argvus-hyprland/windows-and-layout/) ayarını yapın.
4. [Görev çubuğunu](/docs/argvus-taskbar/taskbar/) yapılandırın ve hangi Kontrol Paneli kartlarının görünür olmasını istediğinize karar verin.
5. [Klavye kısayollarını](/docs/argvus-hyprland/keyboard-shortcuts/) inceleyin ve yalnızca gerçekten kullandığınız bağlamaları değiştirin.
6. İlgili ayarlar sayfalarından [fare ve dokunmatik yüzeyi](/docs/argvus-hyprland/input/), ekranları, ağı ve gücü yapılandırın.

İlk dört adım isteğe bağlı kişiselleştirmedir. Geri kalan adımlar normalde yalnızca donanımınız, diliniz veya iş akışınız gerektirdiğinde gereklidir.

## Ayarları değiştirmek için iki yer

ARGVUS'un nasıl yapılandırılması gerektiğini açıklayan ayarlar için **Kontrol Merkezi**'ni kullanın: görünüm, düzen, yazı tipleri, giriş, kısayollar, dil, bölge ve varsayılan uygulamalar. Ses seviyesi, parlaklık, ağ durumu, bildirimler, güç ve oturum kontrolleri gibi durum ve sık oturum eylemleri için **Denetim Masası**'nı kullanın.

Tam ayrım için [Kontrol Merkezi ve Kontrol Paneli](/docs/components/argvus-control-center/) ve hızlı bir dizin için [İşlerin nerede yapılandırılacağı](/docs/user-guide/where-to-configure/) konusuna bakın.

## Değişiklikler, kalıcılık ve iyileşme

Çoğu Kontrol Merkezi değişikliği, oturum çalışırken uygulanır ve ARGVUS kullanıcı durumu olarak kaydedilir. Görünüm değişiklikleri kurallı yapılandırmaya kaydedilir ve `argvus-config`, Hyprland tarafından tüketilen dosyaları ve ondan gelen kabuğu yansıtır. Denetim Masası geçişi, kalıcı bir yapılandırma yerine doğrudan bir eylem veya oturum durumu değişikliği olabilir.

Bir sayfada **Varsayılanları sıfırla** seçeneği sunulduğunda, oluşturulan dosyaları silmek yerine o sayfa işlemini kullanın. Görünümün vurgular, duvar kağıtları ve düzen değerleri için ayrı sıfırlama eylemleri vardır; tuş bağlama sayfasının kendi geri yükleme işlemi vardır. Her masaüstü ayarı için tek bir evrensel sıfırlama düğmesi yoktur.

## Sonraki adımlar

- [Görünüm](/docs/user-guide/appearance/) — temalar, duvar kağıtları, efektler ve düzen.
- [Masaüstü düzeni](/docs/argvus-hyprland/windows-and-layout/) — boşlukları, sınırları ve panel alanını anlayın.
- [Giriş aygıtları](/docs/argvus-hyprland/input/) — fare ve dokunmatik yüzey kontrolleri.
- [Tema ve duvar kağıdı sorunlarını giderme](/docs/user-guide/troubleshooting/theme-and-wallpaper/) — görsel bir değişiklik beklediğiniz gibi olmadığında kurtarma.