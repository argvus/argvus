---
title: ARGVUS Uluslararasılaştırma
description: Dil katalogları, seçimi ve Unicode desteği.
---

# ARGVUS'un Uluslararasılaşması

ARGVUS, İngilizce (`en-US`), Brezilya Portekizcesi (`pt-BR`), İspanyolca (`es-ES`),
Fransızca (`fr-FR`), İtalyanca (`it-IT`), Almanca (`de-DE`), Rusça (`ru-RU`), Basitleştirilmiş
Çince (`zh-CN`), Japonca (`ja-JP`) ve Korece (`ko-KR`). İngilizce kanoniktir
katalog ve geri dönüş. Sekiz ek çeviri makineyle üretildi
çeviri yardımı ve teknik düzeltmeler; bağımsız anadil konuşmacısı incelemesi
sertifikalandırılmamıştır.

## Bir dil seçin

Denetim Merkezi > Tercihler > Dil ve Bölge > Dil veya Ayarlar > öğesini açın.
Dil. Sayfa, yüklü katalogları keşfeder ve yerel adlarını gösterir.
`LANG`'yu `$XDG_CONFIG_HOME/locale.conf`'ya kaydeder (varsayılan `~/.config/locale.conf`).
Önce Sistem Yerel Ayarlarında ilgili UTF-8 sistem yerel ayarını etkinleştirin. Örnekler
`es_ES.UTF-8`, `zh_CN.UTF-8`, `ja_JP.UTF-8` ve `ko_KR.UTF-8` içerir.

Tercih, `/etc/locale.conf`'yu değiştirmez. Yorumlar ve bölgesel kategori
ayarlar korunur. Kaydedildikten sonra Kontrol Paneli yeniden başlatılır; oturumu kapat ve
Oturum boyunca dili uygulamak için tekrar içeri girin. Yapılandırılmamış kullanıcıların kullanımı
İngilizce. Devralınan ortam değişkenleri ve eski ARGVUS dil dosyaları
bu tercihi değiştirin. Oturum açma selamlayıcısı, kullanıcının tercihi yapılmadan önce çalışır.
mevcuttur ve kendi hizmet hesabı dilini (varsayılan olarak İngilizce) korur.

Bölgesel değişkenler, kesin bir bilgi olduğunda kendi dilleri için yüklü kataloğu kullanır.
katalog yok; örneğin `es_MX.UTF-8`, `es-ES`'yu kullanır. `zh-Hans` seçer
Basitleştirilmiş Çince. Geleneksel Çince (`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`)
Basitleştirilmiş Çince ile sessizce eşleştirilmez ve İngilizceye geri döner.

## Kataloglar ve katkılar

Kaynak, argvus deposundaki `resources/locales/<locale>/<domain>.json`'dur.
Yüklü kataloglar  `argvus` `/usr/share/argvus/i18n/`'da yayınlanmaktadır. Ustayla gönderiyorlar
`argvus` paketi. `locale.json` tanımlayıcıyı ve yerel/görünen adları sağlar.
Ayrı bir PO kataloğu veya gerekli çevrimiçi çeviri platformu yoktur.

Anlamsal anahtarları kullanın ve yer tutucu adlarını, komutları, seçenekleri, yolları ve bilgileri koruyun.
klavye kısayolları. Cümleleri birleştirmek yerine mesajların tamamını çevirin
çevrilmiş parçalardan. Yeni kanonik anahtarların onda da çeviriye ihtiyacı var
diller. Her alan adında terminolojiyi tutarlı tutun ve çevirileri inceleyin
bağlamda, özellikle kimlik doğrulama, izinler ve yıkıcı eylemler.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```

`ARGVUS_I18N_DIR`, geliştirme ve aşamalandırma için alternatif bir katalog kökü seçer
paket kontrolleri. Doğrulama, etki alanlarını, anahtarları, dize değerlerini, yer tutucuları,
boş çeviriler ve yerel ayara özgü tamsayı çoğul formları. Doğrulama kanıtlıyor
Dilsel doğruluk değil, yapısal bütünlük.

## Unicode ve yazı tipleri

`argvus-fonts`, Latince, sembol ve emojiye ek olarak Noto CJK yazı tiplerini gerektirir
aileler. Fontconfig, Çince, Japonca ve Hangul için geri dönüş sağlar.
kullanıcının seçtiği birincil aileyi değiştiriyor. Uygulamalar ve terminaller hala
yazı tipi geri dönüşünün etkinleştirilmesi gerekiyor.

Paylaşılan Ratatui metin yardımcıları terminal hücrelerini ölçer, genişletilmiş grafiği korur
kesme ve geniş metni sarma sırasında kümeler. Metin editörleri taşır ve siler
bütün grafikler. Son derece dar alanlar iki hücreli bir glifi tek bir alanda görüntüleyemez
hücre; bu durumda sert sarma yardımcısı bir değiştirme karakteri kullanır. Emoji ve
belirsiz genişlikteki karakterler, terminal/yazı tipi uygulamaları arasında hala farklılık gösterebilir.
Terminalin genişlik politikası Ratatui'nin normal Unicode genişliğine uygun olmalıdır.

Dil seçimi bir giriş yöntemi yüklemez veya klavye düzenini değiştirmez.Çince, Japonca veya Korece girerken Wayland uyumlu bir giriş yöntemi kullanın.
