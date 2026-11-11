---
title: Yerelleştirme
description: ARGVUS'u İngilizce veya Portekizce kullanın.
---

`argvus-i18n`, paylaşılan katalogları, `argvus i18n validate` komutunu ve `org.argvus.i18n` QML modülünü sağlar. ARGVUS şu anda İngilizce ve Portekizce kataloglar göndermektedir.

Yerel uygulamalar ve kabuk bileşenleri, paylaşılan katalogları kullanır. Bu nedenle çevrilmiş bir dize birden fazla veri havuzu tarafından kullanılabilir.

Dili, bölgesel yerel ayarı, oluşturulan sistem yerel ayarlarını, saat dilimini, tarihi/saati ve klavyeyi yapılandırmak için **Kontrol Merkezi → Yerel Ayar ve Bölge**'yi kullanın. Güncel ARGVUS katalogları İngilizce ve Portekizce sağlar; bu, işletim sisteminin oluşturduğu sistem yerel ayarları listesinden ayrıdır.

Görüntüleme dilinin değiştirilmesi ARGVUS tercihini kaydeder ve `argvus-control-panel.service` otomatik olarak yeniden başlatılır, böylece Kontrol Paneli değişikliği hemen yansıtır. Diğer uygulamalar ve kabuk yüzeyleri oturumun yeniden yüklenmesini veya yeni bir oturumu gerektirebilir. `/etc/locale.gen` girişlerini oluşturmak veya değiştirmek, yetkilendirme gerektirir ve `locale-gen`'yu çalıştırır; oluşturulmayan bir yerel ayarın seçilmesi reddedilir. Klavye düzeni, değişken ve konsol tuş haritası ayrı ayarlardır ve arayüz diliyle karıştırılmamalıdır.

## Ortam değişkenleri değişikliği geçersiz kıldığında

ARGVUS uygulamaları dili önce kalıcı ARGVUS dil dosyasından, ardından `LC_ALL`, `LC_MESSAGES` ve `LANG`'dan algılar. Yerel ortam değişkenleri bu nedenle, özellikle bir kullanıcı hizmeti veya grafik oturumu eski bir ortamı koruduğunda, Kontrol Merkezinin bir dil değişikliğini görmezden geliyormuş gibi görünmesine neden olabilir.

Etkin değerleri aşağıdakilerle inceleyin:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

`~/.config/locale.conf` mevcutsa ve `LANG`, `LC_ALL` veya `LC_MESSAGES`'yu tanımlıyorsa, bu değerleri artık yeniden vermemesi için onu geçici olarak yeniden adlandırın. Silmek yerine yedek tutun:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

**Denetim Merkezi → Yerel Ayar ve Bölge**'de dili tekrar uygulayın. Kontrol Paneli otomatik olarak yeniden başlatılır; diğer bileşenler hala önceki dili gösteriyorsa, oturumu kapatıp açın, böylece grafik oturumu ve `systemd --user` yeni ortamı alır. Daha sonra teşhis komutlarını tekrarlayın. Teknik sözleşmeler ve öncelikler için bkz. [Ortam değişkenleri](/docs/reference/environment-variables/).