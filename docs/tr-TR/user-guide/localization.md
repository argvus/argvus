---
title: Yerelleştirme
description: ARGVUS'u İngilizce veya Portekizce kullanın.
---

`argvus-i18n` Paylaşılan kataloglar sağlar, the provides the shared catalogs, the the provides the shared catalogs, the the provides the shared catalogs, the the the provides the shared catalogs, the the provides the shared catalogs, the the the provides the shared catalogs, the the the provides the shared catalogs, the the the the shared catalogs, the `argvus i18n validate` komut ve QML modül modülü `org.argvus.i18n`. ARGVUS Gemiler 18 arayüz dilleri: Arapça, Bengali, Almanca, İngilizce, İspanyolca, Fransızca, Hindi, Endonezya, İtalya, Japonca, Kore, Brezilyalı Portekiz, Avrupa Portekiz, Rusça, Türkçe, Urdu, Vietnam ve Basitleştirilmiş Çin.

Yerli Yerli Yerli Yerli ARGVUS Uygulamalar ve kabuk bileşenleri paylaşılan katalogları tüketmektedir. Bu nedenle tercüme edilmiş bir dize birden fazla havuz tarafından kullanılabilir. Arapça ve Urdu desteklenmekte olan sağ sol düzeni kullanır GTK, Quickshell ve terminal arayüzleri.

Use **Control Center → Tercihler → Dil & Bölge → Dil** seçmek için ARGVUS arayüz dili. Bu yazıyor `ARGVUS_LANGUAGE` toklanmak için `$XDG_CONFIG_HOME/locale.conf` (normal olarak `~/.config/locale.conf`). Örneğin, `ARGVUS_LANGUAGE=pt-PT` Avrupa Portekiz'i seçin. Brezilyalı Portekiz (İngilizce).`pt-BR`) ayrı bir seçenek. Dil katalogu ile yüklenir ARGVUSAma buna bağlı değildir `locale-gen`.

Kontrol Panel, arayüz dili değişikliklerinden sonra otomatik olarak yeniden başlar. Oturum bileşenleri için uygulamak için yeni bir oturum başlatın. The separate **Regional Locale** setting Write `LANG` ve bölgesel formatları ve üçüncü taraf uygulamalarını kontrol edin. Oluşturulmuş bir sistem yerel gerektirir; biri izin vermek `/etc/locale.gen` run run run run run run run `locale-gen` Yönetici izni ile. Klavye düzeni, değişken ve konsol anahtarmap ayrı ayarlardır.

## Çevre değişimini değiştirirken

ARGVUS Başvuruları oku `ARGVUS_LANGUAGE` Kullanıcınınkinden `locale.conf`Eğer yok ise, bölgeyi çözdüler. `LANG` Oluşturulan bir kataloga değer. İnherited locale değişkenleri açıklanmadı `ARGVUS_LANGUAGE` tercih. Üçüncü taraf uygulamaları sistemi bölgesel yerel kullanmaya devam ediyor.

Etkili değerleri göz önünde bulundurun:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

Check that Check that Check that `~/.config/locale.conf` Geçerli bir doğrulama içerir `ARGVUS_LANGUAGE=<catalog>` değer. Bölgesel formatlama veya üçüncü taraf uygulama dili değiştirmek için, **Regional Locale** kullanın ve üretilen bir yerel seçin. Kontrol Panel, tercih edilen değişikliklerden sonra yeniden başlatılır; oturum açma ve diğer oturum bileşenleri hala eski arayüz dilini gösterirse. [Environment değişkenleri]/docs/reference/environment-variables/) teknik sözleşmeler ve öncekiler için.
